<script lang="ts">
	const SOURCE = 'https://github.com/Z1xus/uploadnt';
	const stats = import.meta.env.PUBLIC_IMPACTITE;
	const link = 'underline underline-offset-4';
</script>

<main class="mx-auto max-w-[58ch] space-y-6 px-6 pt-[10vh] text-lg leading-relaxed">
	<p>Files go directly from your browser to the other person's. Nothing is uploaded or stored.</p>
	<p>The link holds the encryption key. The part after # never leaves your browser.</p>
	<p>
		A small server, the lobby, connects the two browsers. If they can't connect directly, you can choose to
		send encrypted bytes through our relay, plan b. It doesn't keep anything.
	</p>
	<p>
		The lobby and plan b see your IP address while you're connected and don't store it. The only thing kept is
		a daily count of direct and plan b transfers.
	</p>
	{#if stats}
		<p>
			Page visits are counted with <a href={stats} class={link}>impactite</a>, without cookies. It sees the
			page, referrer and browser, never the link or the file.
		</p>
	{/if}
	<p>
		The <a href={SOURCE} class={link}>source</a> is GPL-3.0. Every release is built and signed in public, so you
		can check the site matches it. Or run your own.
	</p>
	<p><a href="/" class={link}>back</a></p>
</main>
