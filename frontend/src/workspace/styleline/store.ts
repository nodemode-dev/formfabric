import { call, onEngineChanged } from '../../engine/service';
import type { DocStore } from '../doc';
import { dist } from '../geom';
import type { Vec } from '../types';
import { hasCommand } from '../viz/api';

export type LineMode = 'none' | 'stiffen' | 'fold';

export interface StyleLineInfo {
  id: number;
  panel: number;
  closed: boolean;
  mode: LineMode;
  stiffen: number;
  width: number;
  fold_angle: number;
  fold_strength: number;
  points: Vec[];
  length: number;
}

const num = (v: unknown, fallback: number): number => (typeof v === 'number' && Number.isFinite(v) ? v : fallback);

function parse(raw: unknown): StyleLineInfo | null {
  if (!raw || typeof raw !== 'object') return null;
  const r = raw as Record<string, unknown>;
  if (typeof r.id !== 'number' || typeof r.panel !== 'number' || !Array.isArray(r.points)) return null;
  const points: Vec[] = [];
  for (const p of r.points) if (Array.isArray(p) && p.length >= 2) points.push([num(p[0], 0), num(p[1], 0)]);
  const mode: LineMode = r.mode === 'stiffen' || r.mode === 'fold' ? r.mode : 'none';
  return {
    id: r.id, panel: r.panel, closed: r.closed === true, mode, points,
    stiffen: num(r.stiffen, 8), width: num(r.width, 12), fold_angle: num(r.fold_angle, 150), fold_strength: num(r.fold_strength, 0.5), length: num(r.length, 0),
  };
}

export function segmentDistance(a: Vec, b: Vec, w: Vec): number {
  const dx = b[0] - a[0];
  const dy = b[1] - a[1];
  const l2 = dx * dx + dy * dy;
  const t = l2 > 0 ? Math.min(1, Math.max(0, ((w[0] - a[0]) * dx + (w[1] - a[1]) * dy) / l2)) : 0;
  return dist([a[0] + dx * t, a[1] + dy * t], w);
}

export class StyleLineStore {
  lines: StyleLineInfo[] = [];
  selected: number | null = null;
  private doc: DocStore | null = null;
  private key = '';
  private busy = false;
  private again = false;
  private readonly listeners = new Set<() => void>();

  attach(doc: DocStore): void {
    if (this.doc) return;
    this.doc = doc;
    doc.onChange(() => this.poll());
    onEngineChanged((e) => { if (/^(styleline|history|file|sample)\./.test(e.name)) this.poll(true); });
    this.poll(true);
  }

  onChange(cb: () => void): () => void {
    this.listeners.add(cb);
    return () => { this.listeners.delete(cb); };
  }

  get available(): boolean { return hasCommand('styleline.list'); }

  get current(): StyleLineInfo | null {
    return this.lines.find((l) => l.id === this.selected) ?? null;
  }

  select(id: number | null): void {
    if (this.selected === id) return;
    this.selected = id;
    this.emit();
  }

  hit(world: Vec, toWorld: (panelId: number, p: Vec) => Vec, radius: number): StyleLineInfo | null {
    let best: StyleLineInfo | null = null;
    let bd = radius;
    for (const l of this.lines) {
      const pts = l.points.map((p) => toWorld(l.panel, p));
      const n = l.closed ? pts.length : pts.length - 1;
      for (let i = 0; i < n; i++) {
        const d = segmentDistance(pts[i], pts[(i + 1) % pts.length], world);
        if (d < bd) { bd = d; best = l; }
      }
    }
    return best;
  }

  private poll(force = false): void {
    if (!this.doc) return;
    const g = this.doc.garment as unknown as { style_lines?: unknown; selection?: { edit3d2?: { lines?: unknown } } };
    const key = JSON.stringify([g.style_lines ?? null, g.selection?.edit3d2?.lines ?? null]);
    if (!force && key === this.key) return;
    this.key = key;
    void this.pull();
  }

  private async pull(): Promise<void> {
    if (this.busy) { this.again = true; return; }
    if (!this.available) return;
    this.busy = true;
    try {
      const r = await call<{ lines?: unknown[] }>('styleline.list', {});
      const next = (r.lines ?? []).map(parse).filter((x): x is StyleLineInfo => x !== null);
      const same = JSON.stringify(next) === JSON.stringify(this.lines);
      this.lines = next;
      if (this.selected !== null && !next.some((l) => l.id === this.selected)) this.selected = null;
      if (!same) this.emit();
    } catch { /* the engine may lack style lines */ } finally {
      this.busy = false;
      if (this.again) { this.again = false; void this.pull(); }
    }
  }

  private emit(): void {
    for (const cb of this.listeners) cb();
  }
}

export const styleLines = new StyleLineStore();
