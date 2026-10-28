<script lang="ts">
	import { seedOf, underline } from '#lib/marker.ts';

	let { names }: { names: string[] } = $props();
</script>

<div class="relative pb-6 text-4xl break-words">
	{names.join(', ')}
	<svg
		class="mark absolute inset-x-0 bottom-0 h-3 w-full overflow-visible"
		viewBox="0 0 100 10"
		preserveAspectRatio="none"
		aria-hidden="true"
	>
		<path
			d={underline(seedOf(names.join()))}
			stroke-width="2"
			vector-effect="non-scaling-stroke"
			style:stroke="var(--ink)"
		/>
	</svg>
</div>
