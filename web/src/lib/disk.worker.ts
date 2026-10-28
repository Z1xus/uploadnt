/// <reference lib="webworker" />

type Request =
	| { op: 'open'; root: string; sizes: number[]; chunks: number }
	| { op: 'write'; file: number; offset: number; chunk: number; data: Uint8Array }
	| { op: 'finish' };

const PERSIST_EVERY = 16;

let files: FileSystemSyncAccessHandle[] = [];
let bitmap: FileSystemSyncAccessHandle;
let have: Uint8Array;
let dirty = 0;

function persist() {
	files.forEach((f) => f.flush());
	bitmap.write(have, { at: 0 });
	bitmap.flush();
	dirty = 0;
}

async function handle(req: Request) {
	switch (req.op) {
		case 'open': {
			if (files.length) return have.slice();
			const dir = await (
				await navigator.storage.getDirectory()
			).getDirectoryHandle(req.root, { create: true });
			for (const [i, size] of req.sizes.entries()) {
				const file = await (await dir.getFileHandle(`${i}`, { create: true })).createSyncAccessHandle();
				if (file.getSize() !== size) file.truncate(size);
				files.push(file);
			}
			bitmap = await (await dir.getFileHandle('have', { create: true })).createSyncAccessHandle();
			have = new Uint8Array(Math.ceil(req.chunks / 8));
			bitmap.read(have, { at: 0 });
			return have.slice();
		}
		case 'write':
			files[req.file].write(req.data, { at: req.offset });
			have[req.chunk >> 3] |= 1 << (req.chunk & 7);
			if (++dirty >= PERSIST_EVERY) persist();
			return;
		case 'finish':
			persist();
			files.forEach((f) => f.close());
			bitmap.close();
			files = [];
	}
}

let queue = Promise.resolve();

onmessage = ({ data: { id, req } }: MessageEvent<{ id: number; req: Request }>) => {
	queue = queue.then(() =>
		handle(req).then(
			(result) => postMessage({ id, result }),
			(error) => postMessage({ id, error: String(error) })
		)
	);
};
