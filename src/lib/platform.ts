// The one place in the interface that knows which operating system it runs on. Components ask
// here, or read a catalog that has already been adapted (`i18n/mac.ts`), rather than testing
// the system themselves.

/** Whether the interface runs on a Mac, from the webview's user agent (WebKit on macOS). Read
 *  once: the system does not change under a running window. */
export const isMac: boolean =
	typeof navigator !== 'undefined' && /Macintosh|Mac OS X/.test(navigator.userAgent);
