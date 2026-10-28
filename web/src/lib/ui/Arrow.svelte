<script lang="ts">
	import { pen, type Point } from '#lib/doodle.ts';
	import Strokes from './Strokes.svelte';

	let { seed, up = false, class: cls = '' }: { seed: number; up?: boolean; class?: string } = $props();

	const points: Point[] = $derived(
		up
			? [
					[44, 46],
					[22, 34],
					[12, 6]
				]
			: [
					[56, 30],
					[30, 10],
					[4, 18]
				]
	);
	const strokes = $derived.by(() => {
		const p = pen(seed);
		p.arrow(points, 'accent', 2);
		return p.strokes;
	});
</script>

<svg class="overflow-visible {cls}" viewBox={up ? '0 0 50 50' : '0 0 60 36'} aria-hidden="true">
	<Strokes {strokes} width={2.5} step={120} delay={600} />
</svg>
