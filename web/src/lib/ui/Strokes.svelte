<script lang="ts">
	import type { Stroke } from '#lib/doodle.ts';

	let {
		strokes,
		width,
		step,
		delay = 0
	}: { strokes: Stroke[]; width: number; step: number; delay?: number } = $props();

	const tones = { foreground: 'stroke-foreground', accent: 'stroke-accent', muted: 'stroke-muted' };
</script>

{#each strokes as s, i}
	<path
		d={s.d}
		class="draw-in {tones[s.tone]}"
		fill="none"
		pathLength="1"
		stroke-width={width}
		stroke-linecap="round"
		stroke-linejoin="round"
		style:--delay="{delay + i * step}ms"
	/>
{/each}
