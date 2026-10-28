<script lang="ts">
	import { encode } from 'uqr';

	let { text }: { text: string } = $props();

	const qr = $derived(encode(text, { border: 0 }));
	const path = $derived(
		qr.data.flatMap((row, y) => row.map((on, x) => (on ? `M${x} ${y}h1v1h-1z` : ''))).join('')
	);
</script>

<svg
	viewBox="0 0 {qr.size} {qr.size}"
	class="h-40 w-40 fill-ink"
	shape-rendering="crispEdges"
	role="img"
	aria-label="qr code of the link"
>
	<path d={path} />
</svg>
