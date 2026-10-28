<script lang="ts">
	import Margin from '../ui/Margin.svelte';
	import Scrawl from '../ui/Scrawl.svelte';
	import Stamp from '../ui/Stamp.svelte';
	import Struck from '../ui/Struck.svelte';
	import type { Pending } from '../sending.svelte';

	let {
		seed,
		previous,
		again,
		onfiles
	}: { seed: number; previous?: Pending; again: boolean; onfiles: (files: File[]) => void } = $props();

	let input: HTMLInputElement;
	let over = $state(false);
	let planb = $state<number>();

	fetch('/stats')
		.then((r) => r.json())
		.then((s: { direct: number; planb: number }) => {
			// a handful of transfers says nothing about networks
			if (s.direct + s.planb >= 50) planb = Math.round((100 * s.planb) / (s.direct + s.planb));
		})
		.catch(() => {});

	function drop(e: DragEvent) {
		e.preventDefault();
		over = false;
		const files = [...(e.dataTransfer?.files ?? [])].filter((f) => f.size || f.type);
		if (files.length) onfiles(files);
	}
</script>

<svelte:window
	ondragover={(e) => {
		e.preventDefault();
		over = true;
	}}
	ondragleave={(e) => !e.relatedTarget && (over = false)}
	ondrop={drop}
/>

<main class="relative mx-auto max-w-[58ch] px-6 pt-[14vh]">
	<h1 class="bleed text-[clamp(4rem,16vw,8.5rem)] leading-[0.9]">
		<Struck seed={seed + 1} delay={200}>upload</Struck><Scrawl
			seed={seed + 2}
			delay={520}
			class="-ml-[0.04em] h-[0.72em] w-[0.78em] align-[-0.04em]"
		/>
	</h1>

	<div
		class="relative mt-16 border-2 border-dashed px-8 py-12 transition-colors {over
			? 'border-marker'
			: 'border-ink'}"
	>
		<p class="mb-8 text-xl">
			{#if over}drop to send.{:else if again}choose files to send on the same link.{:else if previous}choose <span
					>{previous.files.map((f) => f.name).join(', ')}</span
				> again to keep the same link.{:else}<span class="pointer-coarse:hidden"
					>drop a file anywhere, or choose one.</span
				><span class="hidden pointer-coarse:inline">choose a file.</span>{/if}
		</p>
		<Stamp onclick={() => input.click()}>give it away</Stamp>
		<input
			bind:this={input}
			type="file"
			multiple
			class="hidden"
			onchange={() => input.files?.length && onfiles([...input.files])}
		/>

		<Margin>
			<p>send a file directly, with no third parties and no uploads.</p>
			<p>keep this tab open until they've got it.</p>
			{#if planb !== undefined}
				<p>{planb}% of transfers used plan b in the last 30 days.</p>
			{/if}
		</Margin>
	</div>
</main>
