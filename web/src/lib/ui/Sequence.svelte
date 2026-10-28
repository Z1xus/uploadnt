<script lang="ts">
	import { SEQ_ROW, sequence, type Step } from '#lib/doodle.ts';

	let { lanes, steps, seed }: { lanes: string[]; steps: Step[]; seed: number } = $props();

	const seq = $derived(sequence(lanes.length, steps, seed));
	const tones = { ink: 'var(--ink)', red: '', faint: 'var(--graphite)' };
</script>

<svg
	class="scrawl my-2 block w-full max-w-md overflow-visible"
	viewBox="0 0 320 {seq.height}"
	role="img"
	aria-label={steps.map((s) => `${lanes[s.from]} to ${lanes[s.to]}: ${s.label}`).join('. ')}
>
	{#each seq.strokes as s, i}
		<path d={s.d} pathLength="1" stroke-width="2" style:stroke={tones[s.tone]} style:--delay="{i * 30}ms" />
	{/each}
	<g class="halo fill-ink font-fine text-[13px]">
		{#each lanes as lane, i}
			<text x={seq.x(i)} y="16" text-anchor="middle" class="text-[13px]">{lane}</text>
		{/each}
		{#each steps as s, k}
			{@const y = 56 + k * SEQ_ROW}
			{#if s.from === s.to}
				<text
					x={seq.x(s.from) + (s.from === lanes.length - 1 ? -24 : 24)}
					y={y + 2}
					text-anchor={s.from === lanes.length - 1 ? 'end' : 'start'}
					class={s.tone === 'faint' ? 'fill-graphite' : ''}>{s.label}</text
				>
			{:else}
				<text
					x={(seq.x(s.from) + seq.x(s.to)) / 2}
					y={y - 11}
					text-anchor="middle"
					class={s.tone === 'red' ? 'fill-marker' : s.tone === 'faint' ? 'fill-graphite' : ''}>{s.label}</text
				>
			{/if}
		{/each}
	</g>
</svg>
