import init from '@uploadnt/core';
import wasm from '@uploadnt/core/pkg/core_bg.wasm?url';

export * from '@uploadnt/core';

// replaced with the wasm hash at build time, see vite.config.ts
const WASM_SRI = '__uploadnt_wasm_sri__';

let ready: Promise<unknown> | undefined;

export const loadCore = () =>
	(ready ??= init({
		module_or_path: fetch(wasm, WASM_SRI.startsWith('sha384-') ? { integrity: WASM_SRI } : {})
	}));
