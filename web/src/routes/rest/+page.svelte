<script lang="ts">
	import Doodle from '#lib/ui/Doodle.svelte';

	const SOURCE = 'https://github.com/Z1xus/uploadnt';
	const stats = import.meta.env.PUBLIC_IMPACTITE;
	const link = 'underline underline-offset-4';
	const term = 'text-ink';
</script>

<main class="mx-auto max-w-[58ch] space-y-6 px-6 pt-[10vh] text-lg leading-relaxed">
	<p>this is how sending a file usually works:</p>
	<Doodle kind="corp" seed={11} label="you, a big corp in a cloud holding your file, and your friend" />
	<p>
		you give your file to a big company, they pass it on to your friend, and they keep a copy for themselves
		that they can look at whenever they want. <span class="text-marker"
			>bad!!! the corporation has your data!</span
		>
	</p>
	<Doodle kind="direct" seed={23} label="you and your friend with the file going straight between you" />
	<p>
		with uploadn't, the file goes straight from your computer to your friend's, so there's no company in the
		middle that could keep it.
	</p>
	<Doodle
		kind="planb"
		seed={37}
		label="you and your friend with a blindfolded box called plan b between you"
	/>
	<p>
		sometimes the two computers can't reach each other. when that happens we ask you first, and then the file
		goes through our relay, plan b. it only sees scrambled bytes it can't read, and it doesn't keep them.
	</p>
	<p>
		the key that unscrambles the file is the part of the link after the #. browsers don't send that part to
		any server.
	</p>
	<p>
		the lobby and plan b see your ip address while you're connected but don't store it. the only thing we keep
		is how many transfers each day went direct and how many used plan b.
	</p>
	{#if stats}
		<p>
			page visits are counted with <a href={stats} class={link}>impactite</a>, which uses no cookies. it sees
			the page, the referrer and your browser, but not the link or the file.
		</p>
	{/if}
	<p>
		the <a href={SOURCE} class={link}>source</a> is gpl-3.0. every release is built and signed in public, so you
		can check that this site matches it, or run your own copy.
	</p>
	<details>
		<summary
			class="inline-block cursor-pointer list-none border-[3px] border-marker px-3 py-0.5 text-marker hover:bg-marker hover:text-paper [&::-webkit-details-marker]:hidden"
			>i'm a nerd</summary
		>
		<div class="mt-6 space-y-4 text-base text-graphite">
			<p>
				<span class={term}>link.</span> the secret is 128 random bits, written as base32 in the url fragment. the
				lobby only sees a mailbox id derived from it with hkdf-sha256.
			</p>
			<p>
				<span class={term}>handshake.</span> both browsers run spake2 over ed25519 with the secret as the password,
				relayed through the lobby. hkdf-sha256 then derives one key for each direction and one for signalling. with
				a wrong link, the first frame fails to decrypt.
			</p>
			<p>
				<span class={term}>connection.</span> the transfer uses 4 negotiated webrtc data channels, with data striped
				across them and backpressure at 4 mib. the offer, the answer and the ice candidates are encrypted before
				they reach the lobby. ice starts with stun only, and turn is added after the receiver agrees.
			</p>
			<p>
				<span class={term}>frames.</span> frames are encoded with postcard and sealed with chacha20-poly1305. each
				frame carries its 64-bit counter as the nonce, because frames on different channels arrive out of order.
			</p>
			<p>
				<span class={term}>files.</span> files are split into 1 mib chunks hashed with blake3 and sent as 64 kib
				pieces. the manifest lists every chunk hash. the receiver requests chunks with a window of 4 to 32, checks
				each hash and writes to opfs from a worker. it saves a bitmap of finished chunks, so a reload continues
				where it stopped.
			</p>
			<p>
				<span class={term}>core.</span> the protocol is one sans-io rust crate, compiled to wasm with simd.
			</p>
			<p>
				<span class={term}>lobby.</span> on cloudflare, the lobby is a durable object per mailbox that accepts two
				sockets and stores nothing. the self-hosted version is one axum binary that also serves the site and mints
				coturn credentials.
			</p>
			<p>
				<span class={term}>provenance.</span> every script is pinned with sri through an import map, and inline
				scripts with csp hashes. each deploy publishes /.well-known/bundle.txt with an attestation from github actions,
				and an hourly job checks the live site against it.
			</p>
		</div>
	</details>
	<p><a href="/" class={link}>back</a></p>
</main>
