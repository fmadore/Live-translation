// GitHub Pages publishes `docs/` and nothing above it (issue #252). A Markdown link that climbs
// out of the folder — `](../ROADMAP.md)` from `docs/README.md` — works on github.com, which
// renders the whole repository, and is a 404 on the published site. Those links were fixed by
// hand once; this keeps them from coming back. A file outside `docs/` is linked by its GitHub
// URL instead (`https://github.com/fmadore/Live-translation/blob/main/ROADMAP.md`).
//
// The public home page, `docs/index.html`, is held to more: every relative link and image in it
// must resolve to a file in `docs/`, and every in-page anchor to an id on the page.

import { describe, expect, it } from 'vitest';
import { existsSync, readFileSync, readdirSync, statSync } from 'node:fs';
import { dirname, join, relative, resolve, sep } from 'node:path';

const DOCS = join(process.cwd(), 'docs');

function markdownFiles(dir: string): string[] {
	return readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
		const path = join(dir, entry.name);
		if (entry.isDirectory()) return markdownFiles(path);
		return entry.name.endsWith('.md') ? [path] : [];
	});
}

/** A link to another site, a mail address or a place on the same page: nothing to resolve. */
function isExternal(target: string): boolean {
	return /^[a-z][a-z\d+.-]*:/i.test(target) || target.startsWith('//') || target.startsWith('#');
}

/** The file part of a link, without its fragment or query, decoded. */
function pathOf(target: string): string {
	return decodeURIComponent(target.split('#')[0].split('?')[0]);
}

/** Whether a link, written in `file`, points outside `docs/`. A root-relative link (`/x`) does
 *  too: on Pages the root is the account's site, not this repository's. */
function leavesDocs(file: string, target: string): boolean {
	if (isExternal(target)) return false;
	const path = pathOf(target);
	if (path === '') return false;
	if (path.startsWith('/')) return true;
	const rel = relative(DOCS, resolve(dirname(file), path));
	return rel === '..' || rel.startsWith(`..${sep}`);
}

/** Link targets in Markdown: inline links and images, reference definitions, and the `href` and
 *  `src` of inline HTML. Code is left out, since an example there is not a link. */
function markdownTargets(source: string): string[] {
	const prose = source.replace(/^(```|~~~)[\s\S]*?^\1/gm, '').replace(/`[^`\n]*`/g, '');
	const inline = [...prose.matchAll(/\]\(\s*<?([^)\s>]+)>?(?:\s+["'(][^)]*)?\)/g)].map((m) => m[1]);
	const references = [...prose.matchAll(/^\s{0,3}\[[^\]]+\]:\s*<?(\S+?)>?(?:\s|$)/gm)].map(
		(m) => m[1]
	);
	const html = [...prose.matchAll(/\b(?:href|src)\s*=\s*["']([^"']+)["']/g)].map((m) => m[1]);
	return [...inline, ...references, ...html];
}

describe('Markdown links in docs/', () => {
	it('finds the documentation it checks', () => {
		expect(markdownFiles(DOCS).length).toBeGreaterThan(20);
	});

	it('never leave docs/, which is all GitHub Pages publishes', () => {
		const leaving = markdownFiles(DOCS).flatMap((file) =>
			markdownTargets(readFileSync(file, 'utf8'))
				.filter((target) => leavesDocs(file, target))
				.map((target) => `${relative(DOCS, file).split(sep).join('/')}: ${target}`)
		);
		expect(leaving).toEqual([]);
	});

	it('are found in each form the check reads', () => {
		const file = join(DOCS, 'example.md');
		const source = [
			'See the [roadmap](../ROADMAP.md "Roadmap") and ![art](../art.png).',
			'[security]: ../SECURITY.md',
			'<a href="../LICENSE">licence</a>',
			'Stays: [privacy](privacy.md#contact), [site](https://example.com/../x), [top](#top).',
			'Code is not a link: `[x](../code.md)`',
			'```md',
			'[y](../fence.md)',
			'```'
		].join('\n');
		const leaving = markdownTargets(source).filter((target) => leavesDocs(file, target));
		expect(leaving).toEqual(['../ROADMAP.md', '../art.png', '../SECURITY.md', '../LICENSE']);
		expect(leavesDocs(join(DOCS, 'decisions', 'x.md'), '../macos.md')).toBe(false);
		expect(leavesDocs(file, '/Live-translation/privacy')).toBe(true);
	});
});

describe('the public home page, docs/index.html', () => {
	const page = readFileSync(join(DOCS, 'index.html'), 'utf8');
	// Attributes only, so the comment at the top of the file, which names paths, is not read.
	const body = page.replace(/<!--[\s\S]*?-->/g, '');
	const targets = [
		...[...body.matchAll(/\b(?:href|src)\s*=\s*"([^"]*)"/g)].map((m) => m[1]),
		...[...body.matchAll(/url\(\s*'([^']+)'\s*\)/g)].map((m) => m[1])
	];
	const ids = new Set([...body.matchAll(/\bid\s*=\s*"([^"]+)"/g)].map((m) => m[1]));

	/** What Pages serves for a relative link: the file itself, the Markdown page Jekyll renders
	 *  as `name.html`, or a folder's index. */
	function served(target: string): boolean {
		const path = resolve(DOCS, pathOf(target));
		if (existsSync(path) && statSync(path).isFile()) return true;
		if (path.endsWith('.html') && existsSync(path.replace(/\.html$/, '.md'))) return true;
		return ['index.html', 'README.md'].some((index) => existsSync(join(path, index)));
	}

	it('links to files that are published with it', () => {
		const relativeTargets = targets.filter((target) => !isExternal(target) && pathOf(target));
		expect(relativeTargets.length).toBeGreaterThan(10);
		expect(
			relativeTargets.filter((target) => leavesDocs(join(DOCS, 'index.html'), target))
		).toEqual([]);
		expect(relativeTargets.filter((target) => !served(target))).toEqual([]);
	});

	it('links within the page to ids that exist', () => {
		const anchors = targets.filter((target) => target.startsWith('#') && target.length > 1);
		expect(anchors.length).toBeGreaterThan(0);
		expect(anchors.filter((anchor) => !ids.has(anchor.slice(1)))).toEqual([]);
	});

	it('describes every image', () => {
		const images = [...body.matchAll(/<img\b[^>]*>/g)].map((m) => m[0]);
		expect(images.length).toBeGreaterThan(0);
		expect(images.filter((image) => !/\balt="[^"]+"/.test(image))).toEqual([]);
	});

	// The Mac App Store build hides the support link (App Review Guideline 3.1.1, see
	// src-tauri/src/support.rs), and the Stores may link to this page, so it must not become a
	// way round that.
	it('carries no tip or payment link', () => {
		expect(body).not.toMatch(/buymeacoffee|ko-fi|paypal|patreon/i);
	});
});
