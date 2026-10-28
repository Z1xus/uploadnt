/// <reference lib="webworker" />

type Request =
	| { op: 'open'; root: string; sizes: number[]; chunks: number }
	| { op: 'write'; file: number; offset: number; chunk: number; data: Uint8Array }
	| { op: 'finish' };

const PERSIST_EVERY = 16;

let files: FileSystemSyncAccessHandle[] = [];
let bitmap: FileSystemSyncAccessHandle;
let have: Uint8Array;
let opened = '';
let dirty = 0;

async function lock(file: FileSystemFileHandle) {
	// a reloaded page can race the old page's worker, which still holds the handle for a moment
	for (let tries = 0; ; tries++) {
		try {
			return await file.createSyncAccessHandle();
		} catch (e) {
			if ((e as Error).name !== 'NoModificationAllowedError') throw e;
			if (tries === 10) throw new Error('tab');
			await new Promise((resolve) => setTimeout(resolve, 200));
		}
	}
}

function persist() {
	files.forEach((f) => f.flush());
	bitmap.write(have, { at: 0 });
	bitmap.flush();
	dirty = 0;
}

async function handle(req: Request) {
	switch (req.op) {
		case 'open': {
			if (files.length) {
				if (req.root !== opened) throw new Error('root');
				return have.slice();
			}
			opened = req.root;
			const dir = await (
				await navigator.storage.getDirectory()
			).getDirectoryHandle(req.root, { create: true });
			for (const [i, size] of req.sizes.entries()) {
				const file = await lock(await dir.getFileHandle(`${i}`, { create: true }));
				if (file.getSize() !== size) file.truncate(size);
				files.push(file);
			}
			bitmap = await lock(await dir.getFileHandle('have', { create: true }));
			have = new Uint8Array(Math.ceil(req.chunks / 8));
			bitmap.read(have, { at: 0 });
			return have.slice();
		}
		case 'write':
			for (let done = 0; done < req.data.length;) {
				const n = files[req.file].write(req.data.subarray(done), { at: req.offset + done });
				if (!n) throw new Error('room');
				done += n;
			}
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
			(error: Error) => postMessage({ id, error: error.message, name: error.name })
		)
	);
};
