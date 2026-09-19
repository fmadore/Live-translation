import { describe, expect, it, vi } from 'vitest';
import { createHistoryCoordinator, decodeSession } from './history';
import type { StartOptions, TranscriptLine } from './types';

const options: StartOptions = {
	source: 'system',
	mode: 'translate',
	targetLanguage: 'fr',
	provider: 'gemini'
};
const start = new Date('2026-09-19T10:00:00Z');
const id = '12345678-1234-1234-1234-123456789abc';
const id2 = '12345678-1234-1234-1234-123456789abd';
const line: TranscriptLine = {
	id: 1,
	text: 'Um, hello.',
	sourceText: 'Bonjour.',
	origin: 'system',
	startMs: 0,
	endMs: 1000
};

function setup(enabled = true) {
	const disk = new Map<string, string>();
	const port = {
		writeHistory: vi.fn(async (id: string, raw: string) => {
			disk.set(id, raw);
		}),
		deleteHistory: vi.fn(async (id: string) => {
			disk.delete(id);
		})
	};
	const error = vi.fn();
	const history = createHistoryCoordinator(port, () => enabled, error);
	return {
		history,
		port,
		disk,
		error,
		setEnabled: (value: boolean) => {
			enabled = value;
		}
	};
}

describe('session history', () => {
	it('writes nothing by default or for an empty session', async () => {
		const s = setup(false);
		s.history.begin(options, start, id);
		s.history.append(line);
		await s.history.finish();
		expect(s.disk.size).toBe(0);
		s.setEnabled(true);
		s.history.begin(options, start, id2);
		await s.history.finish();
		expect(s.disk.size).toBe(0);
	});
	it('persists raw finalized lines progressively and records duration and language honestly', async () => {
		const s = setup();
		s.history.begin(options, start, id);
		s.history.append(line, new Date(+start + 1500));
		await s.history.flush();
		const running = decodeSession(s.disk.get(id)!, id)!;
		expect(running).toMatchObject({
			sourceLanguage: 'auto',
			targetLanguage: 'fr',
			durationMs: 1500,
			endedAt: null,
			lines: [line]
		});
		await s.history.finish(new Date(+start + 60000));
		expect(decodeSession(s.disk.get(id)!, id)).toMatchObject({
			durationMs: 60000,
			endedAt: '2026-09-19T10:01:00.000Z'
		});
	});
	it('keeps consecutive sessions separate, including queued writes', async () => {
		const s = setup();
		s.history.begin(options, start, id);
		s.history.append(line);
		void s.history.finish();
		s.history.begin({ ...options, mode: 'transcribe', provider: 'ondevice' }, start, id2);
		s.history.append({ ...line, id: 2, text: 'Second' });
		await s.history.finish();
		expect(decodeSession(s.disk.get(id)!, id)?.lines).toEqual([line]);
		expect(decodeSession(s.disk.get(id2)!, id2)).toMatchObject({
			sourceLanguage: 'fr',
			targetLanguage: null,
			lines: [{ text: 'Second' }]
		});
	});
	it('serializes deletion after in-flight writes and prevents resurrection', async () => {
		const s = setup();
		let release!: () => void;
		s.port.writeHistory.mockImplementationOnce(async (i, raw) => {
			await new Promise<void>((r) => (release = r));
			s.disk.set(i, raw);
		});
		s.history.begin(options, start, id);
		s.history.append(line);
		await Promise.resolve();
		s.history.append({ ...line, id: 2 });
		const deletion = s.history.delete(id);
		release();
		await deletion;
		s.history.append({ ...line, id: 3 });
		await s.history.finish();
		expect(s.disk.size).toBe(0);
	});
	it('reports a failed save, retries, and preserves earlier sessions when disabled', async () => {
		const s = setup();
		s.port.writeHistory.mockRejectedValueOnce(new Error('disk full'));
		s.history.begin(options, start, id);
		s.history.append(line);
		await s.history.flush();
		expect(s.error).toHaveBeenCalledOnce();
		expect(s.disk.size).toBe(0);
		await s.history.retry();
		s.setEnabled(false);
		s.history.append({ ...line, id: 2 });
		await s.history.finish();
		expect(decodeSession(s.disk.get(id)!, id)?.lines).toEqual([line]);
	});
	it('rejects corrupt or mismatched sessions while retaining other readable records', async () => {
		const s = setup();
		s.history.begin(options, start, id);
		s.history.append(line);
		await s.history.flush();
		const raw = s.disk.get(id)!;
		expect(decodeSession(raw, id)).not.toBeNull();
		expect(decodeSession(raw, id2)).toBeNull();
		for (const bad of [
			'{',
			JSON.stringify({ ...JSON.parse(raw), version: 9 }),
			JSON.stringify({ ...JSON.parse(raw), durationMs: -1 }),
			JSON.stringify({ ...JSON.parse(raw), lines: [null] })
		])
			expect(decodeSession(bad, id)).toBeNull();
	});
	it('does not extend a completed session when a later start or quit finishes it again', async () => {
		const s = setup();
		s.history.begin(options, start, id);
		s.history.append(line, start);
		await s.history.finish(new Date(+start + 60000));
		await s.history.finish(new Date(+start + 3600000));
		expect(decodeSession(s.disk.get(id)!, id)?.durationMs).toBe(60000);
	});
	it('can retry an earlier failed session after a new session has begun', async () => {
		const s = setup();
		s.port.writeHistory.mockRejectedValueOnce(new Error('disk full'));
		s.history.begin(options, start, id);
		s.history.append(line);
		await s.history.finish();
		s.history.begin(options, start, id2);
		s.history.append({ ...line, id: 2 });
		await s.history.finish();
		await s.history.retry();
		expect(s.disk.size).toBe(2);
	});
});
