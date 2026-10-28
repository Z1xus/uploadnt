<script lang="ts">
	import { scrawl } from '#lib/marker.ts';
	import Strokes from './Strokes.svelte';

	let { seed, delay = 0, class: cls = '' }: { seed: number; delay?: number; class?: string } = $props();
</script>

<svg class="scrawl inline-block overflow-visible {cls}" viewBox="0 0 66 62" aria-label="n't" role="img">
	<Strokes strokes={scrawl(seed).map((d) => ({ d, tone: 'red' }))} width={5} step={160} {delay} />
</svg>
