import { normalizeAppearance, type Appearance } from './appearance';
import { normalizeStartOptions, type StartOptions } from './types';

export { normalizeAppearance, type Appearance };
export const PROFILES_KEY = 'meeting.profiles';
export interface Placement {
	x: number;
	y: number;
	width: number;
	height: number;
}
export interface MeetingProfile {
	id: string;
	name: string;
	options: StartOptions;
	appearance: Appearance;
	placement: Placement | null;
}
export function decodeProfiles(raw: string | null): MeetingProfile[] {
	try {
		const data: unknown = JSON.parse(raw ?? '[]');
		if (!Array.isArray(data)) return [];
		const ids = new Set<string>();
		return data.flatMap((p) => {
			if (
				!p ||
				typeof p.id !== 'string' ||
				ids.has(p.id) ||
				typeof p.name !== 'string' ||
				!p.name.trim()
			)
				return [];
			ids.add(p.id);
			const g = p.placement;
			const valid =
				g &&
				['x', 'y', 'width', 'height'].every((k) => Number.isSafeInteger(g[k])) &&
				Math.abs(g.x) <= 2147483647 &&
				Math.abs(g.y) <= 2147483647 &&
				g.width > 0 &&
				g.height > 0 &&
				g.width <= 16384 &&
				g.height <= 16384;
			return [
				{
					id: p.id,
					name: p.name.trim().slice(0, 80),
					options: normalizeStartOptions(p.options),
					appearance: normalizeAppearance(p.appearance),
					placement: valid ? { x: g.x, y: g.y, width: g.width, height: g.height } : null
				}
			];
		});
	} catch {
		return [];
	}
}
