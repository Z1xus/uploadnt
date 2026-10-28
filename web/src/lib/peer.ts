const CHANNELS = 4;
const HIGH = 4 << 20;
const LOW = 1 << 20;

export type Path = 'direct' | 'relayed' | 'tcp';

export type Signal =
	{ sdp: RTCSessionDescriptionInit } | { candidate: RTCIceCandidateInit } | { relay: boolean };

export type Candidate = { type: string; protocol: string; family: string };

export type Report = {
	state: RTCPeerConnectionState;
	local: Candidate[];
	remote: Candidate[];
	pair?: { local: Candidate; remote: Candidate; rtt?: number };
};

const ICE_MAX_AGE_MS = 3600_000;

// only the kind of address, so debug screenshots show no ips. browsers leave mdns-hidden local addresses out of stats
function candidate(c: { candidateType: string; protocol: string; relayProtocol?: string; address?: string }) {
	const a = c.address ?? '';
	const family = !a || a.endsWith('.local') ? 'mdns' : a.includes(':') ? 'ipv6' : 'ipv4';
	return { type: c.candidateType, protocol: c.relayProtocol ?? c.protocol, family };
}

const STUN = [{ urls: 'stun:stun.cloudflare.com:3478' }];

const servers = new Map<string, { at: number; ice: Promise<RTCIceServer[] | undefined> }>();

export function relayServers(url: string) {
	const cached = servers.get(url);
	if (cached && performance.now() - cached.at < ICE_MAX_AGE_MS) return cached.ice;
	const ice = fetch(url)
		.then((r) => (r.ok ? r.json() : Promise.reject()))
		.then((r: { iceServers: RTCIceServer[] }) => r.iceServers)
		.catch(() => void servers.delete(url));
	servers.set(url, { at: performance.now(), ice });
	return ice;
}

export class Peer {
	onopen = () => {};
	onclose = () => {};

	private pc?: RTCPeerConnection;
	private channels: RTCDataChannel[] = [];
	private pending: RTCIceCandidateInit[] = [];
	private open = 0;
	private waiters: (() => void)[] = [];
	private signals: Signal[] = [];
	private sink?: (signal: Signal) => void;
	private inbox: Uint8Array[] = [];
	private reader?: (msg: Uint8Array) => void;
	private dead = false;
	private started?: Promise<void>;

	constructor(
		private offerer: boolean,
		private turn?: string
	) {}

	set onmessage(reader: (msg: Uint8Array) => void) {
		this.reader = reader;
		this.inbox.splice(0).forEach(reader);
	}

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
		const ice = (this.turn && (await relayServers(this.turn))) || STUN;
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
			ch.onmessage = (e) => {
				const msg = new Uint8Array(e.data);
				if (this.reader) this.reader(msg);
				else this.inbox.push(msg);
			};
			ch.onbufferedamountlow = () => this.wake();
			this.channels.push(ch);
		}
		if (this.offerer) {
			await pc.setLocalDescription();
			this.emit({ sdp: pc.localDescription!.toJSON() });
		}
	}

	async relay(turn: string) {
		await this.start();
		const ice = await relayServers(turn);
		const pc = this.pc;
		if (!pc || !ice || this.dead) return false;
		pc.setConfiguration({ iceServers: ice });
		pc.restartIce();
		await pc.setLocalDescription();
		this.emit({ sdp: pc.localDescription!.toJSON() });
		return true;
	}

	async signal(msg: Exclude<Signal, { relay: boolean }>) {
		await this.start();
		const pc = this.pc;
		if (!pc || this.dead) return;
		if ('sdp' in msg) {
			await pc.setRemoteDescription(msg.sdp);
			for (const c of this.pending.splice(0)) await pc.addIceCandidate(c).catch(() => {});
			if (msg.sdp.type === 'offer') {
				await pc.setLocalDescription();
				this.emit({ sdp: pc.localDescription!.toJSON() });
			}
		} else if (pc.remoteDescription) {
			await pc.addIceCandidate(msg.candidate).catch(() => {});
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

	async report(): Promise<Report | undefined> {
		const pc = this.pc;
		if (!pc) return;
		const stats = await pc.getStats();
		const all = [...stats.values()];
		const transport = all.find((r) => r.type === 'transport');
		const pair =
			stats.get(transport?.selectedCandidatePairId) ??
			all.find((r) => r.type === 'candidate-pair' && r.nominated && r.state === 'succeeded');
		return {
			state: pc.connectionState,
			local: all.filter((r) => r.type === 'local-candidate').map(candidate),
			remote: all.filter((r) => r.type === 'remote-candidate').map(candidate),
			pair: pair && {
				local: candidate(stats.get(pair.localCandidateId)),
				remote: candidate(stats.get(pair.remoteCandidateId)),
				rtt: pair.currentRoundTripTime
			}
		};
	}

	async path(): Promise<Path> {
		const pair = (await this.report())?.pair;
		if (pair?.local.type === 'relay' && pair.local.protocol !== 'udp') return 'tcp';
		if (pair?.local.type === 'relay' || pair?.remote.type === 'relay') return 'relayed';
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
