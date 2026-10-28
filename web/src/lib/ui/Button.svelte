<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { HTMLButtonAttributes } from 'svelte/elements';

	let { children, ...rest }: HTMLButtonAttributes & { children: Snippet } = $props();
</script>

<button
	class="badge text-2xl tracking-wide outline-offset-4 hover:bg-accent hover:text-background focus-visible:outline-2 focus-visible:outline-accent"
	{...rest}
>
	{@render children()}
</button>
