<script lang="ts">
	import { scrawl } from '#lib/marker.ts';
	import Mark from './Mark.svelte';
	import Strokes from './Strokes.svelte';

	let { seed }: { seed: number } = $props();
</script>

<h1 class="text-[clamp(4rem,16vw,8.5rem)] leading-[0.9] blur-(--bleed)">
	<Mark kind="strike" seed={seed + 1} delay={200}>upload</Mark><svg
		class="-ml-[0.04em] inline-block h-[0.72em] w-[0.78em] overflow-visible align-[-0.04em]"
		viewBox="0 0 66 62"
		role="img"
		aria-label="n't"
	>
		<Strokes
			strokes={scrawl(seed + 2).map((d) => ({ d, tone: 'accent' }))}
			width={5}
			step={160}
			delay={520}
		/>
	</svg>
</h1>
