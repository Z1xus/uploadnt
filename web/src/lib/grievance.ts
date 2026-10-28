export const grievances = {
	link: 'this link is broken. ask for a new one.',
	full: 'someone already took this one. ask for a new link.',
	mismatch: "the link doesn't match. ask for it again.",
	version: 'you two run different versions. both of you reload.',
	left: 'the other end walked away. their tab has to stay open.',
	room: "this browser won't give us the disk space. try chrome on a computer.",
	safari: "it's safari. it's always safari. keep this tab in front.",
	broken: 'something broke and it was not you. reload and try again.'
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
