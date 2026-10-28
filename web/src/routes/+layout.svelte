<script lang="ts">
	import '../app.css';
	import type { Snippet } from 'svelte';
	import { afterNavigate } from '$app/navigation';
	import { onMount } from 'svelte';

	let { children }: { children: Snippet } = $props();

	const dark = () =>
		document.documentElement.dataset.theme === 'dark' ||
		(!document.documentElement.dataset.theme && matchMedia('(prefers-color-scheme: dark)').matches);

	let isDark = $state(dark());
	let shown = $state(false);
	onMount(() => document.fonts.ready.then(() => (shown = true)));

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

<div class="flex min-h-dvh flex-col transition-opacity duration-200 {shown ? '' : 'hold opacity-0'}">
	<div class="flex-1">{@render children()}</div>
	<footer
		class="mx-auto flex w-full max-w-[36rem] flex-wrap gap-x-4 gap-y-1 px-6 pt-16 pb-6 font-fine text-base text-graphite"
	>
		<a href="https://z1x.us/" class="mr-auto underline-offset-2 hover:underline"
			>provided to you by an individual with hatred in heart</a
		>
		<span class="flex gap-x-4">
			<a href="/rest" class="underline-offset-2 hover:underline">the rest</a>
			<button class="cursor-pointer underline-offset-2 hover:underline" onclick={toggle}
				>{isDark ? 'light' : 'dark'}</button
			>
		</span>
	</footer>
</div>
