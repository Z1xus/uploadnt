<script lang="ts">
	import type { Snippet } from 'svelte';

	let { children }: { children: Snippet } = $props();
</script>

<aside
	class="mt-10 space-y-3 text-sm leading-snug text-muted lg:absolute lg:top-0 lg:left-[calc(100%+3rem)] lg:mt-0 lg:w-48"
>
	{@render children()}
</aside>
