import { Attempt } from './attempt';
import { parseLink, type JsFile, type LobbyEvent, type ParsedLink, type ReceiverSession } from './core';
import { openSink, sweep, type Sink } from './disk';
import { grievanceOf, type Grievance } from './grievance';
import { Lobby } from './lobby';
import { Peer, relayServers, type Path } from './peer';
import { Rate } from './rate';

type Phase = 'connecting' | 'ask' | 'offered' | 'receiving' | 'done' | 'failed';

const TICK_MS = 1000;
const UNREACHABLE_MS = 20_000;

export class Receiving {
	phase = $state<Phase>('connecting');
	files = $state<JsFile[]>([]);
	size = $state(0);
	bytes = $state(0);
	rate = $state(0);
	path = $state<Path>('direct');
	alone = $state(false);
	where = $state('');
	grievance = $state<Grievance>();

	private link?: ParsedLink;
	private lobby?: Lobby;
	private attempt?: Attempt;
	private session?: ReceiverSession;
	private sink?: Sink;
	private chunks = 0;
	private met = false;
	private relay = false;
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
			if ((e as Error).name !== 'AbortError') this.fail(grievanceOf(e));
			return;
		} finally {
			this.taking = false;
		}
		this.where = this.sink.where;
		this.phase = 'receiving';
		this.accept();
	}

	planb() {
		this.relay = true;
		this.phase = 'connecting';
		this.reach_timer = setTimeout(() => this.unreachable(), UNREACHABLE_MS);
		this.lobby?.reconnect();
	}

	private async unreachable() {
		if (this.phase !== 'connecting') return;
		const relay = !this.relay && (await relayServers());
		if (this.phase !== 'connecting') return;
		if (relay) this.phase = 'ask';
		else this.fail('unreachable');
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
			this.events();
		} catch (err) {
			this.fail(grievanceOf(err));
		}
	}

	private onLobby(e: LobbyEvent) {
		try {
			switch (e.type) {
				case 'peer':
					if (e.present) this.connect();
					break;
				case 'pake':
					this.attempt?.onPake(e.data);
					break;
				case 'sealed': {
					const attempt = this.attempt;
					attempt?.onSealed(e.data).catch((err) => {
						if (this.attempt === attempt) this.fail(grievanceOf(err));
					});
					break;
				}
				case 'full':
					if (!this.met) this.fail('full');
			}
		} catch (err) {
			this.fail(grievanceOf(err));
		}
	}

	private connect() {
		this.met = true;
		this.alone = false;
		clearTimeout(this.alone_timer);
		this.reach_timer ??= setTimeout(() => this.unreachable(), UNREACHABLE_MS);
		const old = this.attempt;
		this.attempt = undefined;
		old?.close();
		this.session = undefined;
		const peer = new Peer(false, this.relay);
		const attempt = new Attempt(this.link!.secret, false, this.lobby!, peer, this.relay);
		this.attempt = attempt;
		peer.onopen = async () => {
			clearTimeout(this.reach_timer);
			this.reach_timer = undefined;
			this.session = attempt.keys!.receiver();
			// phones with 2 GB or less cannot hold 32 chunks in flight
			if ('deviceMemory' in navigator && (navigator.deviceMemory as number) <= 2)
				this.session.setMaxWindow(8);
			this.path = await peer.path();
			this.flush();
		};
		peer.onmessage = (msg) => {
			if (this.attempt !== attempt || !this.session) return;
			try {
				this.session.handle(msg, performance.now());
				this.events();
			} catch (err) {
				this.fail(grievanceOf(err));
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
		for (const e of session.events()) {
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
						(err) => this.fail(grievanceOf(err))
					);
					break;
				case 'progress':
					this.bytes = e.bytes;
					this.rate = this.meter.push(e.bytes);
					document.title = `${Math.floor((100 * e.bytes) / Math.max(e.total, 1))}% · uploadn't`;
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
			this.fail(grievanceOf(err));
			return;
		}
		this.committed = have;
		// a reconnect during the commit leaves a newer session that still needs the done frame
		if (this.session === session) session.complete();
		else if (this.session?.root()) this.session.accept(have, performance.now());
		if (this.session) this.events();
		this.phase = 'done';
		document.title = "uploadn't";
		// closing the connection throws away whatever is still queued, so we wait a second and pray
		setTimeout(() => this.close(), 1000);
	}

	private flush() {
		const peer = this.attempt?.peer;
		if (!peer || !this.session) return;
		for (const msg of this.session.transmit()) peer.send(msg);
	}

	private fail(grievance: Grievance) {
		this.grievance = grievance;
		this.phase = 'failed';
		this.close();
	}
}
