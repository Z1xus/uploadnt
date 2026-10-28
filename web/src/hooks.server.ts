import type { Handle } from '@sveltejs/kit/hooks';

const origin = import.meta.env.PUBLIC_ORIGIN ?? 'https://uploadnt.com';

const pages: Record<string, [title: string, description: string]> = {
	'/': ["uploadn't", 'send a file directly, with no third parties and no uploads.'],
	'/get': ['someone is sending you a file', 'open the link to take it.'],
	'/rest': ["how uploadn't works", 'where your file goes, and what we can see.']
};

function meta(path: string) {
	const [title, description] = pages[path] ?? pages['/'];
	return [
		`<meta name="description" content="${description}" />`,
		`<meta name="theme-color" content="#e0312b" />`,
		`<meta property="og:site_name" content="${new URL(origin).host}" />`,
		`<meta property="og:title" content="${title}" />`,
		`<meta property="og:description" content="${description}" />`,
		`<meta property="og:url" content="${origin}${path}" />`,
		`<meta property="og:image" content="${origin}/og.png" />`,
		`<meta property="og:image:width" content="1200" />`,
		`<meta property="og:image:height" content="630" />`,
		`<meta name="twitter:card" content="summary_large_image" />`
	].join('\n\t\t');
}

export const handle: Handle = ({ event, resolve }) =>
	resolve(event, {
		transformPageChunk: ({ html }) => html.replace('%uploadnt.meta%', meta(event.url.pathname))
	});
