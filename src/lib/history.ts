import { get, writable } from 'svelte/store';
import { decodeRecovery } from './document';
import { api } from './tauri';
import type { StartOptions, TranscriptLine } from './types';

export const HISTORY_ENABLED_KEY = 'transcript.historyEnabled';
export const historyEnabled = writable(
	typeof localStorage !== 'undefined' && localStorage.getItem(HISTORY_ENABLED_KEY) === 'true'
);
historyEnabled.subscribe((value) => {
	if (typeof localStorage !== 'undefined') localStorage.setItem(HISTORY_ENABLED_KEY, String(value));
});
export const historyError = writable('');
export const historyRevision = writable(0);

export interface SavedSession {
	version: 1;
	id: string;
	startedAt: string;
	savedAt: string;
	endedAt: string | null;
	durationMs: number;
	mode: 'translate' | 'transcribe';
	sourceLanguage: 'auto' | 'en' | 'fr';
	targetLanguage: 'en' | 'fr' | null;
	lines: TranscriptLine[];
}

export function decodeSession(raw: string, id: string): SavedSession | null {
	try {
		const s = JSON.parse(raw);
		const recovered = decodeRecovery(raw);
		if (
			!recovered ||
			s.id !== id ||
			!/^[a-f\d]{8}(?:-[a-f\d]{4}){3}-[a-f\d]{12}$/i.test(id) ||
			!['translate', 'transcribe'].includes(s.mode) ||
			!['auto', 'en', 'fr'].includes(s.sourceLanguage) ||
			![null, 'en', 'fr'].includes(s.targetLanguage) ||
			typeof s.startedAt !== 'string' ||
			!Number.isFinite(Date.parse(s.startedAt)) ||
			(s.endedAt !== null &&
				(typeof s.endedAt !== 'string' || !Number.isFinite(Date.parse(s.endedAt)))) ||
			!Number.isFinite(s.durationMs) ||
			s.durationMs < 0 ||
			recovered.lines.length !== s.lines.length ||
			new Set(recovered.lines.map((line) => line.id)).size !== recovered.lines.length ||
			recovered.lines.some((line) => !Number.isSafeInteger(line.id) || line.id < 1)
		)
			return null;
		return {
			version: 1,
			id,
			startedAt: s.startedAt,
			savedAt: recovered.savedAt,
			endedAt: s.endedAt,
			durationMs: s.durationMs,
			mode: s.mode,
			sourceLanguage: s.sourceLanguage,
			targetLanguage: s.targetLanguage,
			lines: recovered.lines
		};
	} catch {
		return null;
	}
}

/** Serialize writes and deletions. Pending snapshots coalesce, but every completed write
 * is atomic. A deleted active session stays deleted; the next Start gets a fresh UUID. */
export function createHistoryCoordinator(
	port: Pick<typeof api, 'writeHistory' | 'deleteHistory'>,
	enabled: () => boolean,
	onError: (error: unknown) => void = () => {},
	onSaved: (failed: boolean) => void = () => {}
) {
	let active: SavedSession | null = null;
	let queue = Promise.resolve();
	let revision = 0;
	const deleted = new Set<string>();
	const failed = new Map<string, SavedSession>();
	async function write(snapshot: SavedSession) {
		if (deleted.has(snapshot.id)) return;
		try {
			await port.writeHistory(snapshot.id, JSON.stringify(snapshot));
			failed.delete(snapshot.id);
			onSaved(failed.size > 0);
		} catch (error) {
			failed.set(snapshot.id, snapshot);
			onError(error);
		}
	}
	function persist() {
		if (!enabled() || !active?.lines.length || deleted.has(active.id)) return;
		const snapshot = { ...active, lines: [...active.lines] };
		const version = ++revision;
		queue = queue
			.then(async () => {
				if (deleted.has(snapshot.id) || (active?.id === snapshot.id && version !== revision))
					return;
				await write(snapshot);
			})
			.catch(onError);
	}
	return {
		begin(options: StartOptions, now = new Date(), id = crypto.randomUUID()) {
			active = {
				version: 1,
				id,
				startedAt: now.toISOString(),
				savedAt: now.toISOString(),
				endedAt: null,
				durationMs: 0,
				mode: options.mode,
				sourceLanguage:
					options.rehearsal ?? (options.provider === 'ondevice' ? options.targetLanguage : 'auto'),
				targetLanguage: options.mode === 'translate' ? options.targetLanguage : null,
				lines: []
			};
		},
		append(line: TranscriptLine, now = new Date()) {
			if (!active || !enabled() || deleted.has(active.id)) return;
			active.lines = [line, ...active.lines];
			active.savedAt = now.toISOString();
			active.durationMs = Math.max(0, now.getTime() - Date.parse(active.startedAt));
			persist();
		},
		finish(now = new Date()) {
			if (active && active.endedAt === null) {
				active.endedAt = now.toISOString();
				active.savedAt = now.toISOString();
				active.durationMs = Math.max(0, now.getTime() - Date.parse(active.startedAt));
				persist();
			}
			return queue;
		},
		retry() {
			if (!enabled()) return queue;
			queue = queue.then(async () => {
				for (const snapshot of [...failed.values()]) await write(snapshot);
			});
			persist();
			return queue;
		},
		flush() {
			return queue;
		},
		delete(id: string) {
			deleted.add(id);
			const result = queue.then(async () => {
				await port.deleteHistory(id);
				failed.delete(id);
				onSaved(failed.size > 0);
			});
			queue = result.catch(onError);
			return result;
		}
	};
}

export const sessionHistory = createHistoryCoordinator(
	api,
	() => get(historyEnabled),
	(error) => historyError.set(String(error)),
	(failed) => {
		if (!failed) historyError.set('');
		historyRevision.update((n) => n + 1);
	}
);
