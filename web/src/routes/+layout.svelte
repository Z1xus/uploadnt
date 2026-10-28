<script lang="ts">
	import '../app.css';
	import type { Snippet } from 'svelte';
	import { afterNavigate } from '$app/navigation';

	let { children }: { children: Snippet } = $props();

	const dark = () =>
		document.documentElement.dataset.theme === 'dark' ||
		(!document.documentElement.dataset.theme && matchMedia('(prefers-color-scheme: dark)').matches);

	let isDark = $state(dark());

	const stats = import.meta.env.PUBLIC_IMPACTITE;
	afterNavigate(({ from }) => {
		if (!stats) return;
		// only the path, never location.href: the fragment holds the secret
		const body = {
			n: 'pageview',
			d: location.hostname,
			u: location.origin + location.pathname,
			r: from ? null : document.referrer
		};
		fetch(`${stats}/api/event`, { method: 'POST', keepalive: true, body: JSON.stringify(body) }).catch(
			() => {}
		);
	});

	function toggle() {
		const theme = isDark ? 'light' : 'dark';
		document.documentElement.dataset.theme = theme;
		localStorage.theme = theme;
		isDark = !isDark;
	}
</script>

<div class="flex min-h-dvh flex-col">
	<div class="flex-1">{@render children()}</div>
	<footer
		class="mx-auto flex w-full max-w-[58ch] flex-wrap gap-x-4 px-6 pt-16 pb-6 font-fine text-[11.5px] text-graphite"
	>
		<span class="mr-auto">provided to you by an individual with hatred in heart</span>
		<a href="/rest" class="underline-offset-2 hover:underline">the rest</a>
		<button class="cursor-pointer underline-offset-2 hover:underline" onclick={toggle}
			>{isDark ? 'light' : 'dark'}</button
		>
	</footer>
</div>
