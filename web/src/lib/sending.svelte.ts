import { Attempt } from './attempt';
import {
	ManifestBuilder,
	newSecret,
	parseLink,
	type CoreEvent,
	type LobbyEvent,
	type SenderSession
} from './core';
import { grievanceOf, type Grievance } from './grievance';
import { Lobby } from './lobby';
import { Peer, type Path } from './peer';
import { Rate } from './rate';

type Phase = 'waiting' | 'sending' | 'done' | 'failed';

export type Pending = { secret: string; files: { name: string; size: number }[]; root?: string; at: number };

const PENDING = 'sending.json';
const PENDING_MAX_AGE_MS = 24 * 3600_000;

const describe = (files: File[]) =>
	files.map((f) => ({ name: f.webkitRelativePath || f.name, size: f.size }));

export async function pending(): Promise<Pending | undefined> {
	try {
		const dir = await navigator.storage.getDirectory();
		const p: Pending = JSON.parse(await (await (await dir.getFileHandle(PENDING)).getFile()).text());
		return Date.now() - p.at < PENDING_MAX_AGE_MS ? p : undefined;
	} catch {
		return undefined;
	}
}

function record(update: (current?: Pending) => Pending | undefined) {
	return navigator.locks
		.request(PENDING, async () => {
			const dir = await navigator.storage.getDirectory();
			const next = update(await pending());
			if (!next) return await dir.removeEntry(PENDING).catch(() => {});
			const out = await (await dir.getFileHandle(PENDING, { create: true })).createWritable();
			await out.write(JSON.stringify(next));
			await out.close();
		})
		.catch(() => {});
}

export class Sending {
	phase = $state<Phase>('waiting');
	hashed = $state(0);
	bytes = $state(0);
	rate = $state(0);
	path = $state<Path>('direct');
	grievance = $state<Grievance>();
	link = $state('');
	readonly total: number;

	private secret: string;
	private lobby?: Lobby;
	private manifest: Promise<ManifestBuilder>;
	private attempt?: Attempt;
	private spare = this.prewarm();
	private session?: SenderSession;
	private reads: Extract<CoreEvent, { type: 'read' }>[] = [];
	private reading = false;
	private meter = new Rate();
	private lock?: WakeLockSentinel;

	constructor(
		readonly files: File[],
		previous?: Pending
	) {
		const same = JSON.stringify(previous?.files) === JSON.stringify(describe(files));
		// a reused link must carry the same bytes, so wait for the root before joining
		const check = same ? previous!.root : undefined;
		this.secret = same ? previous!.secret : newSecret();
		this.link = `${location.origin}/#${this.secret}`;
		this.total = files.reduce((n, f) => n + f.size, 0);
		this.manifest = this.hash();
		record(() => ({ secret: this.secret, files: describe(files), root: check, at: Date.now() }));
		this.manifest.then((builder) => {
			const root = builder.root();
			if (check && root !== check) {
				this.secret = newSecret();
				this.link = `${location.origin}/#${this.secret}`;
			}
			record(() => ({ secret: this.secret, files: describe(files), root, at: Date.now() }));
			if (check) this.join();
		});
		if (!check) this.join();
	}

	private join() {
		if (this.phase === 'failed') return;
		this.lobby = new Lobby(Lobby.url(parseLink(this.secret)!.mailbox), (e) => this.onLobby(e));
	}

	close() {
		this.lobby?.close();
		this.attempt?.close();
		this.lock?.release();
	}

	private prewarm() {
		const peer = new Peer(true);
		peer.start();
		return peer;
	}

	private async hash() {
		const builder = new ManifestBuilder(
			this.files.map((f) => ({ name: f.webkitRelativePath || f.name, size: f.size, mime: f.type }))
		);
		let hashed = 0;
		for (let span = builder.next(); span; span = builder.next()) {
			const file = this.files[span.file];
			builder.push(new Uint8Array(await file.slice(span.offset, span.offset + span.len).arrayBuffer()));
			hashed += span.len;
			this.hashed = hashed;
		}
		this.hashed = this.total;
		return builder;
	}

	private onLobby(e: LobbyEvent) {
		try {
			switch (e.type) {
				case 'peer':
					if (e.present && this.phase !== 'done') this.connect();
					break;
				case 'pake':
					this.attempt?.onPake(e.data);
					break;
				case 'sealed': {
					const attempt = this.attempt;
					attempt?.onSealed(e.data).catch((err) => {
						if (this.attempt === attempt) this.fail(err);
					});
				}
			}
		} catch (err) {
			this.fail(err);
		}
	}

	private connect() {
		const old = this.attempt;
		this.attempt = undefined;
		old?.close();
		this.session = undefined;
		this.reads = [];
		let peer = this.spare;
		if (peer.stale) {
			peer.close();
			peer = this.prewarm();
		}
		this.spare = this.prewarm();
		const attempt = new Attempt(this.secret, true, this.lobby!, peer);
		this.attempt = attempt;
		peer.onopen = async () => {
			const builder = await this.manifest;
			if (this.attempt !== attempt) return;
			this.session = attempt.keys!.sender(builder);
			this.path = await peer.path();
			this.flush();
		};
		peer.onmessage = (msg) => {
			if (this.attempt !== attempt || !this.session) return;
			try {
				this.session.handle(msg);
				this.events();
			} catch (err) {
				this.fail(err);
			}
		};
	}

	private events() {
		for (const e of this.session!.events()) {
			switch (e.type) {
				case 'read':
					this.reads.push(e);
					break;
				case 'progress':
					if (this.phase === 'waiting') this.begin();
					this.bytes = e.bytes;
					this.rate = this.meter.push(e.bytes);
					document.title = `${Math.floor((100 * e.bytes) / Math.max(e.total, 1))}% · uploadn't`;
					break;
				case 'done':
					this.phase = 'done';
					this.bytes = this.total;
					document.title = "uploadn't";
					record((p) => (p?.secret === this.secret ? undefined : p));
					this.close();
					break;
				case 'closed':
					this.fail(new Error('left'));
			}
		}
		this.read();
		this.flush();
	}

	private begin() {
		this.phase = 'sending';
		navigator.wakeLock
			?.request('screen')
			.then((lock) => (this.lock = lock))
			.catch(() => {});
	}

	private async read() {
		if (this.reading) return;
		this.reading = true;
		try {
			const attempt = this.attempt;
			for (let r = this.reads.shift(); r; r = this.reads.shift()) {
				const data = await this.files[r.file].slice(r.offset, r.offset + r.len).arrayBuffer();
				if (this.attempt !== attempt) return;
				this.session!.provide(r.chunk, new Uint8Array(data));
				await this.flush();
			}
		} finally {
			this.reading = false;
		}
	}

	private async flush() {
		const peer = this.attempt?.peer;
		if (!peer || !this.session) return;
		for (const msg of this.session.transmit()) peer.send(msg);
		await peer.drain();
	}

	private fail(err: unknown) {
		const msg = String((err as Error)?.message ?? err);
		this.grievance = msg === 'left' ? 'left' : grievanceOf(err);
		this.phase = 'failed';
		this.close();
	}
}
