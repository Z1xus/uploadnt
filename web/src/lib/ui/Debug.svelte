<script lang="ts">
	import type { Candidate, Peer, Report } from '../peer';

	let { peer }: { peer: () => Peer | undefined } = $props();
	let report = $state<Report>();

	$effect(() => {
		const tick = async () =>
			(report =
				(await peer()
					?.report()
					.catch(() => undefined)) ?? report);
		tick();
		const timer = setInterval(tick, 1000);
		return () => clearInterval(timer);
	});

	const name = (c: Candidate) => `${c.type} ${c.protocol} ${c.family}`;

	function grouped(list: Candidate[] = []) {
		const counts = new Map<string, number>();
		for (const c of list) counts.set(name(c), (counts.get(name(c)) ?? 0) + 1);
		return [...counts].map(([c, n]) => (n > 1 ? `${c} ×${n}` : c));
	}
</script>

{#snippet candidates(label: string, list?: Candidate[])}
	<dt class="text-muted">{label}</dt>
	<dd class="flex flex-wrap gap-x-4 gap-y-1">
		{#each grouped(list) as c}<span>{c}</span>{:else}<span class="text-muted">none yet</span>{/each}
	</dd>
{/snippet}

<section class="mt-16 border-t-2 border-dashed border-muted/40 pt-6 text-sm">
	<h2 class="mb-4 font-serif text-base text-muted">debug</h2>
	<dl class="grid grid-cols-[auto_1fr] gap-x-6 gap-y-3">
		<dt class="text-muted">status</dt>
		<dd>{report?.state ?? 'no connection yet'}</dd>
		<dt class="text-muted">path</dt>
		<dd class="text-accent">
			{report?.pair ? `${name(report.pair.local)} to ${name(report.pair.remote)}` : 'none yet'}
		</dd>
		{#if report?.pair?.rtt !== undefined}
			<dt class="text-muted">round trip</dt>
			<dd>{Math.round(report.pair.rtt * 1000)} ms</dd>
		{/if}
		{@render candidates('this device', report?.local)}
		{@render candidates('other device', report?.remote)}
	</dl>
</section>
