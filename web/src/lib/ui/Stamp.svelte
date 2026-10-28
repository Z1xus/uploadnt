<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { HTMLButtonAttributes } from 'svelte/elements';

	let { children, ...rest }: HTMLButtonAttributes & { children: Snippet } = $props();
</script>

<button
	class="stamp inline-block cursor-pointer border-[3px] border-marker px-4 py-1.5 text-2xl tracking-wide text-marker outline-offset-4 hover:bg-marker hover:text-paper focus-visible:outline-2 focus-visible:outline-marker"
	{...rest}
>
	{@render children()}
</button>
