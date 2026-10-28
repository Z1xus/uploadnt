<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { Path } from '../peer';
	import { size } from '../rate';
	import Percent from './Percent.svelte';

	let {
		value,
		rate,
		path,
		label,
		children
	}: { value: number; rate: number; path: Path; label?: string; children?: Snippet } = $props();
</script>

<Percent {value} />
<p class="mt-6 flex flex-wrap items-center gap-x-6 gap-y-3 text-xl">
	{#if label}<span>{label}</span>{/if}
	{#if rate}<span>{size(rate)}/s</span>{/if}
	{#if path === 'direct'}
		<span>direct</span>
	{:else}
		<span class="badge">plan b</span>
	{/if}
</p>
{#if path === 'tcp'}<p class="mt-3 text-muted">this network blocks udp.</p>{/if}
{@render children?.()}
