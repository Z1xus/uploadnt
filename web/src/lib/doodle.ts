import { jitter, random, smooth, type Point } from './marker';

export type Tone = 'ink' | 'red' | 'faint';
export type Stroke = { d: string; tone: Tone };
export type Step = { from: number; to: number; label: string; tone?: Tone };

export const SEQ_ROW = 34;

export function pen(seed: number) {
	const rand = random(seed);
	const strokes: Stroke[] = [];
	const add = (points: Point[], tone: Tone = 'ink', wobble = 2.5) =>
		strokes.push({ d: smooth(jitter(points, rand, wobble)), tone });

	const line = (a: Point, b: Point, tone: Tone = 'ink') =>
		add(
			Array.from({ length: 5 }, (_, i) => [a[0] + ((b[0] - a[0]) * i) / 4, a[1] + ((b[1] - a[1]) * i) / 4]),
			tone
		);

	const ring = (cx: number, cy: number, r: number, bumps = false) =>
		add(
			Array.from({ length: 15 }, (_, i) => {
				const a = (i / 13) * Math.PI * 2;
				const k = bumps && i % 2 ? 1.25 : 1;
				return [cx + Math.cos(a) * r * k * 1.3, cy + Math.sin(a) * r * k];
			}),
			'ink',
			bumps ? 4 : 2
		);

	const arrow = (points: Point[], tone: Tone = 'ink', wobble = 6) => {
		add(points, tone, wobble);
		const [[x0, y0], [x1, y1]] = points.slice(-2);
		const a = Math.atan2(y1 - y0, x1 - x0);
		for (const s of [-1, 1])
			line([x1, y1], [x1 - Math.cos(a + s * 0.5) * 10, y1 - Math.sin(a + s * 0.5) * 10], tone);
	};

	const person = (x: number, happy: boolean) => {
		ring(x, 86, 9);
		line([x - 4, 84], [x - 3.5, 85]);
		line([x + 4, 84], [x + 3.5, 85]);
		add(
			happy
				? [
						[x - 4, 89],
						[x, 92],
						[x + 4, 89]
					]
				: [
						[x - 4, 92],
						[x, 89],
						[x + 4, 92]
					],
			'ink',
			0.8
		);
		line([x, 98], [x, 128]);
		line([x - 15, 106], [x + 15, 108]);
		line([x, 128], [x - 11, 150]);
		line([x, 128], [x + 12, 150]);
	};

	const paper = (x: number, y: number) => {
		add(
			[
				[x, y],
				[x + 16, y],
				[x + 22, y + 6],
				[x + 22, y + 26],
				[x, y + 26],
				[x, y]
			],
			'ink',
			1.5
		);
		line([x + 4, y + 11], [x + 16, y + 11]);
		line([x + 4, y + 18], [x + 16, y + 18]);
	};

	return { strokes, add, line, ring, arrow, person, paper };
}

export function doodle(kind: 'corp' | 'direct' | 'planb', seed: number) {
	const p = pen(seed);
	const happy = kind !== 'corp';
	p.person(40, happy);
	p.person(280, happy);
	if (kind === 'corp') {
		p.ring(160, 48, 30, true);
		p.line([145, 38], [154, 43]);
		p.line([175, 38], [166, 43]);
		p.paper(149, 52);
		p.arrow([
			[58, 102],
			[85, 60],
			[118, 50]
		]);
		p.arrow([
			[202, 50],
			[240, 62],
			[262, 102]
		]);
	} else if (kind === 'direct') {
		p.arrow(
			[
				[58, 106],
				[160, 96],
				[262, 106]
			],
			'red'
		);
		p.paper(148, 66);
		p.ring(296, 22, 9);
		for (let i = 0; i < 6; i++) {
			const a = (i / 6) * Math.PI * 2;
			p.line(
				[296 + Math.cos(a) * 16, 22 + Math.sin(a) * 14],
				[296 + Math.cos(a) * 24, 22 + Math.sin(a) * 20]
			);
		}
	} else {
		p.arrow(
			[
				[58, 106],
				[100, 102],
				[136, 100]
			],
			'red'
		);
		p.arrow(
			[
				[184, 100],
				[224, 102],
				[262, 106]
			],
			'red'
		);
		p.line([138, 80], [182, 80]);
		p.line([182, 80], [182, 120]);
		p.line([182, 120], [138, 120]);
		p.line([138, 120], [138, 80]);
		p.line([146, 92], [154, 98]);
		p.line([154, 92], [146, 98]);
		p.line([166, 92], [174, 98]);
		p.line([174, 92], [166, 98]);
		p.line([150, 110], [170, 110]);
	}
	return p.strokes;
}

export function sequence(lanes: number, steps: Step[], seed: number) {
	const p = pen(seed);
	const x = (i: number) => 40 + (i * 240) / Math.max(lanes - 1, 1);
	const bottom = 46 + steps.length * SEQ_ROW;
	for (let i = 0; i < lanes; i++)
		p.add(
			[
				[x(i), 26],
				[x(i), (26 + bottom) / 2],
				[x(i), bottom]
			],
			'faint',
			2
		);
	steps.forEach((s, k) => {
		const y = 56 + k * SEQ_ROW;
		if (s.from === s.to) {
			const dx = s.from === lanes - 1 ? -1 : 1;
			p.arrow(
				[
					[x(s.from), y - 8],
					[x(s.from) + dx * 18, y - 4],
					[x(s.from) + dx * 2, y + 6]
				],
				s.tone,
				1.5
			);
		} else
			p.arrow(
				[
					[x(s.from), y],
					[(x(s.from) + x(s.to)) / 2, y + 1],
					[x(s.to), y]
				],
				s.tone,
				2
			);
	});
	return { strokes: p.strokes, x, height: bottom + 8 };
}
