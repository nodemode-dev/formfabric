import { obj, quiet } from '../viz/api';

export const TABS = [
  ['scene', 'Scene'],
  ['fabric', 'Fabric'],
  ['graphic', 'Graphic'],
  ['button', 'Button'],
  ['buttonhole', 'Buttonhole'],
  ['topstitch', 'Topstitch'],
  ['puckering', 'Puckering'],
  ['trim', 'Trim'],
  ['zipper', 'Zipper'],
] as const;

export type TabId = (typeof TABS)[number][0];

export interface Row {
  key: string;
  id: number;
  kind: string;
  name: string;
  sub: string;
  swatch: string;
  panel: number | null;
}

const num = (v: unknown, d = 0): number => (typeof v === 'number' && Number.isFinite(v) ? v : d);
const str = (v: unknown, d = ''): string => (typeof v === 'string' ? v : d);
const list = (v: unknown): Record<string, unknown>[] => (Array.isArray(v) ? v.map(obj) : []);

const hex = (c: unknown): string => {
  if (typeof c === 'string') return /^#[0-9a-f]{6}$/i.test(c) ? c : '';
  const a = Array.isArray(c) ? c.filter((x): x is number => typeof x === 'number') : [];
  if (a.length < 3) return '';
  const k = a.some((x) => x > 1) ? 1 : 255;
  return `#${a.slice(0, 3).map((x) => Math.round(Math.max(0, Math.min(255, x * k))).toString(16).padStart(2, '0')).join('')}`;
};

export const SCENE_KINDS: Record<string, string> = { panel: 'Panel', seam: 'Seam', trim: 'Trim', body: 'Body', style: 'Style' };

export async function loadScene(): Promise<Row[] | undefined> {
  const r = await quiet<unknown>('scene.tree', {});
  if (r === undefined) return undefined;
  return list(obj(r).items).map((o) => {
    const kind = str(o.kind);
    return { key: str(o.key), id: num(o.id), kind, name: str(o.name), sub: kind, swatch: '', panel: kind === 'panel' ? num(o.id) : null };
  });
}

export async function loadFabric(): Promise<Row[] | undefined> {
  const r = await quiet<unknown>('fabric.list', {});
  if (r === undefined) return undefined;
  const o = obj(r);
  return list(o.fabrics).map((f) => {
    const id = num(f.id);
    const used = Array.isArray(f.panels) ? f.panels.length : 0;
    return { key: `fabric:${id}`, id, kind: 'fabric', name: str(f.name, `Fabric ${id}`), sub: String(used), swatch: hex(f.color), panel: null };
  });
}

export async function loadStyles(kind: string): Promise<Row[] | undefined> {
  const r = await quiet<unknown>('style.list', { kind });
  if (r === undefined) return undefined;
  return list(obj(r).styles).map((s) => {
    const id = num(s.id);
    const u = obj(s.used_by);
    const used = num(u.hardware) + num(u.zippers) + num(u.stitches);
    return { key: `style:${id}`, id, kind, name: str(s.name, `${kind} ${id}`), sub: String(used), swatch: '', panel: null };
  });
}

export function loadGraphics(garment: unknown): Row[] {
  const panels = list(obj(garment).panels);
  const out: Row[] = [];
  for (const p of panels) {
    const pid = num(p.id);
    for (const g of list(p.prints)) {
      const id = num(g.id);
      out.push({ key: `print:${pid}:${id}`, id, kind: 'graphic', name: str(g.name) || str(g.image).split(/[\\/]/).pop() || `Graphic ${id}`, sub: str(p.name, `Panel ${pid}`), swatch: '', panel: pid });
    }
  }
  return out;
}

export function sorted(rows: Row[], dir: 0 | 1 | -1): Row[] {
  if (dir === 0) return rows;
  const out = rows.slice();
  out.sort((a, b) => dir * a.name.localeCompare(b.name, undefined, { sensitivity: 'base' }));
  return out;
}
