<script lang="ts">
	import type { Step } from '#lib/doodle.ts';
	import Doodle from '#lib/ui/Doodle.svelte';
	import Sequence from '#lib/ui/Sequence.svelte';

	const SOURCE = 'https://github.com/Z1xus/uploadnt';
	const stats = import.meta.env.PUBLIC_IMPACTITE;
	const link = 'underline underline-offset-4';
	const term = 'text-ink';

	const nerd: { title: string; text: string; lanes: string[]; steps: Step[] }[] = [
		{
			title: 'the page',
			text: 'every script on the page is pinned by an sri hash in an import map, and inline scripts by csp hashes. each deploy lists its files in /.well-known/bundle.txt. github actions attests that list, and anyone can check the live site against it. the protocol is a sans-io rust crate compiled to wasm with simd.',
			lanes: ['github', 'site', 'browser'],
			steps: [
				{ from: 0, to: 1, label: 'attested bundle' },
				{ from: 1, to: 2, label: 'page + sri hashes' },
				{ from: 2, to: 2, label: 'checks scripts' }
			]
		},
		{
			title: 'the link',
			text: "the sender's browser makes a secret of 128 random bits and puts it in the link after the #, written as base32. you send the link yourself, however you like.",
			lanes: ['sender', 'receiver'],
			steps: [
				{ from: 0, to: 0, label: '128 random bits' },
				{ from: 0, to: 1, label: 'uploadnt.com/#secret', tone: 'red' }
			]
		},
		{
			title: 'the lobby',
			text: 'both browsers open a websocket to the lobby and send a mailbox id derived from the secret with hkdf-sha256. the secret itself never leaves the browser. the lobby holds each mailbox in memory, with room for two sockets, and stores nothing. when both browsers are in, it tells them.',
			lanes: ['sender', 'lobby', 'receiver'],
			steps: [
				{ from: 0, to: 1, label: 'mailbox id' },
				{ from: 2, to: 1, label: 'mailbox id' },
				{ from: 1, to: 0, label: 'peer is here' },
				{ from: 1, to: 2, label: 'peer is here' }
			]
		},
		{
			title: 'the handshake',
			text: "through the lobby, the browsers run spake2 over ed25519 with the secret as the password. these messages don't reveal the secret to the lobby. hkdf-sha256 then derives one key for each direction and one for signalling. with a wrong link, the first frame fails to decrypt.",
			lanes: ['sender', 'lobby', 'receiver'],
			steps: [
				{ from: 0, to: 2, label: 'spake2 message' },
				{ from: 2, to: 0, label: 'spake2 message' },
				{ from: 0, to: 0, label: 'keys', tone: 'red' },
				{ from: 2, to: 2, label: 'same keys', tone: 'red' }
			]
		},
		{
			title: 'signalling',
			text: 'webrtc needs an offer, an answer and ice candidates before it can connect. the browsers encrypt them with the signalling key before sending them through the lobby, which only sees ciphertext.',
			lanes: ['sender', 'lobby', 'receiver'],
			steps: [
				{ from: 0, to: 2, label: 'sealed offer + candidates' },
				{ from: 2, to: 0, label: 'sealed answer + candidates' }
			]
		},
		{
			title: 'the connection',
			text: 'the browsers open 4 negotiated webrtc data channels and stripe the data across them, with backpressure at 4 mib. ice first tries direct paths with stun only. if nothing connects within 5 seconds, the receiver is asked, and turn is added only if they agree. turn relays the same encrypted frames.',
			lanes: ['sender', 'plan b', 'receiver'],
			steps: [
				{ from: 0, to: 2, label: '4 data channels, direct', tone: 'red' },
				{ from: 0, to: 1, label: 'only after yes', tone: 'faint' },
				{ from: 1, to: 2, label: 'same frames', tone: 'faint' }
			]
		},
		{
			title: 'the manifest',
			text: 'the sender splits each file into 1 mib chunks and hashes each chunk with blake3. the first frames carry the file names, sizes and chunk hashes, and the receiver checks each chunk against them as it arrives.',
			lanes: ['sender', 'receiver'],
			steps: [
				{ from: 0, to: 0, label: 'blake3 per 1 mib chunk' },
				{ from: 0, to: 1, label: 'names, sizes, hashes' }
			]
		},
		{
			title: 'the transfer',
			text: "the receiver asks for chunks, with a window that grows from 4 to 32 chunks, and the sender sends each chunk as 64 kib pieces. every frame is encoded with postcard and sealed with chacha20-poly1305. it carries its 64-bit counter as the nonce, because frames on different channels arrive out of order. if a chunk's hash is wrong, the receiver asks for it again.",
			lanes: ['sender', 'receiver'],
			steps: [
				{ from: 1, to: 0, label: 'want these chunks' },
				{ from: 0, to: 1, label: 'sealed 64 kib pieces', tone: 'red' },
				{ from: 1, to: 1, label: 'check blake3' },
				{ from: 1, to: 0, label: 'got it' }
			]
		},
		{
			title: 'saving',
			text: "a worker writes each chunk into the browser's private storage (opfs) and saves a bitmap of what it has every 16 chunks. after a reload or a dropped connection, the transfer continues from there.",
			lanes: ['receiver', 'opfs'],
			steps: [
				{ from: 0, to: 1, label: 'write chunk' },
				{ from: 0, to: 1, label: 'bitmap, every 16 chunks' },
				{ from: 1, to: 0, label: 'after a reload: resume', tone: 'faint' }
			]
		},
		{
			title: 'done',
			text: "when every chunk is there, the receiver copies the file to the place you picked, or downloads it. after that it sends done with the root hash, and the sender's page says sent.",
			lanes: ['sender', 'receiver', 'disk'],
			steps: [
				{ from: 1, to: 2, label: 'save the file' },
				{ from: 1, to: 0, label: 'done + root hash', tone: 'red' }
			]
		}
	];
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
		<ol class="mt-6 space-y-8 text-base text-graphite">
			{#each nerd as n, i}
				<li>
					<p><span class={term}>{i + 1}. {n.title}.</span> {n.text}</p>
					<Sequence lanes={n.lanes} steps={n.steps} seed={i * 97 + 5} />
				</li>
			{/each}
		</ol>
	</details>
	<p><a href="/" class={link}>back</a></p>
</main>
