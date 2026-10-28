<script lang="ts">
	import { strike, underline } from '#lib/marker.ts';

	let {
		kind = 'strike',
		seed,
		width = 3,
		delay = 0,
		class: cls = ''
	}: {
		kind?: 'strike' | 'underline';
		seed: number;
		width?: number;
		delay?: number;
		class?: string;
	} = $props();

	const d = $derived(kind === 'strike' ? strike(seed) : underline(seed));
</script>

<svg
	class="mark pointer-events-none absolute overflow-visible {cls}"
	viewBox={kind === 'strike' ? '0 0 100 20' : '0 0 100 10'}
	preserveAspectRatio="none"
	aria-hidden="true"
	style:--delay="{delay}ms"
>
	<path {d} pathLength="1" stroke-width={width} vector-effect="non-scaling-stroke" />
</svg>
