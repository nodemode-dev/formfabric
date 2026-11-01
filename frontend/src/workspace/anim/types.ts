export type Interp = 'step' | 'linear' | 'smooth';

export interface KeyItem { frame: number; value: unknown; interp: Interp }
export interface TrackItem { id: number; target: string; keys: KeyItem[] }
export interface MotionItem { id: number; name: string; source: string; body: number; start: number; frames: number; fps: number; lead_in: number; retarget: unknown }
export interface CacheItem { id: number; name: string; start: number; frames: number; visible: boolean; entry: string }

export interface AnimState {
  fps: number;
  start: number;
  end: number;
  current: number;
  motions: MotionItem[];
  tracks: TrackItem[];
  caches: CacheItem[];
  active: number | null;
}

export interface JobInfo { running: boolean; progress: number; text: string; failed: string; playing: boolean; playFrame: number | null }

export type RowKind = 'cache' | 'motion' | 'track';
export interface Row { kind: RowKind; id: number }

export const INTERPS: Interp[] = ['step', 'linear', 'smooth'];

export const EMPTY: AnimState = { fps: 30, start: 0, end: 120, current: 0, motions: [], tracks: [], caches: [], active: null };

const rec = (v: unknown): Record<string, unknown> | null => (v && typeof v === 'object' && !Array.isArray(v) ? (v as Record<string, unknown>) : null);
const num = (v: unknown, d: number): number => (typeof v === 'number' && Number.isFinite(v) ? v : d);
const list = (v: unknown): Record<string, unknown>[] => (Array.isArray(v) ? v.map(rec).filter((r): r is Record<string, unknown> => r !== null) : []);

export function parseAnim(raw: unknown): AnimState {
  const top = rec(raw);
  const r = rec(top?.anim) ?? top;
  if (!r) return EMPTY;
  const interp = (v: unknown): Interp => (v === 'step' || v === 'smooth' ? v : 'linear');
  return {
    fps: num(r.fps, 30),
    start: num(r.start, 0),
    end: num(r.end, 120),
    current: num(r.current, 0),
    motions: list(r.motions).map((m) => ({
      id: num(m.id, 0), name: typeof m.name === 'string' ? m.name : '', source: typeof m.source === 'string' ? m.source : '',
      body: num(m.body, 0), start: num(m.start, 0), frames: num(m.frames, 0), fps: num(m.fps, 30), lead_in: num(m.lead_in, 0), retarget: m.retarget,
    })),
    tracks: list(r.tracks).map((tr) => ({
      id: num(tr.id, 0), target: typeof tr.target === 'string' ? tr.target : '',
      keys: list(tr.keys).map((k) => ({ frame: num(k.frame, 0), value: k.value, interp: interp(k.interp) })).sort((a, b) => a.frame - b.frame),
    })),
    caches: list(r.caches).map((c) => ({
      id: num(c.id, 0), name: typeof c.name === 'string' ? c.name : '', start: num(c.start, 0), frames: num(c.frames, 0), visible: c.visible !== false, entry: typeof c.entry === 'string' ? c.entry : '',
    })),
    active: typeof r.active_cache === 'number' ? r.active_cache : null,
  };
}

export function parseJob(raw: unknown): JobInfo {
  const r = rec(raw) ?? {};
  const state = typeof r.phase === 'string' ? r.phase : typeof r.state === 'string' ? r.state : '';
  const running = r.running === true || state === 'running';
  let p = typeof r.progress === 'number' ? r.progress : NaN;
  if (!Number.isFinite(p)) {
    const frame = num(r.frame, NaN);
    const total = num(r.total ?? r.frames, NaN);
    p = Number.isFinite(frame) && Number.isFinite(total) && total > 0 ? frame / total : 0;
  }
  if (p > 1) p /= 100;
  const message = typeof r.message === 'string' ? r.message : '';
  const failed = typeof r.error === 'string' ? r.error : state === 'failed' ? message || state : '';
  return { running, progress: Math.max(0, Math.min(1, p)), text: state, failed, playing: r.playing === true, playFrame: typeof r.play_frame === 'number' ? r.play_frame : null };
}

export function bodyRows(raw: unknown): Array<{ id: number; name: string }> {
  const r = rec(raw);
  return list(r?.bodies).map((b) => ({ id: num(b.id, 0), name: typeof b.name === 'string' ? b.name : String(b.id) }));
}

export function rowsOf(a: AnimState): Row[] {
  return [
    ...a.caches.map((c): Row => ({ kind: 'cache', id: c.id })),
    ...a.motions.map((m): Row => ({ kind: 'motion', id: m.id })),
    ...a.tracks.map((tr): Row => ({ kind: 'track', id: tr.id })),
  ];
}

export function parseValue(text: string): unknown {
  const s = text.trim();
  if (s === '') return undefined;
  const n = Number(s);
  if (Number.isFinite(n)) return n;
  if (/^[[{"]/.test(s) || s === 'true' || s === 'false') { try { return JSON.parse(s); } catch { return s; } }
  return s;
}

export function showValue(v: unknown): string {
  if (v === undefined || v === null) return '';
  return typeof v === 'string' ? v : JSON.stringify(v);
}
