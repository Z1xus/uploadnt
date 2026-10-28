<script lang="ts">
	import { seedOf } from '#lib/marker.ts';
	import type { Sending } from '../sending.svelte';
	import { percent, size } from '../rate';
	import { grievances } from '../grievance';
	import Done from '../ui/Done.svelte';
	import Margin from '../ui/Margin.svelte';
	import Mark from '../ui/Mark.svelte';
	import Pointer from '../ui/Pointer.svelte';
	import Progress from '../ui/Progress.svelte';
	import Qr from '../ui/Qr.svelte';
	import Stamp from '../ui/Stamp.svelte';

	let { s, onback, onmore }: { s: Sending; onback: () => void; onmore: (files: File[]) => void } = $props();

	const quiet = 'cursor-pointer text-graphite underline-offset-4 hover:underline';

	const seed = $derived(seedOf(s.link));
	let copied = $state(false);
	let input = $state<HTMLInputElement>();

	async function copy() {
		await navigator.clipboard.writeText(s.link);
		copied = true;
	}
</script>

{#snippet list()}
	<ul class="space-y-1 text-lg">
		{#each s.files as f}<li class="break-all">
				{f.webkitRelativePath || f.name} <span class="text-graphite">{size(f.size)}</span>
			</li>{/each}
	</ul>
{/snippet}

<main class="relative mx-auto max-w-[36rem] px-6 pt-[10vh]">
	{#if s.phase === 'waiting' && s.more}
		<p class="text-3xl">waiting for them to take it.</p>
		<div class="mt-8">{@render list()}</div>
		<button class="mt-10 block {quiet}" onclick={onback}>cancel</button>
	{:else if s.phase === 'waiting'}
		<div class="relative">
			<button class="relative block w-full cursor-copy text-left text-2xl break-all" onclick={copy}>
				{s.link}
				{#if copied}<Mark kind="underline" seed={seed + 1} class="inset-x-0 -bottom-2 h-2 w-full" />{/if}
			</button>
			<p
				class="mt-3 flex items-end gap-1 font-fine text-lg text-marker lg:absolute lg:top-0 lg:left-[calc(100%+0.5rem)] lg:mt-0 lg:w-48 lg:items-start"
			>
				<Pointer seed={seed + 7} up class="inline-block h-10 w-10 lg:hidden" />
				<Pointer seed={seed + 7} class="hidden h-8 w-14 shrink-0 lg:inline-block" />
				<span class="lg:mt-5"
					>{#if copied}copied{:else}<span class="pointer-coarse:hidden">click to copy</span><span
							class="hidden pointer-coarse:inline">tap to copy</span
						>{/if}</span
				>
			</p>
		</div>
		<div class="mt-10 flex flex-wrap items-end gap-10">
			<div><Qr text={s.link} /></div>
			{@render list()}
		</div>
		<p class="relative mt-14 inline-block text-3xl">
			keep this tab open
			<Mark kind="underline" seed={seed + 5} delay={300} class="inset-x-0 -bottom-2 h-2 w-full" />
		</p>
		<button class="mt-10 block {quiet}" onclick={onback}>cancel</button>
		{#if s.hashed < s.total}
			<Margin><p>preparing… {percent(s.hashed, s.total)}%</p></Margin>
		{/if}
	{:else if s.phase === 'sending'}
		<Progress value={s.bytes / Math.max(s.total, 1)} rate={s.rate} path={s.path} label="not uploading" />
		<button class="mt-10 block {quiet}" onclick={onback}>cancel</button>
	{:else if s.phase === 'done'}
		<Done names={s.files.map((f) => f.webkitRelativePath || f.name)} />
		<p class="mt-6 text-xl">sent.</p>
		<div class="mt-10 flex flex-wrap items-center gap-8">
			<Stamp onclick={() => input?.click()}>send them more</Stamp>
			<button class={quiet} onclick={onback}>i'm done</button>
		</div>
		<input
			bind:this={input}
			type="file"
			multiple
			class="hidden"
			onchange={() => input?.files?.length && onmore([...input.files])}
		/>
	{:else}
		<p class="text-3xl text-marker">{grievances[s.grievance ?? 'broken']}</p>
		<div class="mt-10"><Stamp onclick={() => location.reload()}>start over</Stamp></div>
	{/if}
</main>
