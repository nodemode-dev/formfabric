import { geoOf } from '../geom';
import type { Garment, Vec } from '../types';
import { atLength, sceneOf, walkOf, type LocalRun } from './scene';
import { num } from './types';

export interface PanelSurface { local: Float32Array; idx: Uint32Array; pos: Float32Array }
export type SurfaceOf = (panel: number) => PanelSurface | null;

export type TrimLineLayer = 'topstitch' | 'puckering' | 'fray' | 'zipper' | 'zipperOpen' | 'finish' | 'buttonhole';
export type TrimPointLayer = 'button' | 'trim';

export interface TrimLines { layer: TrimLineLayer; rgb: [number, number, number]; positions: Float32Array }
export interface TrimPoints { layer: TrimPointLayer; rgb: [number, number, number]; positions: Float32Array; diameters: Float32Array }
export interface TrimBuffers { lines: TrimLines[]; points: TrimPoints[] }

const LINE_RGB: Record<TrimLineLayer, [number, number, number]> = {
  topstitch: [0.93, 0.93, 0.9],
  puckering: [0.72, 0.72, 0.72],
  fray: [0.95, 0.78, 0.25],
  zipper: [0.35, 0.82, 0.92],
  zipperOpen: [0.35, 0.82, 0.92],
  finish: [0.96, 0.66, 0.26],
  buttonhole: [0.12, 0.12, 0.16],
};

const POINT_RGB: Record<TrimPointLayer, [number, number, number]> = {
  button: [0.95, 0.95, 0.92],
  trim: [0.8, 0.8, 0.85],
};

interface Grid { cell: number; x0: number; y0: number; nx: number; ny: number; cells: Map<number, number[]>; tris: number }

const grids = new WeakMap<Float32Array, Grid>();

function gridOf(s: PanelSurface): Grid {
  const hit = grids.get(s.local);
  const tris = s.idx.length / 3;
  if (hit && hit.tris === tris) return hit;
  let x0 = Infinity;
  let y0 = Infinity;
  let x1 = -Infinity;
  let y1 = -Infinity;
  for (let i = 0; i < s.local.length; i += 2) {
    const x = s.local[i];
    const y = s.local[i + 1];
    if (x < x0) x0 = x;
    if (x > x1) x1 = x;
    if (y < y0) y0 = y;
    if (y > y1) y1 = y;
  }
  const side = Math.max(1, Math.ceil(Math.sqrt(tris)));
  const cell = Math.max((x1 - x0) / side, (y1 - y0) / side, 1e-3);
  const nx = Math.max(1, Math.ceil((x1 - x0) / cell));
  const ny = Math.max(1, Math.ceil((y1 - y0) / cell));
  const cells = new Map<number, number[]>();
  for (let t = 0; t < tris; t++) {
    let tx0 = Infinity;
    let ty0 = Infinity;
    let tx1 = -Infinity;
    let ty1 = -Infinity;
    for (let k = 0; k < 3; k++) {
      const v = s.idx[t * 3 + k];
      const x = s.local[v * 2];
      const y = s.local[v * 2 + 1];
      if (x < tx0) tx0 = x;
      if (x > tx1) tx1 = x;
      if (y < ty0) ty0 = y;
      if (y > ty1) ty1 = y;
    }
    const cx0 = Math.max(0, Math.floor((tx0 - x0) / cell));
    const cx1 = Math.min(nx - 1, Math.floor((tx1 - x0) / cell));
    const cy0 = Math.max(0, Math.floor((ty0 - y0) / cell));
    const cy1 = Math.min(ny - 1, Math.floor((ty1 - y0) / cell));
    for (let cy = cy0; cy <= cy1; cy++) {
      for (let cx = cx0; cx <= cx1; cx++) {
        const key = cy * nx + cx;
        const list = cells.get(key);
        if (list) list.push(t); else cells.set(key, [t]);
      }
    }
  }
  const grid: Grid = { cell, x0, y0, nx, ny, cells, tris };
  grids.set(s.local, grid);
  return grid;
}

export function lift(s: PanelSurface, p: Vec): [number, number, number] | null {
  const g = gridOf(s);
  const cx = Math.min(g.nx - 1, Math.max(0, Math.floor((p[0] - g.x0) / g.cell)));
  const cy = Math.min(g.ny - 1, Math.max(0, Math.floor((p[1] - g.y0) / g.cell)));
  let best = -1;
  let bestScore = Infinity;
  let bw: [number, number, number] = [0, 0, 0];
  for (let dy = -1; dy <= 1; dy++) {
    for (let dx = -1; dx <= 1; dx++) {
      const x = cx + dx;
      const y = cy + dy;
      if (x < 0 || y < 0 || x >= g.nx || y >= g.ny) continue;
      const list = g.cells.get(y * g.nx + x);
      if (!list) continue;
      for (const t of list) {
        const a = s.idx[t * 3] * 2;
        const b = s.idx[t * 3 + 1] * 2;
        const c = s.idx[t * 3 + 2] * 2;
        const ax = s.local[a];
        const ay = s.local[a + 1];
        const v0x = s.local[b] - ax;
        const v0y = s.local[b + 1] - ay;
        const v1x = s.local[c] - ax;
        const v1y = s.local[c + 1] - ay;
        const v2x = p[0] - ax;
        const v2y = p[1] - ay;
        const den = v0x * v1y - v1x * v0y;
        if (Math.abs(den) < 1e-12) continue;
        const u = (v2x * v1y - v1x * v2y) / den;
        const v = (v0x * v2y - v2x * v0y) / den;
        const score = -Math.min(u, v, 1 - u - v);
        if (score < bestScore) { bestScore = score; best = t; bw = [1 - u - v, u, v]; }
      }
    }
  }
  if (best < 0) return null;
  const w0 = Math.max(0, bw[0]);
  const w1 = Math.max(0, bw[1]);
  const w2 = Math.max(0, bw[2]);
  const sum = w0 + w1 + w2 || 1;
  const out: [number, number, number] = [0, 0, 0];
  const ws = [w0 / sum, w1 / sum, w2 / sum];
  for (let k = 0; k < 3; k++) {
    const o = s.idx[best * 3 + k] * 3;
    out[0] += s.pos[o] * ws[k];
    out[1] += s.pos[o + 1] * ws[k];
    out[2] += s.pos[o + 2] * ws[k];
  }
  return out;
}

class Lines {
  private readonly data = new Map<TrimLineLayer, number[]>();

  segment(layer: TrimLineLayer, a: [number, number, number] | null, b: [number, number, number] | null): void {
    if (!a || !b) return;
    let list = this.data.get(layer);
    if (!list) { list = []; this.data.set(layer, list); }
    list.push(a[0], a[1], a[2], b[0], b[1], b[2]);
  }

  done(): TrimLines[] {
    return [...this.data].map(([layer, list]) => ({ layer, rgb: LINE_RGB[layer], positions: Float32Array.from(list) }));
  }
}

function dashed(out: Lines, layer: TrimLineLayer, s: PanelSurface, pts: Vec[], on: number, off: number, from = 0, to = Infinity): void {
  const w = walkOf(pts);
  const end = Math.min(to, w.total);
  for (let at = from; at < end; at += on + off) {
    const a = atLength(w, at).p;
    const b = atLength(w, Math.min(end, at + on)).p;
    out.segment(layer, lift(s, a), lift(s, b));
  }
}

function runs(out: Lines, layer: TrimLineLayer, surface: SurfaceOf, list: LocalRun[], on: number, off: number, from = 0, to = Infinity): void {
  for (const r of list) {
    const s = surface(r.panel.id);
    if (s) dashed(out, layer, s, r.pts, on, off, from, to);
  }
}

export function trimBuffers(garment: Garment, surface: SurfaceOf): TrimBuffers {
  const scene = sceneOf(garment.panels.map(geoOf), garment);
  const lines = new Lines();
  for (const st of scene.stitches) {
    const p = st.style?.params;
    if (st.mode === 'topstitch') {
      const len = 25.4 / Math.max(1, num(p, 'spi', 10));
      runs(lines, 'topstitch', surface, st.local, len * 0.66, len * 0.34);
    } else if (st.mode === 'puckering') runs(lines, 'puckering', surface, st.local, 4, 0.01);
    else runs(lines, 'fray', surface, st.local, 1.2, 2.4);
  }
  for (const f of scene.finishes) runs(lines, 'finish', surface, f.local, 6, 0.01);
  for (const z of scene.zippers) {
    const closed = Math.min(1, Math.max(0, z.zipper.closed));
    for (const side of [z.localA, z.localB]) {
      const total = side.reduce((sum, r) => sum + walkOf(r.pts).total, 0);
      let used = 0;
      for (const r of side) {
        const s = surface(r.panel.id);
        const len = walkOf(r.pts).total;
        if (s) {
          const cut = Math.max(0, Math.min(len, total * closed - used));
          dashed(lines, 'zipper', s, r.pts, 3, 0.01, 0, cut);
          if (cut < len) dashed(lines, 'zipperOpen', s, r.pts, 3, 3, cut, len);
        }
        used += len;
      }
    }
  }
  const pointData = new Map<TrimPointLayer, { pos: number[]; dia: number[] }>();
  for (const h of scene.hardware) {
    const s = surface(h.panel.id);
    if (!s) continue;
    const at = h.hw.anchor.point;
    const kind = h.style?.kind;
    if (kind === 'buttonhole') {
      const r = (h.hw.rotation * Math.PI) / 180;
      const d: Vec = [Math.cos(r) * h.length * 0.5, Math.sin(r) * h.length * 0.5];
      lines.segment('buttonhole', lift(s, [at[0] - d[0], at[1] - d[1]]), lift(s, [at[0] + d[0], at[1] + d[1]]));
      continue;
    }
    const p = lift(s, at);
    if (!p) continue;
    const layer: TrimPointLayer = kind === 'trim' ? 'trim' : 'button';
    let entry = pointData.get(layer);
    if (!entry) { entry = { pos: [], dia: [] }; pointData.set(layer, entry); }
    entry.pos.push(p[0], p[1], p[2]);
    entry.dia.push(h.width);
  }
  const points = [...pointData].map(([layer, e]): TrimPoints => ({ layer, rgb: POINT_RGB[layer], positions: Float32Array.from(e.pos), diameters: Float32Array.from(e.dia) }));
  return { lines: lines.done(), points };
}
