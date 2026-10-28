<script lang="ts">
	import { random } from '#lib/marker.ts';

	let { value }: { value: number } = $props();

	const pct = $derived(Math.min(99, Math.floor(value * 100)));
	const style = $derived.by(() => {
		const rand = random(pct * 7919 + 1);
		return `translate: ${(rand() - 0.5) * 12}px ${(rand() - 0.5) * 8}px`;
	});
</script>

<div class="bleed -ml-[0.06em] text-[min(55vh,48vw)] leading-[0.8] tabular-nums" {style} aria-live="polite">
	{pct}
</div>
