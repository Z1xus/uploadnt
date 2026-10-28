<script lang="ts">
	import type { Step } from '#lib/doodle.ts';
	import { circle } from '#lib/marker.ts';
	import Strokes from '#lib/ui/Strokes.svelte';
	import Doodle from '#lib/ui/Doodle.svelte';
	import Sequence from '#lib/ui/Sequence.svelte';

	const SOURCE = 'https://github.com/Z1xus/uploadnt';
	const stats = import.meta.env.PUBLIC_IMPACTITE;
	const link = 'underline';

	const refs: Record<string, string> = {
		sri: 'https://developer.mozilla.org/en-US/docs/Web/Security/Subresource_Integrity',
		'import map': 'https://developer.mozilla.org/en-US/docs/Web/HTML/Element/script/type/importmap',
		csp: 'https://developer.mozilla.org/en-US/docs/Web/HTTP/CSP',
		'/.well-known/bundle.txt': '/.well-known/bundle.txt',
		'github actions': 'https://github.com/actions/attest-build-provenance',
		'sans-io': 'https://sans-io.readthedocs.io/',
		'rust crate': `${SOURCE}/tree/main/crates/uploadnt-core`,
		wasm: 'https://webassembly.org/',
		simd: 'https://github.com/WebAssembly/simd',
		base32: 'https://datatracker.ietf.org/doc/html/rfc4648#section-6',
		'hkdf-sha256': 'https://datatracker.ietf.org/doc/html/rfc5869',
		websocket: 'https://developer.mozilla.org/en-US/docs/Web/API/WebSockets_API',
		spake2: 'https://datatracker.ietf.org/doc/html/rfc9382',
		ed25519: 'https://datatracker.ietf.org/doc/html/rfc8032',
		webrtc: 'https://developer.mozilla.org/en-US/docs/Web/API/WebRTC_API',
		ice: 'https://datatracker.ietf.org/doc/html/rfc8445',
		'data channels': 'https://developer.mozilla.org/en-US/docs/Web/API/RTCDataChannel',
		stun: 'https://datatracker.ietf.org/doc/html/rfc8489',
		turn: 'https://datatracker.ietf.org/doc/html/rfc8656',
		blake3: 'https://github.com/BLAKE3-team/BLAKE3',
		postcard: 'https://github.com/jamesmunns/postcard',
		'chacha20-poly1305': 'https://datatracker.ietf.org/doc/html/rfc8439',
		opfs: 'https://developer.mozilla.org/en-US/docs/Web/API/File_System_API/Origin_private_file_system'
	};
	const terms = new RegExp(
		`(?<![\\w-])(${Object.keys(refs)
			.sort((a, b) => b.length - a.length)
			.map((t) => t.replace(/[.*+?^${}()|[\]\\/]/g, '\\$&'))
			.join('|')})(?![\\w-])`
	);

	function linked(text: string) {
		const seen = new Set<string>();
		return text.split(terms).map((part, i) => {
			const first = i % 2 === 1 && !seen.has(part);
			seen.add(part);
			return { part, href: first ? refs[part] : undefined };
		});
	}

	const nerd: { title: string; text: string; lanes: string[]; steps: Step[] }[] = [
		{
			title: 'page load',
			text: 'every script on the page is pinned by an sri hash in an import map, and inline scripts by csp hashes. each deploy lists its files in /.well-known/bundle.txt. github actions attests that list, and anyone can check the live site against it. the protocol is a sans-io rust crate compiled to wasm with simd.',
			lanes: ['github', 'site', 'browser'],
			steps: [
				{ from: 0, to: 1, label: 'attested bundle' },
				{ from: 1, to: 2, label: 'page + sri hashes' },
				{ from: 2, to: 2, label: 'checks scripts' }
			]
		},
		{
			title: 'link',
			text: "the sender's browser makes a secret of 128 random bits and puts it in the link after the #, written as base32. you send the link yourself, however you like.",
			lanes: ['sender', 'receiver'],
			steps: [
				{ from: 0, to: 0, label: '128 random bits' },
				{ from: 0, to: 1, label: 'uploadnt.com/#secret', tone: 'accent' }
			]
		},
		{
			title: 'lobby',
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
			title: 'handshake',
			text: "through the lobby, the browsers run spake2 over ed25519 with the secret as the password. these messages don't reveal the secret to the lobby. hkdf-sha256 then derives one key for each direction and one for signalling. with a wrong link, the first frame fails to decrypt.",
			lanes: ['sender', 'lobby', 'receiver'],
			steps: [
				{ from: 0, to: 2, label: 'spake2 message' },
				{ from: 2, to: 0, label: 'spake2 message' },
				{ from: 0, to: 0, label: 'keys', tone: 'accent' },
				{ from: 2, to: 2, label: 'same keys', tone: 'accent' }
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
			title: 'connection',
			text: 'the browsers open 4 negotiated webrtc data channels and stripe the data across them, with backpressure at 4 mib. ice first tries direct paths with stun only. if nothing connects within 5 seconds, the receiver is asked, and turn is added only if they agree. turn relays the same encrypted frames.',
			lanes: ['sender', 'plan b', 'receiver'],
			steps: [
				{ from: 0, to: 2, label: '4 data channels, direct', tone: 'accent' },
				{ from: 0, to: 1, label: 'only after yes', tone: 'muted' },
				{ from: 1, to: 2, label: 'same frames', tone: 'muted' }
			]
		},
		{
			title: 'manifest',
			text: 'the sender splits each file into 1 mib chunks and hashes each chunk with blake3. the first frames carry the file names, sizes and chunk hashes, and the receiver checks each chunk against them as it arrives.',
			lanes: ['sender', 'receiver'],
			steps: [
				{ from: 0, to: 0, label: 'blake3 per 1 mib chunk' },
				{ from: 0, to: 1, label: 'names, sizes, hashes' }
			]
		},
		{
			title: 'transfer',
			text: "the receiver asks for chunks, with a window that grows from 4 to 32 chunks, and the sender sends each chunk as 64 kib pieces. every frame is encoded with postcard and sealed with chacha20-poly1305. it carries its 64-bit counter as the nonce, because frames on different channels arrive out of order. if a chunk's hash is wrong, the receiver asks for it again.",
			lanes: ['sender', 'receiver'],
			steps: [
				{ from: 1, to: 0, label: 'want these chunks' },
				{ from: 0, to: 1, label: 'sealed 64 kib pieces', tone: 'accent' },
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
				{ from: 1, to: 0, label: 'after a reload: resume', tone: 'muted' }
			]
		},
		{
			title: 'done',
			text: "when every chunk is there, the receiver copies the file to the place you picked, or downloads it. after that it sends done with the root hash, and the sender's page says sent.",
			lanes: ['sender', 'receiver', 'disk'],
			steps: [
				{ from: 1, to: 2, label: 'save the file' },
				{ from: 1, to: 0, label: 'done + root hash', tone: 'accent' }
			]
		}
	];
</script>

<article class="space-y-6 text-lg leading-relaxed">
	<p>this is how sending a file usually works:</p>
	<Doodle kind="corp" seed={11} label="you, a big corp in a cloud holding your file, and your friend" />
	<p>
		you give your file to a big company, they pass it on to your friend, and they keep a copy for themselves
		that they can look at whenever they want. <span class="text-accent"
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
			class="badge cursor-pointer list-none hover:bg-accent hover:text-background [&::-webkit-details-marker]:hidden"
			>i'm a nerd</summary
		>
		<ol class="mt-6 space-y-8 text-base text-muted">
			{#each nerd as n, i}
				<li id={n.title.replaceAll(' ', '-')} class="scroll-mt-8">
					<h3>
						<a href="#{n.title.replaceAll(' ', '-')}" class="flex items-center gap-3 text-xl text-foreground"
							><span class="relative grid size-9 shrink-0 place-items-center text-base"
								>{i + 1}<svg class="absolute inset-0 overflow-visible" viewBox="0 0 40 40" aria-hidden="true"
									><Strokes strokes={[{ d: circle(i * 31 + 7), tone: 'accent' }]} width={2} step={0} /></svg
								></span
							>{n.title}</a
						>
					</h3>
					<p class="mt-2">
						{#each linked(n.text) as t}{#if t.href}<a href={t.href} class="{link} decoration-muted/50"
									>{t.part}</a
								>{:else}{t.part}{/if}{/each}
					</p>
					<Sequence lanes={n.lanes} steps={n.steps} seed={i * 97 + 5} />
				</li>
			{/each}
		</ol>
	</details>
	<p><a href="/" class={link}>back</a></p>
</article>
