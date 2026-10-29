<script lang="ts">
	import Aside from '../ui/Aside.svelte';
	import Button from '../ui/Button.svelte';
	import Logo from '../ui/Logo.svelte';
	import type { Pending } from '../sending.svelte';

	let { seed, previous, onfiles }: { seed: number; previous?: Pending; onfiles: (files: File[]) => void } =
		$props();

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

<Logo {seed} />

<div
	class="relative mt-16 border-2 border-dashed px-8 py-12 transition-colors {over
		? 'border-accent'
		: 'border-foreground'}"
>
	<p class="mb-8 text-xl">
		{#if over}drop to send.{:else if previous}choose <span
				>{previous.files.map((f) => f.name).join(', ')}</span
			> again to keep the same link.{:else}<span class="pointer-coarse:hidden"
				>drop a file anywhere, or choose one.</span
			><span class="hidden pointer-coarse:inline">choose a file.</span>{/if}
	</p>
	<Button onclick={() => input.click()}>give it away</Button>
	<input
		bind:this={input}
		type="file"
		multiple
		class="hidden"
		onchange={() => input.files?.length && onfiles([...input.files])}
	/>

	<Aside>
		<p>send a file directly, with no third parties and no uploads.</p>
		{#if planb !== undefined}
			<p>{planb}% of transfers used plan b in the last 30 days.</p>
		{/if}
	</Aside>
</div>
