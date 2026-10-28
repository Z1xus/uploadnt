<script lang="ts">
	import type { Snippet } from 'svelte';
	import Mark from './Mark.svelte';

	let { seed, delay = 0, children }: { seed: number; delay?: number; children: Snippet } = $props();
</script>

<span class="relative inline-block"
	>{@render children()}<Mark {seed} {delay} class="inset-x-0 top-[22%] h-[56%] w-full" /></span
>
