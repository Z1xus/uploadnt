import type { JsFile } from './core';

export interface Sink {
	where: string;
	open(root: string, files: JsFile[], chunks: number): Promise<Uint8Array | undefined>;
	write(file: number, offset: number, chunk: number, data: Uint8Array): Promise<void>;
	finish(): Promise<void>;
}

type Target = FileSystemFileHandle | FileSystemDirectoryHandle | undefined;

const flat = (name: string) => name.replaceAll('/', '_');

async function pick(files: JsFile[]): Promise<Target> {
	if (!('showSaveFilePicker' in window)) return;
	try {
		if (files.length === 1) return await window.showSaveFilePicker({ suggestedName: flat(files[0].name) });
		return await window.showDirectoryPicker({ mode: 'readwrite' });
	} catch (e) {
		if ((e as Error).name === 'AbortError') throw e;
	}
}

async function handleFor(target: Target, name: string) {
	return target?.kind === 'directory' ? target.getFileHandle(flat(name), { create: true }) : target!;
}

async function staging() {
	return navigator.storage.getDirectory();
}

export async function sweep() {
	const root = await staging().catch(() => undefined);
	if (!root) return;
	for await (const [name, entry] of root.entries()) {
		if (entry.kind !== 'directory') continue;
		const done = await entry.getFileHandle('delivered').then(
			() => true,
			() => false
		);
		if (done) await root.removeEntry(name, { recursive: true }).catch(() => {});
	}
}

class Staged implements Sink {
	private worker = new Worker(new URL('./disk.worker.ts', import.meta.url), { type: 'module' });
	private calls = new Map<number, { resolve: (v: any) => void; reject: (e: Error) => void }>();
	private next = 0;
	private root = '';
	private files: JsFile[] = [];

	constructor(
		private target: Target,
		readonly where: string
	) {
		this.worker.onmessage = ({ data }) => {
			const call = this.calls.get(data.id)!;
			this.calls.delete(data.id);
			if ('error' in data) call.reject(new Error(data.error));
			else call.resolve(data.result);
		};
	}

	private call<T>(req: object, transfer: Transferable[] = []) {
		const id = this.next++;
		return new Promise<T>((resolve, reject) => {
			this.calls.set(id, { resolve, reject });
			this.worker.postMessage({ id, req }, transfer);
		});
	}

	open(root: string, files: JsFile[], chunks: number) {
		this.root = root;
		this.files = files;
		return this.call<Uint8Array>({ op: 'open', root, sizes: files.map((f) => f.size), chunks });
	}

	write(file: number, offset: number, chunk: number, data: Uint8Array) {
		return this.call<void>({ op: 'write', file, offset, chunk, data }, [data.buffer]);
	}

	async finish() {
		await this.call({ op: 'finish' });
		this.worker.terminate();
		const root = await staging();
		const dir = await root.getDirectoryHandle(this.root);
		for (const [i, entry] of this.files.entries()) {
			const file = await (await dir.getFileHandle(`${i}`)).getFile();
			if (this.target) {
				const out = await (await handleFor(this.target, entry.name)).createWritable();
				await file.stream().pipeTo(out);
			} else {
				const a = document.createElement('a');
				a.href = URL.createObjectURL(file);
				a.download = flat(entry.name);
				a.click();
			}
		}
		if (this.target) await root.removeEntry(this.root, { recursive: true });
		else await dir.getFileHandle('delivered', { create: true });
	}
}

class Direct implements Sink {
	private out: FileSystemWritableFileStream[] = [];

	constructor(
		private target: Target,
		readonly where: string
	) {}

	async open(_root: string, files: JsFile[]) {
		for (const f of files) this.out.push(await (await handleFor(this.target, f.name)).createWritable());
		return undefined;
	}

	write(file: number, offset: number, _chunk: number, data: Uint8Array) {
		return this.out[file].write({ type: 'write', position: offset, data: data as Uint8Array<ArrayBuffer> });
	}

	async finish() {
		await Promise.all(this.out.map((w) => w.close()));
	}
}

export async function openSink(files: JsFile[], size: number): Promise<Sink> {
	const target = await pick(files);
	const where = target ? `into ${target.name}` : 'into your downloads, at the end';
	const { quota = 0, usage = 0 } = await navigator.storage.estimate().catch((): StorageEstimate => ({}));
	const opfs = 'getDirectory' in navigator.storage;
	if (opfs && quota - usage > size * 1.05) return new Staged(target, where);
	if (target) return new Direct(target, where);
	throw new Error('room');
}
