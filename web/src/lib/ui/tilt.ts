import { tilt } from '#lib/marker.ts';

export function tilted(node: HTMLElement, seed: number) {
	node.style.rotate = `${tilt(seed)}deg`;
}
