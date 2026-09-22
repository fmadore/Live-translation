import { afterEach, expect, it, vi } from 'vitest';
import { render, waitFor } from '@testing-library/svelte';
import OverlayCaptionLine from './OverlayCaptionLine.svelte';

afterEach(() => {
	vi.restoreAllMocks();
	vi.unstubAllGlobals();
});

it('holds earlier rows until a full line overflows and refits whole rows after resizing', async () => {
	// jsdom has no text layout: provide measured browser dimensions and trigger the
	// same ResizeObserver notifications Svelte receives as streamed text wraps.
	const observers: { callback: ResizeObserverCallback; elements: Element[] }[] = [];
	vi.stubGlobal(
		'ResizeObserver',
		class {
			entry: (typeof observers)[number];
			constructor(callback: ResizeObserverCallback) {
				this.entry = { callback, elements: [] };
				observers.push(this.entry);
			}
			observe(element: Element) {
				this.entry.elements.push(element);
			}
			unobserve() {}
			disconnect() {}
		}
	);
	let rows = 2;
	vi.spyOn(HTMLElement.prototype, 'clientWidth', 'get').mockReturnValue(400);
	vi.spyOn(HTMLElement.prototype, 'clientHeight', 'get').mockImplementation(function (
		this: HTMLElement
	) {
		return this.tagName === 'P' ? rows * 40 : 120;
	});
	vi.spyOn(window, 'getComputedStyle').mockReturnValue({
		lineHeight: '40px'
	} as CSSStyleDeclaration);
	const props = {
		stable: true,
		lead: 'First sentence.',
		text: 'Next words',
		interim: true,
		height: 125,
		fontKey: '40',
		language: 'en'
	};
	const view = render(OverlayCaptionLine, props);
	function resize() {
		for (const observer of observers)
			observer.callback(
				observer.elements.map(
					(target) =>
						({ target, contentRect: { width: 400, height: rows * 40 } }) as ResizeObserverEntry
				),
				{} as ResizeObserver
			);
	}
	resize();
	const paragraph = view.container.querySelector('.stable-viewport p') as HTMLElement;
	await waitFor(() => expect(paragraph.style.transform).toBe('translateY(-0px)'));
	await view.rerender({ ...props, text: 'Next words arrive' });
	rows = 3;
	resize();
	await waitFor(() => expect(paragraph.style.transform).toBe('translateY(-0px)'));
	rows = 4;
	resize();
	await waitFor(() => expect(paragraph.style.transform).toBe('translateY(-40px)'));
	await view.rerender({ ...props, height: 85 });
	await waitFor(() => expect(paragraph.style.transform).toBe('translateY(-80px)'));
	expect(paragraph.textContent).toContain('First sentence.');
	expect(view.container.querySelector('.caret')).toBeNull();
	await view.rerender({ ...props, text: 'مرحبا بكم', language: 'ar' });
	expect(view.container.querySelector('.text-region')).toHaveAttribute('dir', 'rtl');
	expect(view.container.querySelector('.text-region')).toHaveAttribute('lang', 'ar');
	await view.rerender({ ...props, stable: false, text: 'こんにちは', language: 'ja' });
	expect(view.container.querySelector('.text-region')).toHaveAttribute('dir', 'ltr');
	expect(view.container.querySelector('.text-region')).toHaveAttribute('lang', 'ja');
	view.unmount();
});
