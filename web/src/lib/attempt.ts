import { Pake, type Keys } from './core';
import type { Lobby } from './lobby';
import { Peer, type Signal } from './peer';

export class Attempt {
	keys?: Keys;
	private pake: Pake;
	private queue: Signal[] = [];

	constructor(
		secret: string,
		sender: boolean,
		private lobby: Lobby,
		readonly peer: Peer
	) {
		this.pake = new Pake(secret, sender);
		lobby.send(this.pake.message());
		peer.onsignal = (signal) => (this.keys ? this.seal(signal) : this.queue.push(signal));
	}

	onPake(data: Uint8Array) {
		this.keys = this.pake.finish(data);
		this.queue.splice(0).forEach((signal) => this.seal(signal));
	}

	async onSealed(data: Uint8Array) {
		if (!this.keys) return;
		await this.peer.signal(JSON.parse(this.keys.open(data)));
	}

	close() {
		this.peer.close();
		this.pake.free();
		this.keys?.free();
	}

	private seal(signal: Signal) {
		this.lobby.send(this.keys!.seal(JSON.stringify(signal)));
	}
}
