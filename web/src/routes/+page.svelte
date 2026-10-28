<script lang="ts">
	import { loadCore } from '#lib/core.ts';
	import { Receiving } from '#lib/receiving.svelte.ts';
	import { Sending, pending, type Pending } from '#lib/sending.svelte.ts';
	import Landing from '#lib/screens/Landing.svelte';
	import Receive from '#lib/screens/Receive.svelte';
	import Send from '#lib/screens/Send.svelte';

	const seed = Math.floor(Math.random() * 2 ** 31);
	const fragment = location.hash.slice(1);

	let sending = $state<Sending>();
	let receiving = $state<Receiving>();
	let previous = $state<Pending>();
	const ready = loadCore().then(async () => {
		if (fragment) receiving = new Receiving(fragment);
		else previous = await pending();
	});
</script>

<svelte:window onhashchange={() => location.reload()} />

{#await ready then}
	{#if receiving}
		<Receive r={receiving} {fragment} />
	{:else if sending}
		<Send s={sending} />
	{:else}
		<Landing {seed} {previous} onfiles={(files: File[]) => (sending = new Sending(files, previous))} />
	{/if}
{/await}
