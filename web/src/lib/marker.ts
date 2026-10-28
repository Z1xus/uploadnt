type Point = [number, number];

export function random(seed: number) {
	return () => {
		seed = (seed + 0x6d2b79f5) | 0;
		let t = Math.imul(seed ^ (seed >>> 15), 1 | seed);
		t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
		return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
	};
}

export function seedOf(text: string) {
	let h = 2166136261;
	for (const c of text) h = Math.imul(h ^ c.charCodeAt(0), 16777619);
	return h >>> 0;
}

function smooth(points: Point[]) {
	const f = (n: number) => n.toFixed(1);
	let d = `M${f(points[0][0])} ${f(points[0][1])}`;
	for (let i = 0; i < points.length - 1; i++) {
		const [p0, p1, p2, p3] = [
			points[i - 1] ?? points[i],
			points[i],
			points[i + 1],
			points[i + 2] ?? points[i + 1]
		];
		const c1: Point = [p1[0] + (p2[0] - p0[0]) / 6, p1[1] + (p2[1] - p0[1]) / 6];
		const c2: Point = [p2[0] - (p3[0] - p1[0]) / 6, p2[1] - (p3[1] - p1[1]) / 6];
		d += ` C${f(c1[0])} ${f(c1[1])} ${f(c2[0])} ${f(c2[1])} ${f(p2[0])} ${f(p2[1])}`;
	}
	return d;
}

function jitter(points: Point[], rand: () => number, amount: number): Point[] {
	return points.map(([x, y]) => [x + (rand() - 0.5) * amount, y + (rand() - 0.5) * amount]);
}

export function strike(seed: number, width = 100, height = 20) {
	const rand = random(seed);
	const slope = (rand() - 0.6) * height * 0.5;
	const steps = 6;
	const points: Point[] = [];
	for (let i = 0; i <= steps; i++) {
		const x = -width * 0.04 + (width * 1.08 * i) / steps;
		points.push([x, height * 0.55 + slope * (i / steps - 0.5) + (rand() - 0.5) * height * 0.18]);
	}
	return smooth(points);
}

export function underline(seed: number, width = 100, height = 10) {
	const rand = random(seed);
	const points: Point[] = [];
	for (let i = 0; i <= 5; i++) points.push([(width * i) / 5, height * 0.5 + (rand() - 0.5) * height * 0.5]);
	return smooth(points);
}

const NT: Point[][] = [
	[
		[3, 58],
		[5, 36],
		[5, 22],
		[9, 18],
		[17, 16],
		[23, 22],
		[24, 40],
		[26, 58]
	],
	[
		[38, 8],
		[36, 16],
		[33, 25]
	],
	[
		[52, 6],
		[51, 30],
		[49, 50],
		[53, 58],
		[62, 55]
	],
	[
		[40, 25],
		[51, 23],
		[64, 21]
	]
];

export function scrawl(seed: number) {
	const rand = random(seed);
	return NT.map((stroke) => smooth(jitter(stroke, rand, 3.2)));
}
