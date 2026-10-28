const CHANNELS = 4;
const HIGH = 4 << 20;
const LOW = 1 << 20;

export type Path = 'direct' | 'relayed' | 'tcp';

export type Signal = { sdp: RTCSessionDescriptionInit } | { candidate: RTCIceCandidateInit };

// the worker mints turn credentials for 12 hours; fetch fresh ones well before that
const ICE_MAX_AGE_MS = 3600_000;

let servers: Promise<RTCIceServer[]> | undefined;
let fetched = 0;

function iceServers() {
	if (!servers || performance.now() - fetched > ICE_MAX_AGE_MS) {
		fetched = performance.now();
		servers = fetch('/turn/credentials')
			.then((r) => (r.ok ? r.json() : Promise.reject()))
			.then((r: { iceServers: RTCIceServer[] }) => r.iceServers)
			.catch(() => [{ urls: 'stun:stun.cloudflare.com:3478' }]);
	}
	return servers;
}

export class Peer {
	onopen = () => {};
	onmessage = (_: Uint8Array) => {};
	onclose = () => {};

	private pc?: RTCPeerConnection;
	private channels: RTCDataChannel[] = [];
	private pending: RTCIceCandidateInit[] = [];
	private open = 0;
	private waiters: (() => void)[] = [];
	private signals: Signal[] = [];
	private sink?: (signal: Signal) => void;
	private dead = false;
	private started?: Promise<void>;

	constructor(private offerer: boolean) {}

	set onsignal(sink: (signal: Signal) => void) {
		this.sink = sink;
		this.signals.splice(0).forEach(sink);
	}

	private emit(signal: Signal) {
		if (this.sink) this.sink(signal);
		else this.signals.push(signal);
	}

	start() {
		return (this.started ??= this.init());
	}

	private async init() {
		const ice = await iceServers();
		if (this.dead) return;
		const pc = new RTCPeerConnection({ iceServers: ice });
		this.pc = pc;
		pc.onicecandidate = (e) => e.candidate && this.emit({ candidate: e.candidate.toJSON() });
		pc.onconnectionstatechange = () => {
			if (pc.connectionState === 'failed' || pc.connectionState === 'closed') this.close();
		};
		for (let id = 0; id < CHANNELS; id++) {
			const ch = pc.createDataChannel(`nope${id}`, { negotiated: true, id });
			ch.binaryType = 'arraybuffer';
			ch.bufferedAmountLowThreshold = LOW;
			ch.onopen = () => ++this.open === CHANNELS && this.onopen();
			ch.onclose = () => this.close();
			ch.onmessage = (e) => this.onmessage(new Uint8Array(e.data));
			ch.onbufferedamountlow = () => this.wake();
			this.channels.push(ch);
		}
		if (this.offerer) {
			await pc.setLocalDescription();
			this.emit({ sdp: pc.localDescription!.toJSON() });
		}
	}

	async signal(msg: Signal) {
		await this.start();
		const pc = this.pc;
		if (!pc || this.dead) return;
		if ('sdp' in msg) {
			await pc.setRemoteDescription(msg.sdp);
			for (const c of this.pending.splice(0)) await pc.addIceCandidate(c);
			if (msg.sdp.type === 'offer') {
				await pc.setLocalDescription();
				this.emit({ sdp: pc.localDescription!.toJSON() });
			}
		} else if (pc.remoteDescription) {
			await pc.addIceCandidate(msg.candidate);
		} else {
			this.pending.push(msg.candidate);
		}
	}

	send(msg: Uint8Array) {
		if (this.dead) return;
		// small frames are control frames and keep their order on one channel
		const ch =
			msg.length < 1024
				? this.channels[0]
				: this.channels.reduce((a, b) => (b.bufferedAmount < a.bufferedAmount ? b : a));
		ch.send(msg as Uint8Array<ArrayBuffer>);
	}

	async drain() {
		while (!this.dead && this.channels.some((ch) => ch.bufferedAmount > HIGH)) {
			await new Promise<void>((resolve) => this.waiters.push(resolve));
		}
	}

	async path(): Promise<Path> {
		const stats = await this.pc!.getStats();
		for (const r of stats.values()) {
			if (r.type === 'candidate-pair' && r.nominated && r.state === 'succeeded') {
				const local = stats.get(r.localCandidateId);
				const remote = stats.get(r.remoteCandidateId);
				if (local?.relayProtocol === 'tcp' || local?.relayProtocol === 'tls') return 'tcp';
				if (local?.candidateType === 'relay' || remote?.candidateType === 'relay') return 'relayed';
			}
		}
		return 'direct';
	}

	private wake() {
		this.waiters.splice(0).forEach((resolve) => resolve());
	}

	close() {
		if (this.dead) return;
		this.dead = true;
		this.wake();
		this.pc?.close();
		this.onclose();
	}
}
