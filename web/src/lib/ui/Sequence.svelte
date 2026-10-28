<script lang="ts">
	import { SEQ_ROW, sequence, type Step } from '#lib/doodle.ts';
	import Strokes from './Strokes.svelte';

	let { lanes, steps, seed }: { lanes: string[]; steps: Step[]; seed: number } = $props();

	const seq = $derived(sequence(lanes.length, steps, seed));
</script>

<svg
	class="my-2 block w-full max-w-md overflow-visible select-none"
	viewBox="0 0 320 {seq.height}"
	role="img"
	aria-label={steps.map((s) => `${lanes[s.from]} to ${lanes[s.to]}: ${s.label}`).join('. ')}
>
	<Strokes strokes={seq.strokes} width={2} step={30} />
	<g class="fill-foreground text-outline font-serif text-[13px]">
		{#each lanes as lane, i}
			<text x={seq.x(i)} y="16" text-anchor="middle">{lane}</text>
		{/each}
		{#each steps as s, k}
			{@const y = 56 + k * SEQ_ROW}
			{#if s.from === s.to}
				<text
					x={seq.x(s.from) + (s.from === lanes.length - 1 ? -24 : 24)}
					y={y + 2}
					text-anchor={s.from === lanes.length - 1 ? 'end' : 'start'}
					class={s.tone === 'muted' ? 'fill-muted' : ''}>{s.label}</text
				>
			{:else}
				<text
					x={(seq.x(s.from) + seq.x(s.to)) / 2}
					y={y - 11}
					text-anchor="middle"
					class={s.tone === 'accent' ? 'fill-accent' : s.tone === 'muted' ? 'fill-muted' : ''}>{s.label}</text
				>
			{/if}
		{/each}
	</g>
</svg>
