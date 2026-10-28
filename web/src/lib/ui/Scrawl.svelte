<script lang="ts">
	import { scrawl } from '#lib/marker.ts';

	let { seed, delay = 0, class: cls = '' }: { seed: number; delay?: number; class?: string } = $props();
</script>

<svg class="scrawl inline-block overflow-visible {cls}" viewBox="0 0 66 62" aria-label="n't" role="img">
	{#each scrawl(seed) as d, i}
		<path {d} pathLength="1" stroke-width="5" style:--delay="{delay + i * 160}ms" />
	{/each}
</svg>
