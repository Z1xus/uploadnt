<script lang="ts">
	import type { Snippet } from 'svelte';
	import { strike, underline } from '#lib/marker.ts';

	let {
		kind,
		seed,
		delay = 0,
		show = true,
		children
	}: {
		kind: 'strike' | 'underline';
		seed: number;
		delay?: number;
		show?: boolean;
		children: Snippet;
	} = $props();
</script>

<span class="relative inline-block"
	>{@render children()}{#if show}<svg
			class="pointer-events-none absolute inset-x-0 w-full wipe-in overflow-visible {kind === 'strike'
				? 'top-[22%] h-[56%]'
				: '-bottom-2 h-2'}"
			viewBox={kind === 'strike' ? '0 0 100 20' : '0 0 100 10'}
			preserveAspectRatio="none"
			aria-hidden="true"
			style:--delay="{delay}ms"
			><path
				d={kind === 'strike' ? strike(seed) : underline(seed)}
				class="stroke-accent"
				fill="none"
				stroke-width="3"
				stroke-linecap="round"
				vector-effect="non-scaling-stroke"
			/></svg
		>{/if}</span
>
