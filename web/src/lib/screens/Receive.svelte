<script lang="ts">
	import { seedOf } from '#lib/marker.ts';
	import type { Receiving } from '../receiving.svelte';
	import { size } from '../rate';
	import Button from '../ui/Button.svelte';
	import Failure from '../ui/Failure.svelte';
	import Mark from '../ui/Mark.svelte';
	import Progress from '../ui/Progress.svelte';

	let { r, fragment }: { r: Receiving; fragment: string } = $props();

	const seed = $derived(seedOf(fragment));
</script>

{#if r.phase === 'connecting'}
	<p class="text-2xl text-muted">
		{r.alone ? 'waiting for the sender. their tab needs to be open.' : 'connecting…'}
	</p>
{:else if r.phase === 'ask'}
	<p class="text-3xl">can't connect directly</p>
	<p class="mt-3 text-xl text-muted">this will transfer encrypted bytes over our relay.</p>
	{#if r.blocked}
		<p class="mt-3 text-muted">
			your browser blocks direct connections. allowing webrtc in its settings fixes this.
		</p>
	{/if}
	<div class="mt-10"><Button onclick={() => r.planb()}>use plan b</Button></div>
{:else if r.phase === 'offered'}
	<ul class="space-y-1 text-3xl">
		{#each r.files as f}<li class="break-all">{f.name}</li>{/each}
	</ul>
	<p class="mt-3 text-xl text-muted">{size(r.size)}</p>
	<div class="mt-10"><Button onclick={() => r.take()}>take it</Button></div>
{:else if r.phase === 'receiving'}
	<Progress value={r.bytes / Math.max(r.size, 1)} rate={r.rate} path={r.path}>
		<p class="mt-3 text-muted">{r.where}</p>
	</Progress>
{:else if r.phase === 'done'}
	<p class="text-4xl break-words">
		<Mark kind="underline" {seed}>{r.files.map((f) => f.name).join(', ')}</Mark>
	</p>
	<p class="mt-6 text-xl">received.</p>
	<p class="mt-3 text-muted">keep this tab open to get more files.</p>
	{#if r.again}
		<button class="mt-10 block text-muted hover:underline" onclick={() => r.redownload()}
			>download again</button
		>
	{/if}
{:else}
	<Failure grievance={r.grievance} error={r.error} />
	{#if r.grievance !== 'link' && r.grievance !== 'full'}
		<div class="mt-10"><Button onclick={() => location.reload()}>try again</Button></div>
	{/if}
{/if}
