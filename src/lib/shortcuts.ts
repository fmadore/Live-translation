export type Shortcut = 'toggleSession' | 'toggleOverlay' | 'larger' | 'smaller' | 'direction';
/** App-local shortcuts. Never intercept typing, repeats, IME or a dialog's own keys. */
export function shortcut(event: KeyboardEvent, dialogOpen: boolean): Shortcut | null {
	if (
		dialogOpen ||
		event.defaultPrevented ||
		event.repeat ||
		event.isComposing ||
		event.altKey ||
		event.metaKey
	)
		return null;
	const target = event.target;
	if (
		target instanceof Element &&
		target.closest('input,textarea,select,[contenteditable="true"],[role="dialog"]')
	)
		return null;
	if (!event.ctrlKey && !event.shiftKey && event.key === 'F2') return 'direction';
	if (!event.ctrlKey || !event.shiftKey) return null;
	if (event.code === 'Space') return 'toggleSession';
	if (event.code === 'KeyO') return 'toggleOverlay';
	if (event.code === 'ArrowUp') return 'larger';
	if (event.code === 'ArrowDown') return 'smaller';
	return null;
}
