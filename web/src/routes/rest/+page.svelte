<script lang="ts">
	const SOURCE = 'https://github.com/Z1xus/uploadnt';
	const stats = import.meta.env.PUBLIC_IMPACTITE;
</script>

<main class="mx-auto max-w-[58ch] space-y-6 px-6 pt-[10vh] text-lg leading-relaxed">
	<p>
		the file goes from one browser straight to the other, encrypted with a key that only lives in the link.
		the part after the # never reaches any server.
	</p>
	<p>
		a small server called the lobby introduces the two ends. it sees two addresses and ciphertext, then gets
		out of the way.
	</p>
	<p>
		when two networks refuse to meet, the bytes go through a relay called plan b. it forwards ciphertext as it
		arrives and keeps nothing. the page says plan b when that happens.
	</p>
	<p>
		a web page cannot prove itself. a server could lie to you and nobody else. the code is public and every
		release is signed, so a swap shows up for everyone except the person it targets.
	</p>
	<p>
		privacy: the lobby and plan b see your ip address while you use them and write none of it down. the only
		thing kept is a daily count of transfers that went direct and transfers that needed plan b.{#if stats}
			page visits are counted with <a href={stats} class="underline underline-offset-4">impactite</a>, without
			cookies. it gets the page, the referrer and your browser, never the link or the file. no recordings, no
			heatmaps.{/if}
	</p>
	<p>
		run your own: one binary, one port. the <a href={SOURCE} class="underline underline-offset-4">source</a> is
		gpl-3.0.
	</p>
	<p><a href="/" class="underline underline-offset-4">back</a></p>
</main>
