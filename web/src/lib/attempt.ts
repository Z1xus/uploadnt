import { Pake, type Keys } from './core';
import type { Lobby } from './lobby';
import { Peer, type Signal } from './peer';

export class Attempt {
	keys?: Keys;
	private pake: Pake;
	private queue: Signal[] = [];
	// the sender holds its offer until the receiver says whether it agreed to plan b
	private ready: boolean;
	private closed = false;

	constructor(
		secret: string,
		private sender: boolean,
		private lobby: Lobby,
		readonly peer: Peer,
		relay = false
	) {
		this.pake = new Pake(secret, sender);
		this.ready = !sender;
		if (!sender) this.queue.push({ relay });
		lobby.send(this.pake.message());
		peer.onsignal = (signal) => (this.keys && this.ready ? this.seal(signal) : this.queue.push(signal));
	}

	onPake(data: Uint8Array) {
		this.keys = this.pake.finish(data);
		if (this.ready) this.flush();
	}

	async onSealed(data: Uint8Array) {
		if (!this.keys) return;
		const signal: Signal = JSON.parse(this.keys.open(data));
		if (!('relay' in signal)) return await this.peer.signal(signal);
		if (!this.sender || this.ready) return;
		if (signal.relay) {
			const direct = this.queue;
			this.queue = [];
			if (!(await this.peer.relay())) this.queue = direct;
		}
		this.ready = true;
		this.flush();
	}

	close() {
		if (this.closed) return;
		this.closed = true;
		this.peer.close();
		this.pake.free();
		this.keys?.free();
	}

	private flush() {
		this.queue.splice(0).forEach((signal) => this.seal(signal));
	}

	private seal(signal: Signal) {
		this.lobby.send(this.keys!.seal(JSON.stringify(signal)));
	}
}
