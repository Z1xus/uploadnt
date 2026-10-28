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
	import { tilted } from '../ui/tilt';

	let { s }: { s: Sending } = $props();

	const seed = $derived(seedOf(s.link));
	let copied = $state(false);

	async function copy() {
		await navigator.clipboard.writeText(s.link);
		copied = true;
	}
</script>

<main class="relative mx-auto max-w-[58ch] px-6 pt-[10vh]">
	{#if s.phase === 'waiting'}
		<button
			class="relative block w-full cursor-copy text-left text-2xl break-all"
			onclick={copy}
			use:tilted={seed}
		>
			{s.link}
			{#if copied}<Mark kind="underline" seed={seed + 1} class="inset-x-0 -bottom-2 h-2 w-full" />{/if}
		</button>
		<div class="mt-10 flex flex-wrap items-end gap-10">
			<div use:tilted={seed + 2}><Qr text={s.link} /></div>
			<ul class="space-y-1 text-lg" use:tilted={seed + 3}>
				{#each s.files as f}<li class="break-all normal-case">
						{f.webkitRelativePath || f.name} <span class="text-graphite">{size(f.size)}</span>
					</li>{/each}
			</ul>
		</div>
		<p class="relative mt-14 inline-block text-3xl" use:tilted={seed + 4}>
			keep this tab open
			<Mark kind="underline" seed={seed + 5} delay={300} class="inset-x-0 -bottom-2 h-2 w-full" />
		</p>
		{#if s.hashed < s.total}
			<Margin><p>reading {Math.floor((100 * s.hashed) / Math.max(s.total, 1))}%</p></Margin>
		{/if}
	{:else if s.phase === 'sending'}
		<Digit value={s.bytes / Math.max(s.total, 1)} />
		<p class="mt-6 text-xl">not uploading{s.rate ? ` · ${size(s.rate)}/s` : ''}</p>
		<p class="mt-3 text-xl"><Path path={s.path} /></p>
	{:else if s.phase === 'done'}
		<Done names={s.files.map((f) => f.webkitRelativePath || f.name)} />
	{:else}
		<p class="text-3xl text-marker" use:tilted={seed}>{grievances[s.grievance ?? 'broken']}</p>
		<div class="mt-10"><Stamp onclick={() => location.reload()}>start over</Stamp></div>
	{/if}
</main>
