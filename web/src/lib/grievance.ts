export const grievances = {
	link: "This link isn't valid. Ask for a new one.",
	full: 'This link is already in use. Ask for a new one.',
	mismatch: "This link doesn't match. Ask for it again.",
	version: "You're on different versions. Both of you reload the page.",
	unreachable: "Couldn't connect. Try a different network.",
	left: 'The other person stopped the transfer.',
	room: 'Not enough storage in this browser. Try Chrome on a computer.',
	safari: 'Keep this tab in front, or Safari will pause the transfer.',
	broken: 'Something went wrong. Reload and try again.'
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
