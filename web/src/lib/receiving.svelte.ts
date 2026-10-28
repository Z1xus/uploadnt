import { Attempt } from './attempt';
import { parseLink, type JsFile, type LobbyEvent, type ParsedLink, type ReceiverSession } from './core';
import { openSink, sweep, type Sink } from './disk';
import { grievanceOf, type Grievance } from './grievance';
import { Lobby } from './lobby';
import { Peer, relayServers, type Path } from './peer';
import { Rate, percent } from './rate';

type Phase = 'connecting' | 'ask' | 'offered' | 'receiving' | 'done' | 'failed';

const TICK_MS = 1000;
// direct usually connects within 1 to 3 s of meeting, a relay can take longer
const DIRECT_MS = 5_000;
const RELAY_MS = 20_000;

export class Receiving {
	phase = $state<Phase>('connecting');
	files = $state<JsFile[]>([]);
	size = $state(0);
	bytes = $state(0);
	rate = $state(0);
	path = $state<Path>('direct');
	alone = $state(false);
	blocked = $state(false);
	where = $state('');
	grievance = $state<Grievance>();
	error = $state<string>();
	again = $state(false);

	private link?: ParsedLink;
	private lobby?: Lobby;
	private attempt?: Attempt;
	private session?: ReceiverSession;
	private sink?: Sink;
	private chunks = 0;
	private met = false;
	private relay = false;
	private open = false;
	private taking = false;
	private finishing = false;
	private committed?: Uint8Array;
	private meter = new Rate();
	private timer?: ReturnType<typeof setInterval>;
	private alone_timer?: ReturnType<typeof setTimeout>;
	private reach_timer?: ReturnType<typeof setTimeout>;

	constructor(fragment: string) {
		this.link = parseLink(fragment);
		if (!this.link) {
			this.fail('link');
			return;
		}
		sweep();
		this.lobby = new Lobby(Lobby.url(this.link.mailbox, this.link.lobby), (e) => this.onLobby(e));
		this.alone_timer = setTimeout(() => (this.alone = true), 4000);
		this.timer = setInterval(() => {
			this.session?.tick(performance.now());
			this.flush();
		}, TICK_MS);
	}

	async take() {
		const root = this.session?.root();
		if (this.taking || !root) return;
		this.taking = true;
		try {
			this.sink = await openSink(root, this.files, this.size);
		} catch (e) {
			if ((e as Error).name !== 'AbortError') this.fail(e);
			return;
		} finally {
			this.taking = false;
		}
		this.where = this.sink.where;
		this.phase = 'receiving';
		this.accept();
	}

	redownload() {
		this.sink?.again?.().catch((err) => this.fail(err));
	}

	planb() {
		this.relay = true;
		this.phase = 'connecting';
		this.reach_timer = setTimeout(() => this.unreachable(), RELAY_MS);
		this.lobby?.reconnect();
	}

	private async unreachable() {
		this.reach_timer = undefined;
		const waiting = () => !this.open && this.phase !== 'done' && this.phase !== 'failed';
		if (!waiting()) return;
		this.blocked = !!(await this.attempt?.peer.blocked());
		const relay = !this.relay && (await relayServers(this.lobby!.turn));
		if (!waiting()) return;
		if (relay) this.phase = 'ask';
		else this.fail(this.blocked ? 'webrtc' : 'unreachable');
	}

	get peer() {
		return this.attempt?.peer;
	}

	close() {
		clearInterval(this.timer);
		clearTimeout(this.alone_timer);
		clearTimeout(this.reach_timer);
		this.lobby?.close();
		this.attempt?.close();
		if (this.phase !== 'done') this.sink?.abort();
	}

	private async accept() {
		const session = this.session;
		const root = session?.root();
		if (!session || !root) return;
		try {
			const have = await this.sink!.open(root, this.files, this.chunks);
			if (session !== this.session) return;
			session.accept(have, performance.now());
			this.phase = 'receiving';
			this.events();
		} catch (err) {
			this.fail(err);
		}
	}

	private onLobby(e: LobbyEvent) {
		try {
			switch (e.type) {
				case 'peer':
					if (!e.present) break;
					if (this.phase === 'done') this.reset();
					this.connect();
					break;
				case 'pake':
					this.attempt?.onPake(e.data);
					break;
				case 'sealed': {
					const attempt = this.attempt;
					attempt?.onSealed(e.data).catch((err) => {
						if (this.attempt === attempt) this.fail(err);
					});
					break;
				}
				case 'full':
					if (!this.met) this.fail('full');
			}
		} catch (err) {
			this.fail(err);
		}
	}

	private connect() {
		this.met = true;
		this.alone = false;
		clearTimeout(this.alone_timer);
		this.open = false;
		this.reach_timer ??= setTimeout(() => this.unreachable(), this.relay ? RELAY_MS : DIRECT_MS);
		const old = this.attempt;
		this.attempt = undefined;
		old?.close();
		this.session = undefined;
		const peer = new Peer(false, this.relay ? this.lobby!.turn : undefined);
		const attempt = new Attempt(this.link!.secret, false, this.lobby!, peer, this.relay);
		this.attempt = attempt;
		peer.onopen = async () => {
			try {
				this.open = true;
				clearTimeout(this.reach_timer);
				this.reach_timer = undefined;
				this.session = attempt.keys!.receiver();
				// phones with 2 GB or less cannot hold 32 chunks in flight
				if ('deviceMemory' in navigator && (navigator.deviceMemory as number) <= 2)
					this.session.setMaxWindow(8);
				peer.onmessage = (msg) => {
					if (this.attempt !== attempt || !this.session) return;
					try {
						this.session.handle(msg, performance.now());
						this.events();
					} catch (err) {
						this.fail(err);
					}
				};
				this.path = await peer.path();
				this.flush();
			} catch (err) {
				this.fail(err);
			}
		};
		peer.onclose = () => {
			if (this.attempt === attempt && this.phase !== 'done' && this.phase !== 'failed')
				this.lobby!.reconnect();
		};
		peer.start();
	}

	private events() {
		const session = this.session!;
		for (let batch = session.events(); batch.length; batch = session.events())
			for (const e of batch) {
				switch (e.type) {
					case 'manifest':
						this.files = e.files;
						this.size = e.size;
						this.chunks = e.chunks;
						if (this.committed) session.accept(this.committed, performance.now());
						else if (this.finishing) break;
						else if (this.sink) this.accept();
						else this.phase = 'offered';
						break;
					case 'write':
						this.sink!.write(e.file, e.offset, e.chunk, e.data).then(
							() => {
								if (session !== this.session) return;
								session.written(e.chunk, performance.now());
								this.events();
							},
							(err) => this.fail(err)
						);
						break;
					case 'progress':
						this.bytes = e.bytes;
						this.rate = this.meter.push(e.bytes);
						document.title = `${percent(e.bytes, e.total)}% uploadn't`;
						break;
					case 'done':
						if (this.committed) session.complete();
						else this.finish();
						break;
					case 'closed':
						this.fail('left');
				}
			}
		this.flush();
	}

	private async finish() {
		const session = this.session!;
		this.finishing = true;
		const have = session.have();
		try {
			await this.sink!.finish();
		} catch (err) {
			this.fail(err);
			return;
		}
		this.committed = have;
		// a reconnect during the commit leaves a newer session that still needs the done frame
		if (this.session === session) session.complete();
		else if (this.session?.root()) this.session.accept(have, performance.now());
		if (this.session) this.events();
		this.phase = 'done';
		this.again = !!this.sink?.again;
		document.title = "uploadn't";
		const attempt = this.attempt;
		// closing the connection throws away whatever is still queued, so we wait a second and pray
		setTimeout(() => attempt?.close(), 1000);
	}

	private reset() {
		this.files = [];
		this.size = this.bytes = this.rate = this.chunks = 0;
		this.where = '';
		this.again = false;
		this.sink = undefined;
		this.finishing = false;
		this.committed = undefined;
		this.meter = new Rate();
		this.phase = 'connecting';
	}

	private flush() {
		const peer = this.attempt?.peer;
		if (!peer || !this.session) return;
		for (const msg of this.session.transmit()) peer.send(msg);
	}

	private fail(err: unknown) {
		this.grievance = grievanceOf(err);
		this.error = String(err);
		this.phase = 'failed';
		this.close();
	}
}
