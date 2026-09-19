export type CaptionLayout = 'fit' | 'compact' | 'stable';
export const CAPTION_LAYOUT_KEY = 'overlay.captionLayout';

/** Bottom alignment is a subtitle strip; manually resizing still permits more context. */
export function bottomCaptionHeight(
	fontSize: number,
	origins: number,
	layout: CaptionLayout
): number {
	const rows = Math.max(1, Math.min(2, origins));
	const textHeight = 2 * fontSize * 1.34 * rows + 18 * (rows - 1);
	return Math.max(160, Math.ceil(layout === 'compact' ? textHeight / 0.85 : textHeight + 32));
}

export function isCaptionLayout(value: unknown): value is CaptionLayout {
	return value === 'fit' || value === 'compact' || value === 'stable';
}

export function loadCaptionLayout(): CaptionLayout {
	if (typeof localStorage === 'undefined') return 'fit';
	const value = localStorage.getItem(CAPTION_LAYOUT_KEY);
	return isCaptionLayout(value) ? value : 'fit';
}

/** Bounded reading context, separate from the full session transcript. */
export function appendCaptionHistory(history: string, text: string): string {
	return `${history} ${text}`.replace(/\s+/g, ' ').trim().slice(-12000);
}

/** Keep the newest words using the actual rendered height, including the live caret. */
export function fitCaptionTail(text: string, fits: (candidate: string) => boolean): string {
	const normalized = text.replace(/\s+/g, ' ').trim();
	if (!normalized || fits(normalized)) return normalized;
	const words = normalized.split(' ');
	let low = 1;
	let high = words.length;
	while (low < high) {
		const mid = Math.floor((low + high) / 2);
		if (fits('… ' + words.slice(mid).join(' '))) high = mid;
		else low = mid + 1;
	}
	if (low < words.length) return '… ' + words.slice(low).join(' ');
	// A single long token can exceed the region. Keep its newest Unicode code points.
	const chars = Array.from(words.at(-1) ?? '');
	low = 0;
	high = chars.length;
	while (low < high) {
		const mid = Math.floor((low + high) / 2);
		if (fits('… ' + chars.slice(mid).join(''))) high = mid;
		else low = mid + 1;
	}
	const result = '… ' + chars.slice(low).join('');
	return fits(result) ? result : '';
}
