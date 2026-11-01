import type { Frame } from './frame';

export interface Handle {
  kind: 'move' | 'rotate';
  axis: 0 | 1 | 2;
}

const AXIS_CSS = ['#dc5252', '#66c761', '#528ce6'];
const RING_STEPS = 48;
const ARROW_PX = 96;
const RING_FRAC = 0.78;

export class Gizmo {
  readonly center = new Float64Array(3);
  len = 1;
  hot: Handle | null = null;
  drag: { handle: Handle; x: number; y: number; degrees: number } | null = null;
  readonly translate = new Float64Array(3);
  private readonly c = new Float64Array(3);
  private readonly t = new Float64Array(3);
  private readonly q = new Float64Array(3);
  private readonly ring = new Float64Array(2 * (RING_STEPS + 1));

  place(cx: number, cy: number, cz: number, worldPerPixel: number): void {
    this.center[0] = cx;
    this.center[1] = cy;
    this.center[2] = cz;
    this.len = worldPerPixel * ARROW_PX;
  }

  shown(f: Frame): boolean {
    return f.project(this.center[0] + this.translate[0], this.center[1] + this.translate[1], this.center[2] + this.translate[2], this.c);
  }

  private axisTip(f: Frame, axis: number, out: Float64Array): boolean {
    const x = this.center[0] + this.translate[0] + (axis === 0 ? this.len : 0);
    const y = this.center[1] + this.translate[1] + (axis === 1 ? this.len : 0);
    const z = this.center[2] + this.translate[2] + (axis === 2 ? this.len : 0);
    return f.project(x, y, z, out);
  }

  private buildRing(f: Frame, axis: number): number {
    const r = this.len * RING_FRAC;
    const u = (axis + 1) % 3;
    const v = (axis + 2) % 3;
    let n = 0;
    for (let i = 0; i <= RING_STEPS; i++) {
      const a = (i / RING_STEPS) * Math.PI * 2;
      const w0 = this.center[0] + this.translate[0];
      const w1 = this.center[1] + this.translate[1];
      const w2 = this.center[2] + this.translate[2];
      const cu = Math.cos(a) * r;
      const cv = Math.sin(a) * r;
      const wx = w0 + (u === 0 ? cu : v === 0 ? cv : 0);
      const wy = w1 + (u === 1 ? cu : v === 1 ? cv : 0);
      const wz = w2 + (u === 2 ? cu : v === 2 ? cv : 0);
      if (!f.project(wx, wy, wz, this.q)) continue;
      this.ring[n * 2] = this.q[0];
      this.ring[n * 2 + 1] = this.q[1];
      n++;
    }
    return n;
  }

  draw(ctx: CanvasRenderingContext2D, f: Frame, dpr: number): void {
    if (!f.project(this.center[0] + this.translate[0], this.center[1] + this.translate[1], this.center[2] + this.translate[2], this.c)) return;
    ctx.lineWidth = 2 * dpr;
    ctx.lineCap = 'round';
    ctx.lineJoin = 'round';
    for (let a = 0; a < 3; a++) {
      const hotRing = this.is('rotate', a);
      const hotMove = this.is('move', a);
      const n = this.buildRing(f, a);
      ctx.strokeStyle = AXIS_CSS[a];
      ctx.globalAlpha = hotRing ? 1 : 0.7;
      ctx.lineWidth = (hotRing ? 3 : 2) * dpr;
      if (n > 1) {
        ctx.beginPath();
        ctx.moveTo(this.ring[0] * dpr, this.ring[1] * dpr);
        for (let i = 1; i < n; i++) ctx.lineTo(this.ring[i * 2] * dpr, this.ring[i * 2 + 1] * dpr);
        ctx.stroke();
      }
      if (!this.axisTip(f, a, this.t)) continue;
      ctx.globalAlpha = hotMove ? 1 : 0.9;
      ctx.lineWidth = (hotMove ? 3 : 2) * dpr;
      const ux = this.t[0] - this.c[0];
      const uy = this.t[1] - this.c[1];
      const l = Math.hypot(ux, uy) || 1;
      const nx = ux / l;
      const ny = uy / l;
      ctx.beginPath();
      ctx.moveTo(this.c[0] * dpr, this.c[1] * dpr);
      ctx.lineTo(this.t[0] * dpr, this.t[1] * dpr);
      ctx.moveTo((this.t[0] - nx * 10 - ny * 5) * dpr, (this.t[1] - ny * 10 + nx * 5) * dpr);
      ctx.lineTo(this.t[0] * dpr, this.t[1] * dpr);
      ctx.lineTo((this.t[0] - nx * 10 + ny * 5) * dpr, (this.t[1] - ny * 10 - nx * 5) * dpr);
      ctx.stroke();
    }
    ctx.globalAlpha = 1;
  }

  private is(kind: Handle['kind'], axis: number): boolean {
    const h = this.drag ? this.drag.handle : this.hot;
    return !!h && h.kind === kind && h.axis === axis;
  }

  hit(f: Frame, px: number, py: number): Handle | null {
    if (!f.project(this.center[0], this.center[1], this.center[2], this.c)) return null;
    let best = Infinity;
    let found: Handle | null = null;
    for (let a = 0; a < 3; a++) {
      if (!this.axisTip(f, a, this.t)) continue;
      if (Math.hypot(px - this.c[0], py - this.c[1]) <= 10) continue;
      const d = segDist(px, py, this.c[0], this.c[1], this.t[0], this.t[1]);
      if (d <= 7 && d < best) {
        best = d;
        found = { kind: 'move', axis: a as 0 | 1 | 2 };
      }
    }
    if (found) return found;
    for (let a = 0; a < 3; a++) {
      const n = this.buildRing(f, a);
      for (let i = 1; i < n; i++) {
        const d = segDist(px, py, this.ring[(i - 1) * 2], this.ring[(i - 1) * 2 + 1], this.ring[i * 2], this.ring[i * 2 + 1]);
        if (d <= 6 && d < best) {
          best = d;
          found = { kind: 'rotate', axis: a as 0 | 1 | 2 };
        }
      }
    }
    return found;
  }

  begin(handle: Handle, x: number, y: number): void {
    this.drag = { handle, x, y, degrees: 0 };
    this.translate.fill(0);
  }

  update(f: Frame, px: number, py: number): void {
    const d = this.drag;
    if (!d) return;
    if (!f.project(this.center[0], this.center[1], this.center[2], this.c)) return;
    const h = d.handle;
    const dx = px - d.x;
    const dy = py - d.y;
    if (h.kind === 'move') {
      const saved0 = this.translate[0];
      const saved1 = this.translate[1];
      const saved2 = this.translate[2];
      this.translate.fill(0);
      const ok = this.axisTip(f, h.axis, this.t);
      this.translate[0] = saved0;
      this.translate[1] = saved1;
      this.translate[2] = saved2;
      if (!ok) return;
      const sx = this.t[0] - this.c[0];
      const sy = this.t[1] - this.c[1];
      const s2 = sx * sx + sy * sy || 1;
      const along = ((dx * sx + dy * sy) / s2) * this.len;
      this.translate.fill(0);
      this.translate[h.axis] = along;
      return;
    }
    const a0 = Math.atan2(d.y - this.c[1], d.x - this.c[0]);
    const a1 = Math.atan2(py - this.c[1], px - this.c[0]);
    let deg = ((a1 - a0) * 180) / Math.PI;
    deg = ((deg + 540) % 360) - 180;
    const toward = h.axis === 0 ? f.eye[0] - this.center[0] : h.axis === 1 ? f.eye[1] - this.center[1] : f.eye[2] - this.center[2];
    d.degrees = deg * (toward >= 0 ? -1 : 1);
  }

  matrix(out: Float32Array): void {
    const d = this.drag;
    out.fill(0);
    out[0] = 1;
    out[5] = 1;
    out[10] = 1;
    out[15] = 1;
    if (!d) return;
    let r00 = 1, r01 = 0, r02 = 0, r10 = 0, r11 = 1, r12 = 0, r20 = 0, r21 = 0, r22 = 1;
    if (d.handle.kind === 'rotate') {
      const rad = (d.degrees * Math.PI) / 180;
      const cs = Math.cos(rad);
      const sn = Math.sin(rad);
      if (d.handle.axis === 0) { r11 = cs; r12 = -sn; r21 = sn; r22 = cs; }
      else if (d.handle.axis === 1) { r00 = cs; r02 = sn; r20 = -sn; r22 = cs; }
      else { r00 = cs; r01 = -sn; r10 = sn; r11 = cs; }
    }
    const cx = this.center[0], cy = this.center[1], cz = this.center[2];
    out[0] = r00; out[1] = r10; out[2] = r20;
    out[4] = r01; out[5] = r11; out[6] = r21;
    out[8] = r02; out[9] = r12; out[10] = r22;
    out[12] = cx + this.translate[0] - (r00 * cx + r01 * cy + r02 * cz);
    out[13] = cy + this.translate[1] - (r10 * cx + r11 * cy + r12 * cz);
    out[14] = cz + this.translate[2] - (r20 * cx + r21 * cy + r22 * cz);
  }

  end(): void {
    this.drag = null;
    this.translate.fill(0);
  }
}

function segDist(px: number, py: number, ax: number, ay: number, bx: number, by: number): number {
  const dx = bx - ax;
  const dy = by - ay;
  const t = Math.max(0, Math.min(1, ((px - ax) * dx + (py - ay) * dy) / (dx * dx + dy * dy || 1)));
  return Math.hypot(px - ax - dx * t, py - ay - dy * t);
}
