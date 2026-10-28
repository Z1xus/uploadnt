<script lang="ts">
	import { seedOf } from '#lib/marker.ts';
	import type { Receiving } from '../receiving.svelte';
	import { size } from '../rate';
	import { grievances } from '../grievance';
	import Digit from '../ui/Digit.svelte';
	import Done from '../ui/Done.svelte';
	import Path from '../ui/Path.svelte';
	import Stamp from '../ui/Stamp.svelte';

	let { r, fragment }: { r: Receiving; fragment: string } = $props();

	const seed = $derived(seedOf(fragment));
</script>

<main class="relative mx-auto max-w-[58ch] px-6 pt-[10vh]">
	{#if r.phase === 'connecting'}
		<p class="text-2xl text-graphite">
			{r.alone ? 'waiting for the sender. their tab needs to be open.' : 'connecting…'}
		</p>
	{:else if r.phase === 'ask'}
		<p class="text-3xl">can't connect directly</p>
		<p class="mt-3 text-xl text-graphite">this will transfer encrypted bytes over our relay.</p>
		<div class="mt-12"><Stamp onclick={() => r.planb()}>use plan b</Stamp></div>
	{:else if r.phase === 'offered'}
		<ul class="space-y-1 text-3xl">
			{#each r.files as f}<li class="break-all">{f.name}</li>{/each}
		</ul>
		<p class="mt-3 text-xl text-graphite">{size(r.size)}</p>
		<div class="mt-12"><Stamp onclick={() => r.take()}>take it</Stamp></div>
	{:else if r.phase === 'receiving'}
		<Digit value={r.bytes / Math.max(r.size, 1)} />
		<p class="mt-6 flex flex-wrap items-baseline gap-x-6 text-xl">
			{#if r.rate}<span>{size(r.rate)}/s</span>{/if}<Path path={r.path} />
		</p>
		<p class="mt-3 text-graphite">{r.where}</p>
	{:else if r.phase === 'done'}
		<Done names={r.files.map((f) => f.name)} />
		<p class="mt-6 text-xl">received.</p>
		<p class="mt-3 text-graphite">keep this tab open to get more files.</p>
	{:else}
		<p class="text-3xl text-marker">{grievances[r.grievance ?? 'broken']}</p>
		{#if r.grievance !== 'link' && r.grievance !== 'full'}
			<div class="mt-10"><Stamp onclick={() => location.reload()}>try again</Stamp></div>
		{/if}
	{/if}
</main>
