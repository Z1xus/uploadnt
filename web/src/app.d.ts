declare global {
	interface Window {
		showSaveFilePicker(options?: { suggestedName?: string }): Promise<FileSystemFileHandle>;
		showDirectoryPicker(options?: { mode?: 'read' | 'readwrite' }): Promise<FileSystemDirectoryHandle>;
	}
}

export {};
