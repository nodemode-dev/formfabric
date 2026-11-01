import { flatten, segsOf } from '../geom';
import type { Vec } from '../types';
import type { MeshData, PanelFull } from './model';

const CURVE_STEPS = 14;

function polygonOf(panel: PanelFull): Vec[] {
  const neutral = { ...panel, rotation: 0, mirror: false, position: [0, 0] as Vec };
  const segs = segsOf(neutral, panel.outline, true);
  const out: Vec[] = panel.outline.length ? [panel.outline[0].p] : [];
  for (const s of segs) flatten(s, CURVE_STEPS, out);
  const poly: Vec[] = [];
  for (const p of out) {
    const last = poly[poly.length - 1];
    if (!last || Math.hypot(p[0] - last[0], p[1] - last[1]) > 1e-6) poly.push(p);
  }
  while (poly.length > 1 && Math.hypot(poly[0][0] - poly[poly.length - 1][0], poly[0][1] - poly[poly.length - 1][1]) <= 1e-6) poly.pop();
  return poly;
}

function area(poly: Vec[]): number {
  let a = 0;
  for (let i = 0, j = poly.length - 1; i < poly.length; j = i++) a += poly[j][0] * poly[i][1] - poly[i][0] * poly[j][1];
  return a / 2;
}

function cross(ax: number, ay: number, bx: number, by: number, cx: number, cy: number): number {
  return (bx - ax) * (cy - ay) - (by - ay) * (cx - ax);
}

function inside(px: number, py: number, a: Vec, b: Vec, c: Vec): boolean {
  const d1 = cross(a[0], a[1], b[0], b[1], px, py);
  const d2 = cross(b[0], b[1], c[0], c[1], px, py);
  const d3 = cross(c[0], c[1], a[0], a[1], px, py);
  return d1 >= -1e-9 && d2 >= -1e-9 && d3 >= -1e-9;
}

export function triangulatePolygon(poly: Vec[]): Uint32Array | null {
  const n = poly.length;
  if (n < 3) return null;
  const ccw = area(poly) > 0;
  const order = new Int32Array(n);
  for (let i = 0; i < n; i++) order[i] = ccw ? i : n - 1 - i;
  const prev = new Int32Array(n);
  const next = new Int32Array(n);
  for (let i = 0; i < n; i++) {
    prev[i] = (i + n - 1) % n;
    next[i] = (i + 1) % n;
  }
  const out: number[] = [];
  let remaining = n;
  let cur = 0;
  let guard = 0;
  while (remaining > 3 && guard < n * n) {
    guard++;
    const ip = prev[cur];
    const inx = next[cur];
    const a = poly[order[ip]];
    const b = poly[order[cur]];
    const c = poly[order[inx]];
    let ear = cross(a[0], a[1], b[0], b[1], c[0], c[1]) > 1e-9;
    if (ear) {
      for (let k = next[inx]; k !== ip; k = next[k]) {
        const p = poly[order[k]];
        if (inside(p[0], p[1], a, b, c) && !(p[0] === a[0] && p[1] === a[1]) && !(p[0] === c[0] && p[1] === c[1])) { ear = false; break; }
      }
    }
    if (ear) {
      out.push(order[ip], order[cur], order[inx]);
      next[ip] = inx;
      prev[inx] = ip;
      remaining--;
      cur = ip;
      guard = 0;
    } else cur = inx;
  }
  if (remaining === 3) {
    const ip = prev[cur];
    out.push(order[ip], order[cur], order[next[cur]]);
  } else {
    const start = cur;
    let k = next[start];
    let rest = remaining;
    while (rest > 2) {
      out.push(order[start], order[k], order[next[k]]);
      k = next[k];
      rest--;
    }
  }
  return Uint32Array.from(out);
}

export function outlineMesh(panel: PanelFull): MeshData | null {
  const poly = polygonOf(panel);
  const tri = triangulatePolygon(poly);
  if (!tri) return null;
  const local = new Float32Array(poly.length * 2);
  for (let i = 0; i < poly.length; i++) {
    local[i * 2] = poly[i][0];
    local[i * 2 + 1] = poly[i][1];
  }
  return { local, indices: tri };
}
