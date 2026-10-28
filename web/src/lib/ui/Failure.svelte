<script lang="ts">
	import { grievances, type Grievance } from '../grievance';

	let { grievance = 'broken', error }: { grievance?: Grievance; error?: string } = $props();
</script>

<p class="text-3xl text-accent">{grievances[grievance]}</p>
{#if grievance === 'webrtc'}<a href="/webrtc" class="mt-3 block text-muted underline">how to allow it</a>{/if}
{#if grievance === 'broken' && error}<p class="mt-3 text-sm break-all text-muted">{error}</p>{/if}
