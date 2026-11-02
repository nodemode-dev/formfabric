import { serviceUrl } from '../../engine/service';
import { hasCommand, obj, quiet, run } from '../viz/api';

export const KINDS = ['fabric', 'trim', 'style', 'component', 'garment', 'pose', 'motion', 'place'] as const;
export type Kind = (typeof KINDS)[number];

export const KIND_LABEL: Record<string, string> = {
  all: 'All kinds', fabric: 'Fabric', trim: 'Trim style', style: 'Style line', component: 'Component', garment: 'Garment', pose: 'Pose', motion: 'Motion', place: 'Placement',
};

export interface LibItem { id: string; kind: string; name: string; tags: string[]; thumb: string; root: string }
export interface SnapItem { id: string; name: string; at: number; thumb: string; count: number | null }
export interface PortInfo { name: string; length: number | null; spans: number; valid: boolean }
export interface CompItem { id: string; name: string; panels: number[]; ports: PortInfo[] }
export interface PortRow { name: string; a: number | null; b: number | null; ok: boolean; note: string }
export interface CheckResult { ok: boolean; ease: number | null; eased: boolean; rows: PortRow[]; mismatches: string[]; refused: boolean }

const str = (v: unknown, d = ''): string => (typeof v === 'string' ? v : typeof v === 'number' ? String(v) : d);
const numOr = (v: unknown): number | null => (typeof v === 'number' && Number.isFinite(v) ? v : null);

export function thumbSrc(v: unknown): string {
  const s = str(v);
  if (!s) return '';
  if (/^(data:|https?:|blob:)/.test(s)) return s;
  if (s.startsWith('/api/')) return serviceUrl(s);
  if (/^(\/|[A-Za-z]:[\\/])/.test(s)) return `path:${s}`;
  return /^[A-Za-z0-9+/=]{40,}$/.test(s) ? `data:image/png;base64,${s}` : '';
}

const blobs = new Map<string, string>();
const loading = new Map<string, Promise<string>>();

export const cachedThumb = (src: string): string | undefined => (src.startsWith('path:') ? blobs.get(src) : src);

export function resolveThumb(src: string): Promise<string> {
  const have = cachedThumb(src);
  if (have !== undefined) return Promise.resolve(have);
  const wait = loading.get(src);
  if (wait) return wait;
  const job = (async (): Promise<string> => {
    try {
      const response = await fetch(serviceUrl(`/api/files/image?path=${encodeURIComponent(src.slice(5))}`), { cache: 'force-cache' });
      if (!response.ok) return '';
      const url = URL.createObjectURL(await response.blob());
      blobs.set(src, url);
      if (blobs.size > 400) {
        const oldest = blobs.keys().next().value;
        if (oldest !== undefined) { URL.revokeObjectURL(blobs.get(oldest) ?? ''); blobs.delete(oldest); }
      }
      return url;
    } catch { return ''; } finally { loading.delete(src); }
  })();
  loading.set(src, job);
  return job;
}

function rowsOf(r: unknown, key: string): Record<string, unknown>[] {
  if (Array.isArray(r)) return r.map(obj);
  const o = obj(r);
  const list = o[key] ?? o.items ?? o.list;
  return Array.isArray(list) ? list.map(obj) : [];
}

export function toItem(o: Record<string, unknown>): LibItem {
  const tags = Array.isArray(o.tags) ? o.tags.filter((x): x is string => typeof x === 'string') : [];
  return { id: str(o.id ?? o.path ?? o.name), kind: str(o.kind, 'garment'), name: str(o.name ?? o.id), tags, thumb: thumbSrc(o.thumbnail ?? o.thumb ?? o.thumbnail_url), root: str(o.root) };
}

export function toSnap(o: Record<string, unknown>): SnapItem {
  const raw = o.created ?? o.time ?? o.at ?? o.created_at;
  const at = typeof raw === 'number' ? (raw < 1e12 ? raw * 1000 : raw) : Date.parse(str(raw)) || 0;
  return { id: str(o.id ?? o.name), name: str(o.name ?? o.id), at, thumb: thumbSrc(o.thumbnail_png ?? o.thumb ?? o.thumbnail_url), count: numOr(o.panels ?? o.panel_count) };
}

export function toComp(o: Record<string, unknown>): CompItem {
  const ports = (Array.isArray(o.ports) ? o.ports : []).map(obj).map((p): PortInfo => ({ name: str(p.name), length: numOr(p.length_mm ?? p.length), spans: Array.isArray(p.spans) ? p.spans.length : numOr(p.span_count) ?? 0, valid: p.valid !== false }));
  const panels = Array.isArray(o.panels) ? o.panels.filter((x): x is number => typeof x === 'number') : [];
  return { id: str(o.id ?? o.name), name: str(o.name ?? o.id), panels, ports };
}

export function toCheck(r: unknown): CheckResult {
  const top = obj(r);
  const o = top.check && typeof top.check === 'object' ? obj(top.check) : top;
  const raw = o.ports ?? o.rows ?? o.pairs ?? [];
  const rows = (Array.isArray(raw) ? raw : []).map(obj).map((p): PortRow => {
    const a = numOr(p.expected_mm ?? p.length_a ?? p.a);
    const b = numOr(p.actual_mm ?? p.length_b ?? p.b);
    const found = p.found !== false;
    const ok = typeof p.ok === 'boolean' ? p.ok : found && a !== null && b !== null && Math.abs(a - b) < 1;
    const delta = numOr(p.delta_mm);
    const note = str(p.note ?? p.reason ?? p.error) || (!found ? 'missing' : delta !== null ? `${delta > 0 ? '+' : ''}${delta.toFixed(1)} mm` : '');
    return { name: str(p.port ?? p.name), a, b, ok, note };
  });
  const mismatches = rows.filter((x) => !x.ok).map((x) => x.name);
  const ok = typeof o.compatible === 'boolean' ? o.compatible : typeof o.ok === 'boolean' ? o.ok : mismatches.length === 0;
  const eased = top.ease === true || o.ease === true;
  return { ok, ease: null, eased, rows, mismatches, refused: false };
}

export async function list(kind: string, query: string): Promise<LibItem[] | undefined> {
  if (!hasCommand('library.list')) return undefined;
  const r = await quiet<unknown>('library.list', { ...(kind !== 'all' ? { kind } : {}), ...(query ? { query } : {}) });
  return r === undefined ? undefined : rowsOf(r, 'items').map(toItem);
}

export async function roots(): Promise<string[]> {
  if (!hasCommand('library.roots')) return [];
  const o = await quiet<unknown>('library.roots', {});
  const r = Array.isArray(o) ? o : obj(o).roots;
  return Array.isArray(r) ? r.map((x) => (typeof x === 'string' ? x : str(obj(x).path))).filter(Boolean) : [];
}

export const setRoots = (paths: string[]): Promise<unknown> => run('library.set_roots', { paths, confirmed: true });

const snapThumbs = new Map<string, unknown>();

export async function snapshots(): Promise<SnapItem[] | undefined> {
  if (!hasCommand('snapshot.list')) return undefined;
  const r = await quiet<unknown>('snapshot.list', {});
  if (r === undefined) return undefined;
  const rows = rowsOf(r, 'snapshots');
  const live = new Set<string>();
  for (const row of rows) {
    const key = `${str(row.id)}:${str(row.created)}`;
    live.add(key);
    if (row.thumbnail !== true) continue;
    if (!snapThumbs.has(key)) {
      const one = await quiet<unknown>('snapshot.list', { thumbnails: true, ids: [Number(row.id)] });
      snapThumbs.set(key, rowsOf(one, 'snapshots')[0]?.thumbnail_png ?? '');
    }
    row.thumbnail_png = snapThumbs.get(key);
  }
  for (const key of Array.from(snapThumbs.keys())) if (!live.has(key)) snapThumbs.delete(key);
  return rows.map(toSnap);
}

export async function components(): Promise<CompItem[] | undefined> {
  if (!hasCommand('component.list')) return undefined;
  const r = await quiet<unknown>('component.list', {});
  return r === undefined ? undefined : rowsOf(r, 'components').map(toComp);
}

export function thumbFromView(width: number, height: number): Promise<string | undefined> {
  const detail: { out: HTMLCanvasElement | null; w: number; h: number } = { out: null, w: width, h: height };
  document.dispatchEvent(new CustomEvent('ff:thumb', { detail }));
  const c = detail.out;
  if (!c) return Promise.resolve(undefined);
  return new Promise((resolve) => c.toBlob((blob) => {
    if (!blob) { resolve(undefined); return; }
    const fr = new FileReader();
    fr.onload = () => resolve(typeof fr.result === 'string' ? fr.result.split(',')[1] : undefined);
    fr.onerror = () => resolve(undefined);
    fr.readAsDataURL(blob);
  }, 'image/png'));
}

export interface Recovery { present: boolean; at: number; commands: number }

export async function recoveryInfo(): Promise<Recovery | null> {
  try {
    const response = await fetch(serviceUrl('/api/recovery'), { cache: 'no-store' });
    if (!response.ok) return null;
    const o = obj(await response.json());
    const present = o.pending === true || o.present === true || o.available === true || o.exists === true || o.recoverable === true || o.journal === true || (typeof o.journal === 'object' && o.journal !== null);
    const j = obj(o.journal);
    const raw = o.autosave_time ?? o.at ?? o.time ?? o.saved_at ?? o.autosave_at ?? j.at ?? j.time;
    const at = typeof raw === 'number' ? (raw < 1e12 ? raw * 1000 : raw) : Date.parse(str(raw)) || 0;
    const commands = numOr(o.commands ?? o.command_count ?? o.log ?? j.commands ?? j.command_count) ?? (Array.isArray(o.log) ? o.log.length : 0);
    return { present, at, commands };
  } catch { return null; }
}

export async function recoveryAct(action: 'restore' | 'discard'): Promise<boolean> {
  try {
    const response = await fetch(serviceUrl(`/api/recovery/${action}`), { method: 'POST' });
    return response.ok;
  } catch { return false; }
}
