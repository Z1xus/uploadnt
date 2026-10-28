<script lang="ts">
	import { seedOf } from '#lib/marker.ts';
	import type { Sending } from '../sending.svelte';
	import { percent, size } from '../rate';
	import Arrow from '../ui/Arrow.svelte';
	import Button from '../ui/Button.svelte';
	import Failure from '../ui/Failure.svelte';
	import Mark from '../ui/Mark.svelte';
	import Progress from '../ui/Progress.svelte';
	import Qr from '../ui/Qr.svelte';

	let { s, onback, onmore }: { s: Sending; onback: () => void; onmore: (files: File[]) => void } = $props();

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
				{f.webkitRelativePath || f.name} <span class="text-muted">{size(f.size)}</span>
			</li>{/each}
	</ul>
	{#if s.hashed < s.total}<p class="mt-1 text-sm text-muted">preparing… {percent(s.hashed, s.total)}%</p>{/if}
{/snippet}

{#if s.phase === 'waiting' && s.more}
	<p class="text-3xl">waiting for them to take it.</p>
	<div class="mt-3">{@render list()}</div>
	<button class="mt-10 block text-muted hover:underline" onclick={onback}>cancel</button>
{:else if s.phase === 'waiting'}
	<div class="relative">
		<button class="block w-full cursor-copy text-left text-2xl break-all" onclick={copy}>
			<Mark kind="underline" seed={seed + 1} show={copied}>{s.link}</Mark>
		</button>
		<p
			class="mt-3 flex items-end gap-1 font-serif text-lg text-accent lg:absolute lg:top-0 lg:left-[calc(100%+0.5rem)] lg:mt-0 lg:w-48 lg:items-start"
		>
			<Arrow seed={seed + 7} up class="inline-block h-10 w-10 lg:hidden" />
			<Arrow seed={seed + 7} class="hidden h-8 w-14 shrink-0 lg:inline-block" />
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
	<p class="mt-14 text-3xl"><Mark kind="underline" seed={seed + 5} delay={300}>keep this tab open</Mark></p>
	<button class="mt-10 block text-muted hover:underline" onclick={onback}>cancel</button>
{:else if s.phase === 'sending'}
	<Progress value={s.bytes / Math.max(s.total, 1)} rate={s.rate} path={s.path}>
		{#snippet label()}<Mark kind="underline" seed={seed + 9}>not</Mark> uploading{/snippet}
	</Progress>
	<button class="mt-10 block text-muted hover:underline" onclick={onback}>cancel</button>
{:else if s.phase === 'done'}
	<p class="text-4xl break-words">
		<Mark kind="underline" seed={seed + 3}
			>{s.files.map((f) => f.webkitRelativePath || f.name).join(', ')}</Mark
		>
	</p>
	<p class="mt-6 text-xl">sent.</p>
	<div class="mt-10 flex flex-wrap items-center gap-8">
		<Button onclick={() => input?.click()}>send them more</Button>
		<button class="text-muted hover:underline" onclick={onback}>i'm done</button>
	</div>
	<input
		bind:this={input}
		type="file"
		multiple
		class="hidden"
		onchange={() => input?.files?.length && onmore([...input.files])}
	/>
{:else}
	<Failure grievance={s.grievance} error={s.error} />
	<div class="mt-10"><Button onclick={() => location.reload()}>start over</Button></div>
{/if}
