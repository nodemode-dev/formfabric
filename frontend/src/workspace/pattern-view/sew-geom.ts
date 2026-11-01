import { bezAt, dist, nearestOnSeg, place, type PanelGeo, type Seg } from '../geom';
import type { Vec } from '../types';
import type { Seam, SeamEnd, SeamInfo, SewSnapshot, Tack } from './sew-types';

export interface LineRef { panel: number; inner: number | null }

export interface LinePick extends LineRef { segment: number; t: number; pt: Vec; d: number }

export interface Chain { pts: Vec[]; cum: number[]; total: number }

export interface SeamGeo { seam: Seam; a: Chain; b: Chain; color: string; mismatch: number | null; info: SeamInfo | undefined }

export interface TackGeo { tack: Tack; a: Vec | null; b: Vec | null }

export interface SewScene { seams: SeamGeo[]; tacks: TackGeo[] }

export interface LineSpan { ref: LineRef; n: number; closed: boolean; p0: number; p1: number }

export interface Run { segment: number; t0: number; t1: number }

export function lineSegs(geo: PanelGeo | undefined, inner: number | null): Seg[] {
  if (!geo) return [];
  if (inner === null) return geo.outline;
  return geo.inner.find((i) => i.id === inner)?.segs ?? [];
}

export function lineClosed(geo: PanelGeo | undefined, inner: number | null): boolean {
  if (!geo) return false;
  if (inner === null) return true;
  return geo.inner.find((i) => i.id === inner)?.closed ?? false;
}

export function seamColor(id: number): string {
  return `hsl(${Math.round((id * 137.508) % 360)}, 68%, 60%)`;
}

export function pickLine(geos: PanelGeo[], w: Vec, tol: number): LinePick | null {
  let best: LinePick | null = null;
  const ordered = [...geos].sort((x, y) => y.panel.layer - x.panel.layer);
  for (const geo of ordered) {
    const lines: Array<[number | null, Seg[]]> = [[null, geo.outline]];
    for (const inn of geo.inner) lines.push([inn.id, inn.segs]);
    for (const [inner, segs] of lines) {
      for (let i = 0; i < segs.length; i++) {
        const r = nearestOnSeg(segs[i], w);
        if (r.d <= tol && (!best || r.d < best.d - 1e-9)) best = { panel: geo.panel.id, inner, segment: i, t: r.t, pt: r.pt, d: r.d };
      }
    }
  }
  return best;
}

export function nearestParam(geo: PanelGeo | undefined, inner: number | null, w: Vec): { p: number; pt: Vec; d: number } | null {
  const segs = lineSegs(geo, inner);
  let best: { p: number; pt: Vec; d: number } | null = null;
  for (let i = 0; i < segs.length; i++) {
    const r = nearestOnSeg(segs[i], w);
    if (!best || r.d < best.d) best = { p: i + r.t, pt: r.pt, d: r.d };
  }
  return best;
}

export function pointAtParam(geo: PanelGeo | undefined, inner: number | null, p: number): Vec | null {
  const segs = lineSegs(geo, inner);
  if (!segs.length) return null;
  const seg = Math.min(segs.length - 1, Math.max(0, Math.floor(p)));
  return bezAt(segs[seg], Math.min(1, Math.max(0, p - seg)));
}

export function spanPoints(geos: Map<number, PanelGeo>, e: SeamEnd): Vec[] {
  const seg = lineSegs(geos.get(e.panel), e.inner)[e.segment];
  if (!seg) return [];
  const steps = seg.curved ? Math.max(2, Math.ceil(Math.abs(e.t1 - e.t0) * 24)) : 1;
  const out: Vec[] = [];
  for (let i = 0; i <= steps; i++) out.push(bezAt(seg, e.t0 + (e.t1 - e.t0) * (i / steps)));
  return out;
}

export function chainOf(geos: Map<number, PanelGeo>, ends: SeamEnd[]): Chain {
  const pts: Vec[] = [];
  const cum: number[] = [];
  let total = 0;
  for (const e of ends) {
    for (const p of spanPoints(geos, e)) {
      if (pts.length) total += dist(pts[pts.length - 1], p);
      pts.push(p);
      cum.push(total);
    }
  }
  return { pts, cum, total };
}

export function pointAt(c: Chain, f: number): Vec {
  if (!c.pts.length) return [0, 0];
  const want = Math.min(1, Math.max(0, f)) * c.total;
  let i = 1;
  while (i < c.pts.length - 1 && c.cum[i] < want) i++;
  if (c.pts.length === 1) return c.pts[0];
  const span = c.cum[i] - c.cum[i - 1];
  const u = span > 1e-9 ? (want - c.cum[i - 1]) / span : 0;
  const a = c.pts[i - 1];
  const b = c.pts[i];
  return [a[0] + (b[0] - a[0]) * u, a[1] + (b[1] - a[1]) * u];
}

export function nearestOnChain(c: Chain, w: Vec): { d: number; f: number } {
  let best = { d: Infinity, f: 0 };
  for (let i = 1; i < c.pts.length; i++) {
    const a = c.pts[i - 1];
    const b = c.pts[i];
    const dx = b[0] - a[0];
    const dy = b[1] - a[1];
    const l2 = dx * dx + dy * dy;
    const u = l2 === 0 ? 0 : Math.max(0, Math.min(1, ((w[0] - a[0]) * dx + (w[1] - a[1]) * dy) / l2));
    const d = Math.hypot(a[0] + dx * u - w[0], a[1] + dy * u - w[1]);
    if (d < best.d) best = { d, f: c.total > 0 ? (c.cum[i - 1] + (c.cum[i] - c.cum[i - 1]) * u) / c.total : 0 };
  }
  return best;
}

export function buildSewScene(geos: PanelGeo[], sew: SewSnapshot): SewScene {
  const byId = new Map(geos.map((g) => [g.panel.id, g]));
  const seams = sew.seams.map((seam): SeamGeo => ({
    seam,
    a: chainOf(byId, seam.a),
    b: chainOf(byId, seam.b),
    color: seamColor(seam.id),
    mismatch: sew.bad.get(seam.id) ?? null,
    info: sew.info.get(seam.id),
  }));
  const end = (e: Tack['a']): Vec | null => {
    const geo = e.panel === null ? undefined : byId.get(e.panel);
    return geo ? place(geo.panel, e.point) : null;
  };
  const tacks = sew.tacks.map((tack): TackGeo => ({ tack, a: end(tack.a), b: end(tack.b) }));
  return { seams, tacks };
}

export function seamAt(scene: SewScene, w: Vec, tol: number): { seam: SeamGeo; side: 'a' | 'b'; f: number; d: number } | null {
  let best: { seam: SeamGeo; side: 'a' | 'b'; f: number; d: number } | null = null;
  for (const sg of scene.seams) {
    for (const side of ['a', 'b'] as const) {
      const r = nearestOnChain(side === 'a' ? sg.a : sg.b, w);
      if (r.d <= tol && (!best || r.d < best.d)) best = { seam: sg, side, f: r.f, d: r.d };
    }
  }
  return best;
}

export function tackAt(scene: SewScene, w: Vec, tol: number): TackGeo | null {
  let best: TackGeo | null = null;
  let bd = Infinity;
  for (const tg of scene.tacks) {
    for (const p of [tg.a, tg.b]) {
      if (!p) continue;
      const d = dist(p, w);
      if (d <= tol && d < bd) { bd = d; best = tg; }
    }
  }
  return best;
}

const wrap = (v: number, n: number): number => ((v % n) + n) % n;

export function spansBetween(n: number, p0: number, p1: number): Run[] {
  const out: Run[] = [];
  if (n <= 0) return out;
  const dir = p1 >= p0 ? 1 : -1;
  let p = p0;
  let guard = 0;
  while (Math.abs(p1 - p) > 1e-6 && guard++ < n * 2 + 4) {
    const seg = dir > 0 ? Math.floor(p + 1e-9) : Math.ceil(p - 1e-9) - 1;
    const edge = dir > 0 ? seg + 1 : seg;
    const stop = dir > 0 ? Math.min(edge, p1) : Math.max(edge, p1);
    out.push({ segment: wrap(seg, n), t0: p - seg, t1: stop - seg });
    p = stop;
  }
  return out;
}

export function lineSpan(geos: Map<number, PanelGeo>, ends: SeamEnd[]): LineSpan | null {
  if (!ends.length) return null;
  const f = ends[0];
  const geo = geos.get(f.panel);
  const n = lineSegs(geo, f.inner).length;
  if (!geo || n === 0) return null;
  const closed = lineClosed(geo, f.inner);
  const p0 = f.segment + f.t0;
  let p = p0;
  for (let i = 0; i < ends.length; i++) {
    const e = ends[i];
    if (e.panel !== f.panel || e.inner !== f.inner) return null;
    if (i > 0) {
      const gap = p - (e.segment + e.t0);
      const off = closed ? Math.abs(wrap(gap + n / 2, n) - n / 2) : Math.abs(gap);
      if (off > 1e-4) return null;
    }
    p += e.t1 - e.t0;
  }
  return { ref: { panel: f.panel, inner: f.inner }, n, closed, p0, p1: p };
}

export class Tracker {
  p: number;
  private last: number;

  constructor(private readonly start: number, private readonly n: number, private readonly closed: boolean) {
    this.p = start;
    this.last = start;
  }

  feed(raw: number): number {
    if (!this.closed) {
      this.p = Math.min(this.n, Math.max(0, raw));
    } else {
      let d = raw - this.last;
      if (d > this.n / 2) d -= this.n;
      else if (d < -this.n / 2) d += this.n;
      this.p = Math.min(this.start + this.n, Math.max(this.start - this.n, this.p + d));
    }
    this.last = raw;
    return this.p;
  }
}

export function runsToEnds(ref: LineRef, runs: Run[]): SeamEnd[] {
  return runs.map((r) => ({ panel: ref.panel, inner: ref.inner, segment: r.segment, t0: r.t0, t1: r.t1 }));
}

export function walkSign(ends: SeamEnd[]): number {
  let sum = 0;
  for (const e of ends) sum += e.t1 - e.t0;
  return sum < 0 ? -1 : 1;
}
