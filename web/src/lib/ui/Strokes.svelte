<script lang="ts">
	import type { Stroke } from '#lib/doodle.ts';

	let {
		strokes,
		width,
		step,
		delay = 0
	}: { strokes: Stroke[]; width: number; step: number; delay?: number } = $props();

	const tones = { ink: 'var(--ink)', red: undefined, faint: 'var(--graphite)' };
</script>

{#each strokes as s, i}
	<path
		d={s.d}
		pathLength="1"
		stroke-width={width}
		style:stroke={tones[s.tone]}
		style:--delay="{delay + i * step}ms"
	/>
{/each}
