export const grievances = {
	link: "this link isn't valid. ask for a new one.",
	full: 'this link is already in use. ask for a new one.',
	mismatch: "this link doesn't match. ask for it again.",
	version: "you're on different versions. both of you reload the page.",
	unreachable: "couldn't connect. try a different network.",
	left: 'the other person stopped the transfer.',
	room: 'not enough storage in this browser. try chrome on a computer.',
	safari: 'keep this tab in front, or safari will pause the transfer.',
	broken: 'something went wrong. reload and try again.'
} as const;

export type Grievance = keyof typeof grievances;

export function grievanceOf(e: unknown): Grievance {
	console.error(e);
	const msg = String((e as Error)?.message ?? e);
	if (msg === 'decrypt') return 'mismatch';
	if (msg === 'version') return 'version';
	if (msg === 'room') return 'room';
	return 'broken';
}
