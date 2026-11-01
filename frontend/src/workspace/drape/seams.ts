import type { SeamLine } from './types';

export interface MeshInfo {
  key: string;
  segments: number;
  boundary: number[];
  param: number[][];
  positions: number[][];
}

interface Span {
  panel: number;
  inner?: number | null;
  segment: number;
  t0: number;
  t1: number;
}

export interface SeamJson {
  id: number;
  a: Span[];
  b: Span[];
  reversed?: boolean;
}

const EPS = 1e-4;
const MAX_LINES = 6000;

function spanVertices(s: Span, mesh: MeshInfo): number[] {
  if (s.inner != null || mesh.segments === 0) return [];
  const lo = Math.min(s.t0, s.t1);
  const hi = Math.max(s.t0, s.t1);
  const next = (s.segment + 1) % mesh.segments;
  const found: Array<[number, number]> = [];
  for (let i = 0; i < mesh.boundary.length; i++) {
    const [seg, t] = mesh.param[i];
    if (seg === s.segment && t >= lo - EPS && t <= hi + EPS) found.push([t, mesh.boundary[i]]);
    else if (seg === next && t <= EPS && hi >= 1 - EPS) found.push([1, mesh.boundary[i]]);
  }
  found.sort((x, y) => x[0] - y[0]);
  if (s.t0 > s.t1) found.reverse();
  return found.map((f) => f[1]);
}

function sideVertices(spans: Span[], meshes: Map<number, MeshInfo>): Array<[number, number]> {
  const out: Array<[number, number]> = [];
  for (const s of spans) {
    const mesh = meshes.get(s.panel);
    if (!mesh) continue;
    for (const v of spanVertices(s, mesh)) {
      const last = out[out.length - 1];
      if (!last || last[0] !== s.panel || last[1] !== v) out.push([s.panel, v]);
    }
  }
  return out;
}

export function seamLines(seams: SeamJson[], meshes: Map<number, MeshInfo>): SeamLine[] {
  const lines: SeamLine[] = [];
  for (const seam of seams) {
    const a = sideVertices(seam.a, meshes);
    let b = sideVertices(seam.b, meshes);
    if (a.length === 0 || b.length === 0) continue;
    if (seam.reversed !== false) b = b.slice().reverse();
    const n = a.length;
    const m = b.length;
    for (let k = 0; k < n; k++) {
      const j = n === 1 ? 0 : Math.round((k * (m - 1)) / (n - 1));
      lines.push({ pa: a[k][0], va: a[k][1], pb: b[j][0], vb: b[j][1] });
      if (lines.length >= MAX_LINES) return lines;
    }
  }
  return lines;
}
