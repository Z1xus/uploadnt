<script lang="ts">
	const browsers = [
		['brave', 'settings, privacy and security, webrtc ip handling policy. set it to default.'],
		[
			'helium and other chromium browsers',
			'look for webrtc ip handling policy in the privacy settings and set it to default.'
		],
		[
			'firefox on android with ublock origin',
			'in ublock origin, open settings, then privacy, and turn off "prevent webrtc from leaking local ip addresses".'
		],
		['tor browser and mullvad browser', 'webrtc is off there on purpose. send files from another browser.']
	];
</script>

<article class="space-y-6 text-lg leading-relaxed">
	<h1 class="text-3xl">allowing webrtc</h1>
	<p>
		some browsers block webrtc so websites can't see your ip address. uploadn't needs it to connect two
		browsers directly.
	</p>
	<ul class="space-y-4">
		{#each browsers as [name, how]}
			<li><span class="text-accent">{name}</span><br /><span class="text-base text-muted">{how}</span></li>
		{/each}
	</ul>
	<p><a href="/" class="underline">back</a></p>
</article>
