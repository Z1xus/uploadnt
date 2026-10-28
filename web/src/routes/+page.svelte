<script lang="ts">
	import { loadCore } from '#lib/core.ts';
	import { Receiving } from '#lib/receiving.svelte.ts';
	import { Sending } from '#lib/sending.svelte.ts';
	import Landing from '#lib/screens/Landing.svelte';
	import Receive from '#lib/screens/Receive.svelte';
	import Send from '#lib/screens/Send.svelte';

	const seed = Math.floor(Math.random() * 2 ** 31);
	const fragment = location.hash.slice(1);

	let sending = $state<Sending>();
	let receiving = $state<Receiving>();
	const ready = loadCore().then(() => fragment && (receiving = new Receiving(fragment)));
</script>

<svelte:window onhashchange={() => location.reload()} />

{#await ready then}
	{#if receiving}
		<Receive r={receiving} {fragment} />
	{:else if sending}
		<Send s={sending} />
	{:else}
		<Landing {seed} onfiles={(files: File[]) => (sending = new Sending(files))} />
	{/if}
{/await}
