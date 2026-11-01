import type { Frame } from './frame';
import type { PanelBuf, Scene } from './scene';

export class Picker {
  private readonly o = new Float64Array(3);
  private readonly d = new Float64Array(3);
  panel = -1;
  vertex = -1;
  distance = 0;
  private hitTri = -1;
  private hitU = 0;
  private hitV = 0;

  pick(f: Frame, scene: Scene, px: number, py: number): number {
    this.panel = -1;
    this.vertex = -1;
    this.hitTri = -1;
    if (!f.ray(px, py, this.o, this.d)) return -1;
    const ox = this.o[0], oy = this.o[1], oz = this.o[2];
    const dx = this.d[0], dy = this.d[1], dz = this.d[2];
    let best = Infinity;
    const list = scene.list;
    for (let i = 0; i < list.length; i++) {
      const b = list[i];
      if (!b.ready || b.kind === 'body' || !this.box(b, ox, oy, oz, dx, dy, dz, best)) continue;
      const t = this.mesh(b, ox, oy, oz, dx, dy, dz, best);
      if (t < best) {
        best = t;
        this.panel = b.id;
      }
    }
    this.distance = best;
    if (this.panel >= 0 && this.hitTri >= 0) {
      const b = scene.get(this.panel);
      if (b) {
        const w = 1 - this.hitU - this.hitV;
        const k = this.hitTri * 3;
        this.vertex = this.hitU > w && this.hitU >= this.hitV ? b.idx[k + 1] : this.hitV > w && this.hitV > this.hitU ? b.idx[k + 2] : b.idx[k];
      }
    }
    return this.panel;
  }

  private box(b: PanelBuf, ox: number, oy: number, oz: number, dx: number, dy: number, dz: number, limit: number): boolean {
    let t0 = 0;
    let t1 = limit;
    const pad = 1e-3;
    for (let k = 0; k < 3; k++) {
      const o = k === 0 ? ox : k === 1 ? oy : oz;
      const d = k === 0 ? dx : k === 1 ? dy : dz;
      const lo = b.min[k] - pad;
      const hi = b.max[k] + pad;
      if (Math.abs(d) < 1e-12) {
        if (o < lo || o > hi) return false;
        continue;
      }
      let a = (lo - o) / d;
      let c = (hi - o) / d;
      if (a > c) { const s = a; a = c; c = s; }
      if (a > t0) t0 = a;
      if (c < t1) t1 = c;
      if (t0 > t1) return false;
    }
    return true;
  }

  private mesh(b: PanelBuf, ox: number, oy: number, oz: number, dx: number, dy: number, dz: number, limit: number): number {
    const p = b.pos;
    const idx = b.idx;
    let best = limit;
    for (let t = 0; t < idx.length; t += 3) {
      const a = idx[t] * 3;
      const bb = idx[t + 1] * 3;
      const c = idx[t + 2] * 3;
      const e1x = p[bb] - p[a], e1y = p[bb + 1] - p[a + 1], e1z = p[bb + 2] - p[a + 2];
      const e2x = p[c] - p[a], e2y = p[c + 1] - p[a + 1], e2z = p[c + 2] - p[a + 2];
      const hx = dy * e2z - dz * e2y;
      const hy = dz * e2x - dx * e2z;
      const hz = dx * e2y - dy * e2x;
      const det = e1x * hx + e1y * hy + e1z * hz;
      if (det > -1e-12 && det < 1e-12) continue;
      const inv = 1 / det;
      const sx = ox - p[a], sy = oy - p[a + 1], sz = oz - p[a + 2];
      const u = (sx * hx + sy * hy + sz * hz) * inv;
      if (u < 0 || u > 1) continue;
      const qx = sy * e1z - sz * e1y;
      const qy = sz * e1x - sx * e1z;
      const qz = sx * e1y - sy * e1x;
      const v = (dx * qx + dy * qy + dz * qz) * inv;
      if (v < 0 || u + v > 1) continue;
      const tt = (e2x * qx + e2y * qy + e2z * qz) * inv;
      if (tt > 1e-6 && tt < best) {
        best = tt;
        this.hitTri = t / 3;
        this.hitU = u;
        this.hitV = v;
      }
    }
    return best;
  }
}
