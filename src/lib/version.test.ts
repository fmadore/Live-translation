import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';

// A release writes its version by hand into six files (step 1 of the release route in
// `docs/store-updates.md`), and nothing in the build reads them all: a lock file left behind
// turns up later as a stray diff in someone else's change, and a stale `CITATION.cff` has the
// release cited as the one before it. `package.json` is the reference. Each other file is
// read where its own tool writes the app's version, so a dependency's is never taken for it.

const read = (path: string) => readFileSync(new URL(`../../${path}`, import.meta.url), 'utf8');

/** The fields read from `package.json`, `package-lock.json` and `tauri.conf.json`. */
interface Manifest {
	version?: string;
	packages?: Record<string, { version?: string } | undefined>;
}
const manifest = (path: string) => JSON.parse(read(path)) as Manifest;

/** Each TOML table's header and its `key = "string"` lines: enough for the two Cargo files,
 *  where every value read here is a plain string on a line of its own. */
function tomlTables(source: string): { header: string; values: Map<string, string> }[] {
	const tables: { header: string; values: Map<string, string> }[] = [];
	for (const line of source.split(/\r?\n/)) {
		if (line.startsWith('[')) tables.push({ header: line.trim(), values: new Map() });
		const pair = /^([\w-]+)\s*=\s*"([^"]*)"\s*$/.exec(line);
		if (pair) tables.at(-1)?.values.set(pair[1], pair[2]);
	}
	return tables;
}

const VERSION = manifest('package.json').version;

const SOURCES: [file: string, where: string, version: () => string | undefined][] = [
	['package-lock.json', 'version', () => manifest('package-lock.json').version],
	[
		'package-lock.json',
		'packages[""].version',
		() => manifest('package-lock.json').packages?.['']?.version
	],
	[
		'src-tauri/Cargo.toml',
		'[package] version',
		() =>
			tomlTables(read('src-tauri/Cargo.toml'))
				.find((t) => t.header === '[package]')
				?.values.get('version')
	],
	[
		'src-tauri/Cargo.lock',
		'the live-translation package',
		() =>
			tomlTables(read('src-tauri/Cargo.lock'))
				.find((t) => t.header === '[[package]]' && t.values.get('name') === 'live-translation')
				?.values.get('version')
	],
	['src-tauri/tauri.conf.json', 'version', () => manifest('src-tauri/tauri.conf.json').version],
	// A top-level key only: an entry under `references:` is indented, and has its own version.
	[
		'CITATION.cff',
		'version',
		() => /^version:\s*"?([^"\s]+)"?\s*$/m.exec(read('CITATION.cff'))?.[1]
	]
];

describe('the app version', () => {
	it('is set in package.json', () => {
		expect(VERSION, 'package.json has no version').toBeTypeOf('string');
	});

	it.each(SOURCES)('%s (%s) matches package.json', (file, where, version) => {
		const found = version();
		expect(found, `${file} has no ${where}`).toBeDefined();
		expect(found, `${file} (${where}) says ${found}, package.json says ${VERSION}`).toBe(VERSION);
	});
});
