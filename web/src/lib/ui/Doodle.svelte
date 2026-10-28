<script lang="ts">
	import { doodle } from '#lib/doodle.ts';

	let { kind, seed, label }: { kind: 'corp' | 'direct' | 'planb'; seed: number; label: string } = $props();

	const strokes = $derived(doodle(kind, seed));
</script>

<svg
	class="scrawl block w-full max-w-md overflow-visible"
	viewBox={kind === 'planb' ? '0 60 320 115' : '0 0 320 175'}
	role="img"
	aria-label={label}
>
	{#each strokes as s, i}
		<path
			d={s.d}
			pathLength="1"
			stroke-width="2.5"
			style:stroke={s.tone === 'red' ? '' : 'var(--ink)'}
			style:--delay="{i * 45}ms"
		/>
	{/each}
	<g class="halo fill-ink font-fine text-[15px]">
		<text x="40" y="170" text-anchor="middle">you</text>
		<text x="280" y="170" text-anchor="middle">friend</text>
		{#if kind === 'corp'}
			<text x="160" y="8" text-anchor="middle">big corp</text>
			<text x="205" y="22" class="fill-marker text-[20px]">$$$</text>
		{:else if kind === 'planb'}
			<text x="160" y="140" text-anchor="middle">plan b</text>
		{/if}
	</g>
</svg>
