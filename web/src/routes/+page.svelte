<script lang="ts">
	import { loadCore } from '#lib/core.ts';
	import { Receiving } from '#lib/receiving.svelte.ts';
	import { Sending, pending, type Pending } from '#lib/sending.svelte.ts';
	import Landing from '#lib/screens/Landing.svelte';
	import Receive from '#lib/screens/Receive.svelte';
	import Send from '#lib/screens/Send.svelte';
	import Debug from '#lib/ui/Debug.svelte';
	import { beforeNavigate } from '$app/navigation';
	import { onDestroy } from 'svelte';

	const seed = Math.floor(Math.random() * 2 ** 31);
	const fragment = location.hash.slice(1);
	const debugging = new URLSearchParams(location.search).has('debug');

	let sending = $state<Sending>();
	let receiving = $state<Receiving>();
	let previous = $state<Pending>();
	const busy = $derived(sending?.phase === 'sending' || receiving?.phase === 'receiving');

	beforeNavigate(({ cancel }) => {
		if (busy && !confirm('leave this page? the transfer stops.')) cancel();
	});
	onDestroy(() => {
		sending?.close();
		receiving?.close();
	});

	const ready = fragment
		? loadCore().then(() => (receiving = new Receiving(fragment)))
		: (loadCore(), pending().then((p) => (previous = p)));
</script>

<svelte:window onhashchange={() => location.reload()} />

{#await ready}
	{#if fragment}<p class="text-2xl text-muted">connecting…</p>{/if}
{:then}
	{#if receiving}
		<Receive r={receiving} {fragment} />
	{:else if sending}
		<Send
			s={sending}
			onback={async () => {
				sending?.cancel();
				sending = undefined;
				previous = await pending();
			}}
			onmore={(files: File[]) => {
				const secret = sending!.secret;
				sending!.cancel();
				sending = new Sending(files, undefined, secret);
			}}
		/>
	{:else}
		<Landing
			{seed}
			{previous}
			onfiles={(files: File[]) => loadCore().then(() => (sending = new Sending(files, previous)))}
		/>
	{/if}
	{#if debugging && (sending || receiving)}
		<Debug peer={() => (sending ?? receiving)?.peer} />
	{/if}
{/await}
