<script lang="ts">
	import Margin from '../ui/Margin.svelte';
	import Scrawl from '../ui/Scrawl.svelte';
	import Stamp from '../ui/Stamp.svelte';
	import Struck from '../ui/Struck.svelte';
	import type { Pending } from '../sending.svelte';
	import { tilted } from '../ui/tilt';

	let { seed, previous, onfiles }: { seed: number; previous?: Pending; onfiles: (files: File[]) => void } =
		$props();

	let input: HTMLInputElement;
	let over = $state(false);

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
	<h1 class="bleed text-[clamp(4rem,16vw,8.5rem)] leading-[0.9]" use:tilted={seed}>
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
		use:tilted={seed + 3}
	>
		<p class="mb-8 text-xl">
			{#if over}let go.{:else if previous}pick <span class="normal-case"
					>{previous.files.map((f) => f.name).join(', ')}</span
				> again. the link still works.{:else}<span class="pointer-coarse:hidden"
					>drop a file anywhere on this page.</span
				><span class="hidden pointer-coarse:inline">pick a file.</span>{/if}
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
			<p use:tilted={seed + 4}>both of you stay online the whole time.</p>
			<p use:tilted={seed + 5}>it goes as fast as your connection sends.</p>
			<p use:tilted={seed + 6}>phones sleep. keep the screen on.</p>
			<p class="text-marker" use:tilted={seed + 7}>yes, really.</p>
		</Margin>
	</div>
</main>
