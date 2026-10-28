<script lang="ts">
	let { value }: { value: number } = $props();

	const pct = $derived(Math.min(99, Math.floor(value * 100)));
</script>

<div
	class="-ml-[0.06em] text-[min(50dvh,46vw,18rem)] leading-[0.8] whitespace-nowrap tabular-nums blur-(--bleed)"
	aria-live="polite"
>
	<span class="inline-block min-w-[2ch]">{pct}</span><span
		class="ml-[0.04em] align-top text-[0.28em] leading-none">%</span
	>
</div>
