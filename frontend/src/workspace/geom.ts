import type { Panel, Pt, Vec } from './types';

export const add = (a: Vec, b: Vec): Vec => [a[0] + b[0], a[1] + b[1]];
export const sub = (a: Vec, b: Vec): Vec => [a[0] - b[0], a[1] - b[1]];
export const mul = (a: Vec, s: number): Vec => [a[0] * s, a[1] * s];
export const dot = (a: Vec, b: Vec): number => a[0] * b[0] + a[1] * b[1];
export const len = (a: Vec): number => Math.hypot(a[0], a[1]);
export const dist = (a: Vec, b: Vec): number => Math.hypot(a[0] - b[0], a[1] - b[1]);
export const lerp = (a: Vec, b: Vec, t: number): Vec => [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t];

const RAD = Math.PI / 180;

type Placement = Pick<Panel, 'rotation' | 'mirror' | 'position'>;

export function placeVec(panel: Placement, v: Vec): Vec {
  const x = panel.mirror ? -v[0] : v[0];
  const c = Math.cos(panel.rotation * RAD);
  const s = Math.sin(panel.rotation * RAD);
  return [x * c - v[1] * s, x * s + v[1] * c];
}

export function place(panel: Placement, v: Vec): Vec {
  const r = placeVec(panel, v);
  return [r[0] + panel.position[0], r[1] + panel.position[1]];
}

export function unplace(panel: Placement, w: Vec): Vec {
  const x = w[0] - panel.position[0];
  const y = w[1] - panel.position[1];
  const c = Math.cos(-panel.rotation * RAD);
  const s = Math.sin(-panel.rotation * RAD);
  const lx = x * c - y * s;
  return [panel.mirror ? -lx : lx, x * s + y * c];
}

export interface Seg { a: Vec; c1: Vec; c2: Vec; b: Vec; curved: boolean }

export function segsOf(panel: Panel, pts: Pt[], closed: boolean): Seg[] {
  const n = pts.length;
  const out: Seg[] = [];
  const count = closed ? n : n - 1;
  for (let i = 0; i < count; i++) {
    const p0 = pts[i];
    const p1 = pts[(i + 1) % n];
    const a = place(panel, p0.p);
    const b = place(panel, p1.p);
    const c1 = p0.h_out ? add(a, placeVec(panel, p0.h_out)) : a;
    const c2 = p1.h_in ? add(b, placeVec(panel, p1.h_in)) : b;
    out.push({ a, c1, c2, b, curved: !!(p0.h_out || p1.h_in) });
  }
  return out;
}

export function bezAt(s: Seg, t: number): Vec {
  if (!s.curved) return lerp(s.a, s.b, t);
  const u = 1 - t;
  const w0 = u * u * u;
  const w1 = 3 * u * u * t;
  const w2 = 3 * u * t * t;
  const w3 = t * t * t;
  return [w0 * s.a[0] + w1 * s.c1[0] + w2 * s.c2[0] + w3 * s.b[0], w0 * s.a[1] + w1 * s.c1[1] + w2 * s.c2[1] + w3 * s.b[1]];
}

export function flatten(s: Seg, n: number, out: Vec[]): void {
  const steps = s.curved ? n : 1;
  for (let i = 1; i <= steps; i++) out.push(bezAt(s, i / steps));
}

export function segLength(s: Seg): number {
  if (!s.curved) return dist(s.a, s.b);
  let total = 0;
  let prev = s.a;
  for (let i = 1; i <= 24; i++) {
    const p = bezAt(s, i / 24);
    total += dist(prev, p);
    prev = p;
  }
  return total;
}

export function nearestOnSeg(s: Seg, q: Vec): { t: number; pt: Vec; d: number } {
  if (!s.curved) {
    const ab = sub(s.b, s.a);
    const l2 = dot(ab, ab);
    const t = l2 === 0 ? 0 : Math.max(0, Math.min(1, dot(sub(q, s.a), ab) / l2));
    const pt = lerp(s.a, s.b, t);
    return { t, pt, d: dist(pt, q) };
  }
  const N = 24;
  let best = 0;
  let bestD = Infinity;
  for (let i = 0; i <= N; i++) {
    const d = dist(bezAt(s, i / N), q);
    if (d < bestD) { bestD = d; best = i / N; }
  }
  let lo = Math.max(0, best - 1 / N);
  let hi = Math.min(1, best + 1 / N);
  for (let k = 0; k < 10; k++) {
    const m1 = lo + (hi - lo) / 3;
    const m2 = hi - (hi - lo) / 3;
    if (dist(bezAt(s, m1), q) < dist(bezAt(s, m2), q)) hi = m2; else lo = m1;
  }
  const t = (lo + hi) / 2;
  const pt = bezAt(s, t);
  return { t, pt, d: dist(pt, q) };
}

export type Bounds = [number, number, number, number];

export function boundsOf(points: Vec[]): Bounds {
  let x0 = Infinity, y0 = Infinity, x1 = -Infinity, y1 = -Infinity;
  for (const p of points) {
    if (p[0] < x0) x0 = p[0];
    if (p[0] > x1) x1 = p[0];
    if (p[1] < y0) y0 = p[1];
    if (p[1] > y1) y1 = p[1];
  }
  return [x0, y0, x1, y1];
}

export function unionBounds(a: Bounds | null, b: Bounds): Bounds {
  return a ? [Math.min(a[0], b[0]), Math.min(a[1], b[1]), Math.max(a[2], b[2]), Math.max(a[3], b[3])] : b;
}

export function pointInPoly(p: Vec, poly: Vec[]): boolean {
  let inside = false;
  for (let i = 0, j = poly.length - 1; i < poly.length; j = i++) {
    const a = poly[i];
    const b = poly[j];
    if ((a[1] > p[1]) !== (b[1] > p[1]) && p[0] < ((b[0] - a[0]) * (p[1] - a[1])) / (b[1] - a[1]) + a[0]) inside = !inside;
  }
  return inside;
}

export interface InnerGeo { id: number; kind: string; closed: boolean; segs: Seg[]; pts: Vec[] }

export interface PanelGeo {
  panel: Panel;
  outline: Seg[];
  inner: InnerGeo[];
  pts: Vec[];
  hin: Array<Vec | null>;
  hout: Array<Vec | null>;
  poly: Vec[];
  bounds: Bounds;
  center: Vec;
}

const cache = new WeakMap<Panel, PanelGeo>();

export function geoOf(panel: Panel): PanelGeo {
  const hit = cache.get(panel);
  if (hit) return hit;
  const outline = segsOf(panel, panel.outline, true);
  const pts = panel.outline.map((p) => place(panel, p.p));
  const hin = panel.outline.map((p, i) => (p.h_in ? add(pts[i], placeVec(panel, p.h_in)) : null));
  const hout = panel.outline.map((p, i) => (p.h_out ? add(pts[i], placeVec(panel, p.h_out)) : null));
  const poly: Vec[] = pts.length ? [pts[0]] : [];
  for (const s of outline) flatten(s, 16, poly);
  const b = boundsOf(poly.length ? poly : [[0, 0]]);
  const geo: PanelGeo = {
    panel, outline, pts, hin, hout, poly, bounds: b, center: [(b[0] + b[2]) / 2, (b[1] + b[3]) / 2],
    inner: panel.inner.map((i) => ({ id: i.id, kind: i.kind, closed: i.closed, segs: segsOf(panel, i.points, i.closed), pts: i.points.map((p) => place(panel, p.p)) })),
  };
  cache.set(panel, geo);
  return geo;
}

export type HitKind = 'handle' | 'point' | 'segment' | 'panel';

export interface Hit { kind: HitKind; panel: number; index: number; which?: 'in' | 'out'; t?: number; pt?: Vec; d: number }

export interface HitOptions { handles: boolean; points: boolean; segments: boolean; panels: boolean }

export function hitTest(geos: PanelGeo[], w: Vec, tol: number, o: HitOptions): Hit | null {
  const ordered = [...geos].sort((a, b) => b.panel.layer - a.panel.layer);
  if (o.handles) {
    let best: Hit | null = null;
    for (const g of ordered) {
      for (let i = 0; i < g.pts.length; i++) {
        for (const which of ['in', 'out'] as const) {
          const h = which === 'in' ? g.hin[i] : g.hout[i];
          if (!h) continue;
          const d = dist(h, w);
          if (d <= tol && (!best || d < best.d)) best = { kind: 'handle', panel: g.panel.id, index: i, which, d, pt: h };
        }
      }
    }
    if (best) return best;
  }
  if (o.points) {
    let best: Hit | null = null;
    for (const g of ordered) {
      for (let i = 0; i < g.pts.length; i++) {
        const d = dist(g.pts[i], w);
        if (d <= tol && (!best || d < best.d)) best = { kind: 'point', panel: g.panel.id, index: i, d, pt: g.pts[i] };
      }
    }
    if (best) return best;
  }
  if (o.segments) {
    let best: Hit | null = null;
    for (const g of ordered) {
      for (let i = 0; i < g.outline.length; i++) {
        const r = nearestOnSeg(g.outline[i], w);
        if (r.d <= tol && (!best || r.d < best.d)) best = { kind: 'segment', panel: g.panel.id, index: i, t: r.t, pt: r.pt, d: r.d };
      }
    }
    if (best) return best;
  }
  if (o.panels) {
    for (const g of ordered) if (pointInPoly(w, g.poly)) return { kind: 'panel', panel: g.panel.id, index: -1, d: 0 };
  }
  return null;
}
