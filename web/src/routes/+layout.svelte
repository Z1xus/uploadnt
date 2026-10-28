<script lang="ts">
	import '../app.css';
	import type { Snippet } from 'svelte';
	import { afterNavigate } from '$app/navigation';
	import { onMount } from 'svelte';
	import { fade } from 'svelte/transition';

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

{#if shown}
	<div class="mx-auto flex min-h-dvh max-w-xl flex-col px-6" transition:fade={{ duration: 200 }}>
		<main class="relative flex-1 pt-[10dvh]">{@render children()}</main>
		<footer class="flex flex-wrap gap-x-4 gap-y-1 pt-16 pb-6 font-serif text-muted">
			<a href="https://z1x.us/" class="mr-auto hover:underline"
				>provided to you by an individual with hatred in heart</a
			>
			<a href="/rest" class="hover:underline">the rest</a>
			<button class="hover:underline" onclick={toggle}>{isDark ? 'light' : 'dark'}</button>
		</footer>
	</div>
{/if}
