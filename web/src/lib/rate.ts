export class Rate {
	private samples: [number, number][] = [];

	push(bytes: number) {
		const now = performance.now();
		this.samples.push([now, bytes]);
		while (this.samples.length > 2 && now - this.samples[0][0] > 3000) this.samples.shift();
		const [t0, b0] = this.samples[0];
		return now > t0 ? ((bytes - b0) * 1000) / (now - t0) : 0;
	}
}

export const percent = (bytes: number, total: number) => Math.floor((100 * bytes) / Math.max(total, 1));

export function size(bytes: number) {
	const units = ['b', 'kb', 'mb', 'gb', 'tb'];
	let i = 0;
	while (bytes >= 1000 && i < units.length - 1) {
		bytes /= 1000;
		i++;
	}
	return `${bytes.toFixed(i && bytes < 10 ? 1 : 0)} ${units[i]}`;
}
