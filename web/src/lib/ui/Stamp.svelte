<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { HTMLButtonAttributes } from 'svelte/elements';

	let { children, rot = -2, ...rest }: HTMLButtonAttributes & { children: Snippet; rot?: number } = $props();
</script>

<button
	class="stamp inline-block cursor-pointer border-[3px] border-marker px-4 py-1.5 text-2xl tracking-wide text-marker outline-offset-4 hover:bg-marker hover:text-paper focus-visible:outline-2 focus-visible:outline-marker"
	style:--rot="{rot}deg"
	{...rest}
>
	{@render children()}
</button>
