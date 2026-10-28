<script lang="ts">
	const SOURCE = 'https://github.com/Z1xus/uploadnt';
	const stats = import.meta.env.PUBLIC_IMPACTITE;
</script>

<main class="mx-auto max-w-[58ch] space-y-6 px-6 pt-[10vh] text-lg leading-relaxed">
	<h2 class="text-2xl">how it works</h2>
	<p>
		the file goes straight from your browser to theirs. the key lives in the link after the #. browsers never
		send that part to any server
	</p>
	<p>
		a small server called the lobby introduces the two browsers to each other. then its out of the picture
	</p>
	<p>
		sometimes two networks cant connect directly, so the bytes go through a relay called plan b. it just
		passes the encrypted bytes along and keeps nothing. the page says plan b when that happens
	</p>
	<h2 class="pt-4 text-2xl">privacy</h2>
	<p>
		the lobby and plan b see your ip while youre connected and dont write it down. the only thing kept is a
		daily count of how many transfers went direct and how many needed plan b
	</p>
	{#if stats}
		<p>
			page visits here are counted with <a href={stats} class="underline underline-offset-4">impactite</a>, my
			own cookieless analytics. it gets the page path, the referrer and the browser, never the link or the
			file. no sessions, no heatmaps
		</p>
	{/if}
	<h2 class="pt-4 text-2xl">run your own</h2>
	<p>
		the <a href={SOURCE} class="underline underline-offset-4">source</a> is gpl-3.0 and every release is built and
		signed in public ci, so anyone can check that the live site matches the source. if you dont wanna trust me,
		run your own, its one binary
	</p>
	<p><a href="/" class="underline underline-offset-4">back</a></p>
</main>
