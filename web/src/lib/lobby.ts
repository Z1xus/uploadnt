import { decodeLobby, type LobbyEvent } from './core';

export class Lobby {
	private ws?: WebSocket;
	private closed = false;
	private retry?: ReturnType<typeof setTimeout>;

	constructor(
		private url: string,
		private onEvent: (event: LobbyEvent) => void
	) {
		this.connect();
	}

	static url(mailbox: string, host?: string | null) {
		const secure = host || location.protocol === 'https:';
		return `${secure ? 'wss' : 'ws'}://${host || location.host}/m/${mailbox}`;
	}

	get turn() {
		return `${this.url.replace(/^ws/, 'http')}/turn`;
	}

	report(path: string) {
		if (this.ws?.readyState === WebSocket.OPEN) this.ws.send(path);
	}

	send(bytes: Uint8Array) {
		if (this.ws?.readyState === WebSocket.OPEN) this.ws.send(bytes as Uint8Array<ArrayBuffer>);
	}

	reconnect() {
		this.ws?.close();
	}

	close() {
		this.closed = true;
		clearTimeout(this.retry);
		this.ws?.close();
	}

	private connect() {
		const ws = new WebSocket(this.url);
		ws.binaryType = 'arraybuffer';
		ws.onmessage = (e) => this.onEvent(decodeLobby(new Uint8Array(e.data)));
		ws.onclose = () => {
			if (!this.closed) this.retry = setTimeout(() => this.connect(), 1000);
		};
		this.ws = ws;
	}
}
