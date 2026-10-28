<script lang="ts">
	import { pen } from '#lib/doodle.ts';

	let { seed, up = false, class: cls = '' }: { seed: number; up?: boolean; class?: string } = $props();

	const strokes = $derived.by(() => {
		const p = pen(seed);
		if (up)
			p.arrow(
				[
					[44, 46],
					[22, 34],
					[12, 6]
				],
				'red',
				2
			);
		else
			p.arrow(
				[
					[56, 30],
					[30, 10],
					[4, 18]
				],
				'red',
				2
			);
		return p.strokes;
	});
</script>

<svg class="scrawl overflow-visible {cls}" viewBox={up ? '0 0 50 50' : '0 0 60 36'} aria-hidden="true">
	{#each strokes as s, i}
		<path d={s.d} pathLength="1" stroke-width="2.5" style:--delay="{600 + i * 120}ms" />
	{/each}
</svg>
