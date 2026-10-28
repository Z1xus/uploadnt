<script lang="ts">
	import { seedOf } from '#lib/marker.ts';
	import type { Sending } from '../sending.svelte';
	import { size } from '../rate';
	import { grievances } from '../grievance';
	import Digit from '../ui/Digit.svelte';
	import Done from '../ui/Done.svelte';
	import Margin from '../ui/Margin.svelte';
	import Mark from '../ui/Mark.svelte';
	import Path from '../ui/Path.svelte';
	import Qr from '../ui/Qr.svelte';
	import Stamp from '../ui/Stamp.svelte';

	let { s, onback }: { s: Sending; onback: () => void } = $props();

	const back = 'mt-10 block cursor-pointer text-graphite underline-offset-4 hover:underline';

	const seed = $derived(seedOf(s.link));
	let copied = $state(false);

	async function copy() {
		await navigator.clipboard.writeText(s.link);
		copied = true;
	}
</script>

<main class="relative mx-auto max-w-[58ch] px-6 pt-[10vh]">
	{#if s.phase === 'waiting'}
		<button class="relative block w-full cursor-copy text-left text-2xl break-all" onclick={copy}>
			{s.link}
			{#if copied}<Mark kind="underline" seed={seed + 1} class="inset-x-0 -bottom-2 h-2 w-full" />{/if}
		</button>
		<div class="mt-10 flex flex-wrap items-end gap-10">
			<div><Qr text={s.link} /></div>
			<ul class="space-y-1 text-lg">
				{#each s.files as f}<li class="break-all">
						{f.webkitRelativePath || f.name} <span class="text-graphite">{size(f.size)}</span>
					</li>{/each}
			</ul>
		</div>
		<p class="relative mt-14 inline-block text-3xl">
			keep this tab open
			<Mark kind="underline" seed={seed + 5} delay={300} class="inset-x-0 -bottom-2 h-2 w-full" />
		</p>
		<button class={back} onclick={onback}>cancel</button>
		{#if s.hashed < s.total}
			<Margin><p>preparing… {Math.floor((100 * s.hashed) / Math.max(s.total, 1))}%</p></Margin>
		{/if}
	{:else if s.phase === 'sending'}
		<Digit value={s.bytes / Math.max(s.total, 1)} />
		<p class="mt-6 text-xl">not uploading{s.rate ? ` · ${size(s.rate)}/s` : ''}</p>
		<p class="mt-3 text-xl"><Path path={s.path} /></p>
		<button class={back} onclick={onback}>cancel</button>
	{:else if s.phase === 'done'}
		<Done names={s.files.map((f) => f.webkitRelativePath || f.name)} />
		<p class="mt-6 text-xl">sent.</p>
		<div class="mt-10"><Stamp onclick={onback}>send another</Stamp></div>
	{:else}
		<p class="text-3xl text-marker">{grievances[s.grievance ?? 'broken']}</p>
		<div class="mt-10"><Stamp onclick={() => location.reload()}>start over</Stamp></div>
	{/if}
</main>
