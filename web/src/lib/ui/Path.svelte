<script lang="ts">
	let { relayed }: { relayed: boolean } = $props();
</script>

{#if relayed}
	<span class="stamp inline-block border-[3px] border-marker px-3 py-0.5 text-marker" style:--rot="2deg"
		>plan b</span
	>
{:else}
	<span>direct</span>
{/if}
