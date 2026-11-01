import { setClass, setHidden, setText, setTitle } from '../../ui/dom';
import { anyDialogOpen } from '../../ui/dialogs';
import { t } from '../../ui/i18n';
import { textFieldFocused } from '../../ui/native-menu';
import { fmtLen } from '../state';
import type { Frame } from '../view3d/frame';
import type { Scene } from '../view3d/scene';
import { hasCommand, missing, num, obj, quiet, run } from './api';
import type { VizHost, WorldHit } from './host';
import { Pop } from './pop';
import type { MeasureJson } from './types';

export type MeasureTool = 'none' | 'length' | 'circumference' | 'distance';

const TOOLS: Array<[MeasureTool, string]> = [
  ['length', 'Length'],
  ['circumference', 'Circumference'],
  ['distance', 'Distance'],
];

const HINTS: Record<string, string> = {
  length: 'Click points along the garment, Enter finishes, Esc cancels',
  circumference: 'Click a point to measure the girth at its height',
  distance: 'Click two points',
};

interface Slice {
  version: number;
  y: number;
  segs: Float32Array;
  count: number;
  total: number;
}

const a = new Float64Array(3);
const b = new Float64Array(3);

function triples(v: unknown): Array<[number, number, number]> {
  if (!Array.isArray(v)) return [];
  return v.filter((p): p is number[] => Array.isArray(p) && p.length >= 3).map((p) => [Number(p[0]), Number(p[1]), Number(p[2])]);
}

function fromEngine(o: Record<string, unknown>): MeasureJson | null {
  const id = num(o.id);
  const kind = o.kind;
  if (id === null || (kind !== 'length' && kind !== 'circumference' && kind !== 'distance')) return null;
  const value = num(o.value) ?? undefined;
  if (kind === 'circumference') {
    const plane = obj(o.plane);
    const at = triples([plane.point])[0];
    const lines = Array.isArray(o.polylines) ? o.polylines.map(triples).filter((l) => l.length > 1) : [];
    if (!at) return null;
    return { id, kind, points: [at], value, lines, height: at[1] };
  }
  const points = triples(o.polyline);
  if (points.length < 2) return null;
  return { id, kind, points, value };
}

export class Measure {
  tool: MeasureTool = 'none';
  items: MeasureJson[] = [];
  private pending: WorldHit[] = [];
  private readonly buttons = new Map<MeasureTool, HTMLButtonElement>();
  private readonly hint: HTMLDivElement;
  private readonly clear: HTMLButtonElement;
  private localId = -1;
  private slices = new Map<number, Slice>();
  private listTimer = 0;

  constructor(private readonly host: VizHost, private readonly pop: Pop) {
    pop.setLabel('Measure');
    for (const [id, label] of TOOLS) this.buttons.set(id, pop.item(label, () => this.setTool(this.tool === id ? 'none' : id)));
    pop.separator();
    this.clear = pop.item('Clear measures', () => void this.clearAll());
    this.hint = document.createElement('div');
    this.hint.className = 'ffz-hint';
    this.hint.hidden = true;
    host.pane.appendChild(this.hint);
    document.addEventListener('keydown', (e) => this.onKey(e), true);
  }

  get active(): boolean {
    return this.tool !== 'none';
  }

  available(): void {
    const ok = hasCommand('measure3d.add');
    for (const b of this.buttons.values()) b.title = ok ? '' : missing('measure3d.add');
    setTitle(this.pop.button, ok ? t('Measure in 3D') : `${t('Measure in 3D')} (${t('local only')})`);
  }

  setTool(tool: MeasureTool): void {
    this.tool = tool;
    this.pending = [];
    for (const [id, b] of this.buttons) setClass(b, 'is-on', id === tool);
    this.pop.setActive(tool !== 'none');
    setHidden(this.hint, tool === 'none');
    if (tool !== 'none') setText(this.hint, t(HINTS[tool]));
    this.pop.close();
    this.host.redraw();
  }

  private onKey(e: KeyboardEvent): void {
    if (this.tool === 'none' || e.defaultPrevented || anyDialogOpen() || textFieldFocused()) return;
    if (e.key === 'Escape') {
      if (this.pending.length) this.pending = [];
      else this.setTool('none');
      this.host.redraw();
      e.preventDefault();
    } else if (e.key === 'Enter' && this.tool === 'length' && this.pending.length >= 2) {
      void this.commit();
      e.preventDefault();
    }
  }

  click(hit: WorldHit | null): void {
    if (!hit || this.tool === 'none') return;
    this.pending.push(hit);
    const need = this.tool === 'distance' ? 2 : this.tool === 'circumference' ? 1 : 0;
    if (need && this.pending.length >= need) void this.commit();
    this.host.redraw();
  }

  private async commit(): Promise<void> {
    const kind = this.tool;
    const pts = this.pending;
    this.pending = [];
    if (kind === 'none' || pts.length === 0) return;
    const points = pts.map((p) => p.point);
    const m: MeasureJson = { id: this.localId--, kind, points };
    this.items.push(m);
    this.host.redraw();
    if (!hasCommand('measure3d.add')) return;
    const args = kind === 'circumference' ? { kind, height: points[0][1] } : { kind, points: points.map((p) => ({ position: p })) };
    const r = await run<unknown>('measure3d.add', args);
    const got = r === undefined ? null : fromEngine(obj(r));
    if (got) Object.assign(m, got);
    else this.items = this.items.filter((x) => x !== m);
    this.host.redraw();
  }

  private async clearAll(): Promise<void> {
    const ids = this.items.map((m) => m.id).filter((id) => id >= 0);
    this.items = [];
    this.pending = [];
    this.host.redraw();
    this.pop.close();
    if (!hasCommand('measure3d.remove')) return;
    for (const id of ids) await run('measure3d.remove', { id });
  }

  changed(name: string): void {
    if (!/^(measure3d|history|file)\./.test(name)) return;
    window.clearTimeout(this.listTimer);
    this.listTimer = window.setTimeout(() => void this.pull(), 250);
  }

  async pull(): Promise<void> {
    if (!hasCommand('measure3d.list')) return;
    const r = await quiet<unknown>('measure3d.list', {});
    if (r === undefined) return;
    const arr = Array.isArray(r) ? r : Array.isArray(obj(r).measures) ? (obj(r).measures as unknown[]) : Array.isArray(obj(r).items) ? (obj(r).items as unknown[]) : null;
    if (!arr) return;
    const next: MeasureJson[] = [];
    for (const x of arr) {
      const got = fromEngine(obj(x));
      if (got) next.push(got);
    }
    this.items = [...next, ...this.items.filter((m) => m.id < 0)];
    this.host.redraw();
  }

  private slice(scene: Scene, y: number): Slice {
    let s = this.slices.get(y);
    if (s && s.version === scene.version) return s;
    if (!s) {
      s = { version: -1, y, segs: new Float32Array(0), count: 0, total: 0 };
      this.slices.set(y, s);
      if (this.slices.size > 24) this.slices.delete(this.slices.keys().next().value as number);
    }
    let n = 0;
    let total = 0;
    let out = s.segs;
    for (const buf of scene.list) {
      if (!buf.ready || buf.kind !== 'panel') continue;
      const p = buf.pos;
      const idx = buf.idx;
      if (buf.max[1] < y || buf.min[1] > y) continue;
      for (let i = 0; i < idx.length; i += 3) {
        const i0 = idx[i] * 3;
        const i1 = idx[i + 1] * 3;
        const i2 = idx[i + 2] * 3;
        const d0 = p[i0 + 1] - y;
        const d1 = p[i1 + 1] - y;
        const d2 = p[i2 + 1] - y;
        if ((d0 > 0 && d1 > 0 && d2 > 0) || (d0 < 0 && d1 < 0 && d2 < 0)) continue;
        const hits: number[] = [];
        const edge = (ia: number, ib: number, da: number, db: number): void => {
          if ((da > 0) === (db > 0) || da === db) return;
          const k = da / (da - db);
          hits.push(p[ia] + (p[ib] - p[ia]) * k, y, p[ia + 2] + (p[ib + 2] - p[ia + 2]) * k);
        };
        edge(i0, i1, d0, d1);
        edge(i1, i2, d1, d2);
        edge(i2, i0, d2, d0);
        if (hits.length < 6) continue;
        if (out.length < (n + 1) * 6) {
          const grown = new Float32Array(Math.max(256, out.length * 2));
          grown.set(out);
          out = grown;
        }
        for (let k = 0; k < 6; k++) out[n * 6 + k] = hits[k];
        total += Math.hypot(hits[3] - hits[0], hits[5] - hits[2]);
        n++;
      }
    }
    s.segs = out;
    s.count = n;
    s.total = total;
    s.version = scene.version;
    return s;
  }

  value(m: MeasureJson, scene: Scene): number {
    if (m.value !== undefined) return m.value;
    const p = m.points;
    if (m.kind === 'circumference') return this.slice(scene, p[0][1]).total;
    let sum = 0;
    for (let i = 1; i < p.length; i++) sum += Math.hypot(p[i][0] - p[i - 1][0], p[i][1] - p[i - 1][1], p[i][2] - p[i - 1][2]);
    return sum;
  }

  draw(ctx: CanvasRenderingContext2D, f: Frame, scene: Scene, dpr: number, accent: string, ring: string): void {
    if (this.items.length === 0 && this.pending.length === 0) return;
    ctx.lineWidth = 1.5 * dpr;
    ctx.lineJoin = 'round';
    ctx.font = `${Math.round(11 * dpr)}px ui-monospace, monospace`;
    ctx.textBaseline = 'middle';
    const dot = (x: number, y: number): void => {
      ctx.beginPath();
      ctx.arc(x * dpr, y * dpr, 3.5 * dpr, 0, Math.PI * 2);
      ctx.fillStyle = accent;
      ctx.fill();
      ctx.strokeStyle = ring;
      ctx.stroke();
    };
    const units = this.host.units();
    for (const m of this.items) {
      ctx.strokeStyle = accent;
      let lx = NaN;
      let ly = NaN;
      if (m.kind === 'circumference' && m.lines && m.lines.length > 0) {
        ctx.beginPath();
        for (const line of m.lines) {
          let pen = false;
          for (const p of line) {
            if (!f.project(p[0], p[1], p[2], a)) { pen = false; continue; }
            if (pen) ctx.lineTo(a[0] * dpr, a[1] * dpr); else ctx.moveTo(a[0] * dpr, a[1] * dpr);
            pen = true;
          }
        }
        ctx.stroke();
        const anchor = m.lines[0][0];
        if (f.project(anchor[0], anchor[1], anchor[2], a)) { lx = a[0]; ly = a[1]; dot(lx, ly); }
      } else if (m.kind === 'circumference') {
        const s = this.slice(scene, m.points[0][1]);
        ctx.beginPath();
        let any = false;
        for (let i = 0; i < s.count; i++) {
          const o = i * 6;
          if (!f.project(s.segs[o], s.segs[o + 1], s.segs[o + 2], a) || !f.project(s.segs[o + 3], s.segs[o + 4], s.segs[o + 5], b)) continue;
          ctx.moveTo(a[0] * dpr, a[1] * dpr);
          ctx.lineTo(b[0] * dpr, b[1] * dpr);
          any = true;
        }
        if (any) ctx.stroke();
        if (f.project(m.points[0][0], m.points[0][1], m.points[0][2], a)) { lx = a[0]; ly = a[1]; dot(lx, ly); }
      } else {
        ctx.beginPath();
        let first = true;
        let mid = NaN;
        let midY = NaN;
        let count = 0;
        for (const p of m.points) {
          if (!f.project(p[0], p[1], p[2], a)) { first = true; continue; }
          if (first) ctx.moveTo(a[0] * dpr, a[1] * dpr); else ctx.lineTo(a[0] * dpr, a[1] * dpr);
          first = false;
          count++;
          if (count === Math.ceil(m.points.length / 2)) { mid = a[0]; midY = a[1]; }
        }
        ctx.stroke();
        for (const p of m.points) if (f.project(p[0], p[1], p[2], a)) dot(a[0], a[1]);
        if (m.points.length === 2 && f.project(m.points[0][0], m.points[0][1], m.points[0][2], a) && f.project(m.points[1][0], m.points[1][1], m.points[1][2], b)) {
          mid = (a[0] + b[0]) / 2;
          midY = (a[1] + b[1]) / 2;
        }
        lx = mid;
        ly = midY;
      }
      if (!Number.isFinite(lx)) continue;
      const text = fmtLen(this.value(m, scene), units);
      const wd = ctx.measureText(text).width;
      const x = lx * dpr;
      const y = ly * dpr - 14 * dpr;
      ctx.globalAlpha = 0.9;
      ctx.fillStyle = accent;
      ctx.fillRect(x - wd / 2 - 4 * dpr, y - 8 * dpr, wd + 8 * dpr, 16 * dpr);
      ctx.globalAlpha = 1;
      ctx.fillStyle = ring;
      ctx.fillText(text, x - wd / 2, y);
    }
    for (const h of this.pending) if (f.project(h.point[0], h.point[1], h.point[2], a)) dot(a[0], a[1]);
  }
}
