<script lang="ts">
	import type { Path } from '../peer';

	let { path }: { path: Path } = $props();
</script>

{#if path === 'direct'}
	<span>direct</span>
{:else}
	<span class="stamp inline-block border-[3px] border-marker px-3 py-0.5 text-marker">plan b</span>
	{#if path === 'tcp'}<span class="ml-3 text-base text-graphite">this network blocks udp.</span>{/if}
{/if}
