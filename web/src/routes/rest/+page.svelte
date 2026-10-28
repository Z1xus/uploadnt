<script lang="ts">
	const SOURCE = 'https://github.com/Z1xus/uploadnt';
	const stats = import.meta.env.PUBLIC_IMPACTITE;
	const link = 'underline underline-offset-4';
</script>

<main class="mx-auto max-w-[58ch] space-y-6 px-6 pt-[10vh] text-lg leading-relaxed">
	<p>files go directly from your browser to the other person's. nothing is uploaded or stored.</p>
	<p>the link holds the encryption key. the part after # never leaves your browser.</p>
	<p>
		a small server, the lobby, connects the two browsers. if they can't connect directly, you can choose to
		send encrypted bytes through our relay, plan b. it doesn't keep anything.
	</p>
	<p>
		the lobby and plan b see your ip address while you're connected and don't store it. the only thing kept is
		a daily count of direct and plan b transfers.
	</p>
	{#if stats}
		<p>
			page visits are counted with <a href={stats} class={link}>impactite</a>, without cookies. it sees the
			page, referrer and browser, never the link or the file.
		</p>
	{/if}
	<p>
		the <a href={SOURCE} class={link}>source</a> is gpl-3.0. every release is built and signed in public, so you
		can check the site matches it. or run your own.
	</p>
	<p><a href="/" class={link}>back</a></p>
</main>
