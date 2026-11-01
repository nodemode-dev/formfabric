import { dist, place, placeVec, unplace, type PanelGeo } from '../geom';
import type { SeamEnd } from '../pattern-view/sew-types';
import type { Panel, Pt, Vec } from '../types';
import { num, trimDoc, printsOf, styleOf, type TFinish, type THardware, type TPrint, type TStitch, type TStyle, type TZipper, type TrimDoc, type TrimItem } from './types';

export interface LocalRun { panel: Panel; pts: Vec[] }

function lineOf(panel: Panel, inner: number | null): { pts: Pt[]; closed: boolean } | null {
  if (inner === null || inner === undefined) return { pts: panel.outline, closed: true };
  const i = panel.inner.find((x) => x.id === inner);
  return i ? { pts: i.points, closed: i.closed } : null;
}

function bez(a: Vec, c1: Vec, c2: Vec, b: Vec, t: number): Vec {
  const u = 1 - t;
  const w0 = u * u * u;
  const w1 = 3 * u * u * t;
  const w2 = 3 * u * t * t;
  const w3 = t * t * t;
  return [w0 * a[0] + w1 * c1[0] + w2 * c2[0] + w3 * b[0], w0 * a[1] + w1 * c1[1] + w2 * c2[1] + w3 * b[1]];
}

function spanPts(panel: Panel, e: SeamEnd): Vec[] {
  const line = lineOf(panel, e.inner);
  if (!line) return [];
  const n = line.pts.length;
  const count = line.closed ? n : n - 1;
  if (e.segment < 0 || e.segment >= count || n < 2) return [];
  const p0 = line.pts[e.segment];
  const p1 = line.pts[(e.segment + 1) % n];
  const curved = !!(p0.h_out || p1.h_in);
  const c1: Vec = p0.h_out ? [p0.p[0] + p0.h_out[0], p0.p[1] + p0.h_out[1]] : p0.p;
  const c2: Vec = p1.h_in ? [p1.p[0] + p1.h_in[0], p1.p[1] + p1.h_in[1]] : p1.p;
  const steps = curved ? Math.max(4, Math.ceil(Math.abs(e.t1 - e.t0) * 24)) : 1;
  const out: Vec[] = [];
  for (let i = 0; i <= steps; i++) {
    const t = e.t0 + (e.t1 - e.t0) * (i / steps);
    out.push(curved ? bez(p0.p, c1, c2, p1.p, t) : [p0.p[0] + (p1.p[0] - p0.p[0]) * t, p0.p[1] + (p1.p[1] - p0.p[1]) * t]);
  }
  return out;
}

export function localRuns(panels: Map<number, Panel>, ends: SeamEnd[]): LocalRun[] {
  const runs: LocalRun[] = [];
  for (const e of ends) {
    const panel = panels.get(e.panel);
    if (!panel) continue;
    const pts = spanPts(panel, e);
    if (pts.length < 2) continue;
    const last = runs[runs.length - 1];
    if (last && last.panel.id === panel.id) {
      const tail = last.pts[last.pts.length - 1];
      last.pts.push(...(dist(tail, pts[0]) < 1e-6 ? pts.slice(1) : pts));
    } else runs.push({ panel, pts });
  }
  return runs;
}

export function offsetLine(pts: Vec[], d: number): Vec[] {
  const n = pts.length;
  if (n < 2 || d === 0) return pts;
  const seg: Vec[] = [];
  for (let i = 0; i < n - 1; i++) {
    const dx = pts[i + 1][0] - pts[i][0];
    const dy = pts[i + 1][1] - pts[i][1];
    const l = Math.hypot(dx, dy) || 1;
    seg.push([-dy / l, dx / l]);
  }
  const out: Vec[] = [];
  for (let i = 0; i < n; i++) {
    const a = seg[Math.max(0, i - 1)];
    const b = seg[Math.min(n - 2, i)];
    let nx = a[0] + b[0];
    let ny = a[1] + b[1];
    const l = Math.hypot(nx, ny);
    if (l < 1e-9) { nx = b[0]; ny = b[1]; } else { nx /= l; ny /= l; }
    const k = 1 / Math.max(0.5, nx * b[0] + ny * b[1]);
    out.push([pts[i][0] + nx * d * k, pts[i][1] + ny * d * k]);
  }
  return out;
}

export interface Walk { pts: Vec[]; cum: number[]; total: number }

export function walkOf(pts: Vec[]): Walk {
  const cum = [0];
  for (let i = 1; i < pts.length; i++) cum.push(cum[i - 1] + dist(pts[i - 1], pts[i]));
  return { pts, cum, total: cum[cum.length - 1] ?? 0 };
}

export function atLength(w: Walk, s: number): { p: Vec; d: Vec } {
  const pts = w.pts;
  if (pts.length < 2) return { p: pts[0] ?? [0, 0], d: [1, 0] };
  const want = Math.min(w.total, Math.max(0, s));
  let i = 1;
  while (i < pts.length - 1 && w.cum[i] < want) i++;
  const span = w.cum[i] - w.cum[i - 1];
  const u = span > 1e-9 ? (want - w.cum[i - 1]) / span : 0;
  const a = pts[i - 1];
  const b = pts[i];
  const l = Math.hypot(b[0] - a[0], b[1] - a[1]) || 1;
  return { p: [a[0] + (b[0] - a[0]) * u, a[1] + (b[1] - a[1]) * u], d: [(b[0] - a[0]) / l, (b[1] - a[1]) / l] };
}

export function sliceWalk(w: Walk, from: number, to: number): Vec[] {
  const out: Vec[] = [atLength(w, from).p];
  for (let i = 0; i < w.pts.length; i++) if (w.cum[i] > from + 1e-9 && w.cum[i] < to - 1e-9) out.push(w.pts[i]);
  out.push(atLength(w, to).p);
  return out;
}

export interface StitchGeo { stitch: TStitch; style: TStyle | undefined; mode: 'topstitch' | 'puckering' | 'fray'; lines: Vec[][]; local: LocalRun[] }
export interface ZipGeo { zipper: TZipper; style: TStyle | undefined; a: Vec[]; b: Vec[]; width: number; localA: LocalRun[]; localB: LocalRun[] }
export interface FinishGeo { finish: TFinish; lines: Vec[][]; local: LocalRun[] }
export interface HwGeo { hw: THardware; style: TStyle | undefined; at: Vec; angle: number; width: number; length: number; panel: Panel }
export interface PrintGeo { geo: PanelGeo; print: TPrint }

export interface TrimScene {
  doc: TrimDoc;
  stitches: StitchGeo[];
  zippers: ZipGeo[];
  finishes: FinishGeo[];
  hardware: HwGeo[];
  prints: Map<number, PrintGeo[]>;
  byHardware: Map<number, HwGeo>;
}

export const EMPTY_SCENE: TrimScene = { doc: { styles: [], hardware: [], zippers: [], stitches: [], finishes: [] }, stitches: [], zippers: [], finishes: [], hardware: [], prints: new Map(), byHardware: new Map() };

function toWorld(runs: LocalRun[]): Vec[][] {
  return runs.map((r) => r.pts.map((p) => place(r.panel, p)));
}

function flat(runs: LocalRun[]): Vec[] {
  const out: Vec[] = [];
  for (const r of runs) for (const p of r.pts) out.push(place(r.panel, p));
  return out;
}

function pucker(pts: Vec[], amp: number): Vec[] {
  const w = walkOf(pts);
  const out: Vec[] = [];
  const step = Math.max(1.5, amp * 1.4);
  for (let s = 0, k = 0; s <= w.total; s += step, k++) {
    const { p, d } = atLength(w, s);
    const side = k % 2 === 0 ? amp : -amp;
    out.push([p[0] - d[1] * side, p[1] + d[0] * side]);
  }
  return out;
}

export function buildTrimScene(geos: PanelGeo[], doc: TrimDoc): TrimScene {
  const panels = new Map<number, Panel>(geos.map((g) => [g.panel.id, g.panel]));
  const stitches: StitchGeo[] = [];
  for (const stitch of doc.stitches) {
    const style = styleOf(doc, stitch.style);
    const fray = style ? num(style.params, 'fray', 0) : 0;
    const mode = fray > 0 ? 'fray' : style?.kind === 'puckering' ? 'puckering' : 'topstitch';
    const base = localRuns(panels, stitch.spans);
    const rows = mode === 'topstitch' ? Math.max(1, Math.round(num(style?.params, 'count', 1))) : 1;
    const gap = Math.max(1.2, num(style?.params, 'thickness', 0.6) * 3);
    const local: LocalRun[] = [];
    for (const r of base) {
      for (let k = 0; k < rows; k++) {
        const d = stitch.offset + k * gap;
        let pts = offsetLine(r.pts, d);
        if (mode === 'puckering') pts = pucker(pts, Math.max(0.4, num(style?.params, 'width', 3) / 2));
        local.push({ panel: r.panel, pts });
      }
    }
    stitches.push({ stitch, style, mode, lines: toWorld(local), local });
  }
  const zippers: ZipGeo[] = doc.zippers.map((zipper) => {
    const style = styleOf(doc, zipper.style);
    const localA = localRuns(panels, zipper.a);
    const localB = localRuns(panels, zipper.b);
    return { zipper, style, a: flat(localA), b: flat(localB), width: num(style?.params, 'width', 10), localA, localB };
  });
  const finishes: FinishGeo[] = doc.finishes.map((finish) => {
    const local = localRuns(panels, finish.spans);
    return { finish, lines: toWorld(local), local };
  });
  const hardware: HwGeo[] = [];
  const byHardware = new Map<number, HwGeo>();
  for (const hw of doc.hardware) {
    const panel = panels.get(hw.anchor.panel);
    if (!panel) continue;
    const style = styleOf(doc, hw.style);
    const r = (hw.rotation * Math.PI) / 180;
    const dir = placeVec(panel, [Math.cos(r), Math.sin(r)]);
    const g: HwGeo = {
      hw, style, panel,
      at: place(panel, hw.anchor.point),
      angle: Math.atan2(dir[1], dir[0]),
      width: style?.kind === 'buttonhole' ? 3 : num(style?.params, 'width', 12),
      length: style?.kind === 'buttonhole' ? num(style.params, 'length', 14) : num(style?.params, 'width', 12),
    };
    hardware.push(g);
    byHardware.set(hw.id, g);
  }
  const prints = new Map<number, PrintGeo[]>();
  for (const geo of geos) {
    const list = printsOf(geo.panel);
    if (list.length) prints.set(geo.panel.id, list.map((print) => ({ geo, print })));
  }
  return { doc, stitches, zippers, finishes, hardware, prints, byHardware };
}

export function sceneOf(geos: PanelGeo[], g: Parameters<typeof trimDoc>[0]): TrimScene {
  const doc = trimDoc(g);
  const any = doc.hardware.length + doc.zippers.length + doc.stitches.length + doc.finishes.length;
  const prints = geos.some((x) => printsOf(x.panel).length > 0);
  return any === 0 && !prints ? EMPTY_SCENE : buildTrimScene(geos, doc);
}

function nearPoly(pts: Vec[], w: Vec): number {
  let best = Infinity;
  for (let i = 1; i < pts.length; i++) {
    const a = pts[i - 1];
    const b = pts[i];
    const dx = b[0] - a[0];
    const dy = b[1] - a[1];
    const l2 = dx * dx + dy * dy;
    const u = l2 === 0 ? 0 : Math.max(0, Math.min(1, ((w[0] - a[0]) * dx + (w[1] - a[1]) * dy) / l2));
    const d = Math.hypot(a[0] + dx * u - w[0], a[1] + dy * u - w[1]);
    if (d < best) best = d;
  }
  return best;
}

export function inPrint(pg: PrintGeo, w: Vec): boolean {
  const l = unplace(pg.geo.panel, w);
  const dx = l[0] - pg.print.position[0];
  const dy = l[1] - pg.print.position[1];
  const a = -(pg.print.rotation * Math.PI) / 180;
  const lx = dx * Math.cos(a) - dy * Math.sin(a);
  const ly = dx * Math.sin(a) + dy * Math.cos(a);
  return Math.abs(lx) <= pg.print.size[0] / 2 && Math.abs(ly) <= pg.print.size[1] / 2;
}

export function pickTrim(scene: TrimScene, w: Vec, tol: number, only?: Array<TrimItem['kind']>): TrimItem | null {
  const want = (k: TrimItem['kind']): boolean => !only || only.includes(k);
  let best: { item: TrimItem; d: number } | null = null;
  const offer = (item: TrimItem, d: number, limit: number): void => {
    if (d <= limit && (!best || d < best.d)) best = { item, d };
  };
  if (want('hardware')) for (const h of scene.hardware) offer({ kind: 'hardware', id: h.hw.id }, dist(h.at, w), Math.max(tol, h.length / 2));
  if (want('stitch')) for (const s of scene.stitches) for (const l of s.lines) offer({ kind: 'stitch', id: s.stitch.id }, nearPoly(l, w), tol);
  if (want('zipper')) for (const z of scene.zippers) offer({ kind: 'zipper', id: z.zipper.id }, Math.min(nearPoly(z.a, w), nearPoly(z.b, w)), tol);
  if (want('finish')) for (const f of scene.finishes) for (const l of f.lines) offer({ kind: 'finish', id: f.finish.id }, nearPoly(l, w), tol);
  if (best) return (best as { item: TrimItem }).item;
  if (want('print')) {
    for (const list of [...scene.prints.values()].reverse()) {
      for (let i = list.length - 1; i >= 0; i--) if (inPrint(list[i], w)) return { kind: 'print', id: list[i].print.id };
    }
  }
  return null;
}

