import { sveltekit } from '@sveltejs/kit/vite';
import adapter from '@sveltejs/adapter-static';
import tailwindcss from '@tailwindcss/vite';
import { defineConfig, type Plugin } from 'vite';

const wasmIntegrity: Plugin = {
	name: 'wasm-integrity',
	apply: 'build',
	async generateBundle(_, bundle) {
		const wasm = Object.values(bundle).find((f) => f.type === 'asset' && f.fileName.endsWith('.wasm'));
		if (wasm?.type !== 'asset') return;
		const digest = new Uint8Array(
			await crypto.subtle.digest('SHA-384', wasm.source as Uint8Array<ArrayBuffer>)
		);
		const sri = `sha384-${btoa(String.fromCharCode(...digest))}`;
		for (const f of Object.values(bundle))
			// vite has no idea what sri is for wasm, so a fake string goes in and gets find-and-replaced after. peak engineering
			if (f.type === 'chunk') f.code = f.code.replaceAll('__uploadnt_wasm_sri__', sri);
	}
};

export default defineConfig({
	plugins: [tailwindcss(), sveltekit({ adapter: adapter() }), wasmIntegrity],
	envPrefix: 'PUBLIC_',
	server: {
		fs: { allow: ['..'] },
		proxy: { '/m': { target: 'ws://localhost:8787', ws: true }, '/stats': 'http://localhost:8787' }
	}
});
