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
	}: { value: number; rate: number; path: Path; label?: Snippet; children?: Snippet } = $props();
</script>

<Percent {value} />
<p class="mt-6 flex flex-wrap items-center gap-x-6 gap-y-3 text-xl">
	{#if label}<span>{@render label()}</span>{/if}
	<span class="min-w-[9ch] tabular-nums">{rate ? `${size(rate)}/s` : ''}</span>
	{#if path === 'direct'}
		<span>direct</span>
	{:else if path === 'relayed'}
		<span class="badge">plan b</span>
	{/if}
</p>
{#if path === 'tcp'}
	<p class="mt-3 text-muted">
		this network blocks udp, so we're using <span class="text-accent">plan b</span> instead.
	</p>
{:else if path === 'blocked'}
	<p class="mt-3 text-muted">
		your browser blocks direct connections, so we're using <span class="text-accent">plan b</span> instead. allow
		webrtc in its settings to make it faster.
	</p>
{/if}
{@render children?.()}
