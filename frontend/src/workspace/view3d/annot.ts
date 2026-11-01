import type { TapeMark, WindJson } from '../drape/types';
import type { Frame } from './frame';
import type { Scene } from './scene';

const a = new Float64Array(3);
const b = new Float64Array(3);
const RADIAL: Array<[number, number, number]> = [[1, 0, 0], [-1, 0, 0], [0, 1, 0], [0, -1, 0], [0, 0, 1], [0, 0, -1]];

function arrow(ctx: CanvasRenderingContext2D, f: Frame, o: [number, number, number], d: [number, number, number], len: number, dpr: number): void {
  if (!f.project(o[0], o[1], o[2], a) || !f.project(o[0] + d[0] * len, o[1] + d[1] * len, o[2] + d[2] * len, b)) return;
  const x0 = a[0] * dpr;
  const y0 = a[1] * dpr;
  const x1 = b[0] * dpr;
  const y1 = b[1] * dpr;
  const dx = x1 - x0;
  const dy = y1 - y0;
  const l = Math.hypot(dx, dy);
  if (l < 2) return;
  const ux = dx / l;
  const uy = dy / l;
  const h = Math.min(9 * dpr, l * 0.5);
  ctx.beginPath();
  ctx.moveTo(x0, y0);
  ctx.lineTo(x1, y1);
  ctx.moveTo(x1 - ux * h - uy * h * 0.5, y1 - uy * h + ux * h * 0.5);
  ctx.lineTo(x1, y1);
  ctx.lineTo(x1 - ux * h + uy * h * 0.5, y1 - uy * h - ux * h * 0.5);
  ctx.stroke();
}

export function drawWind(ctx: CanvasRenderingContext2D, f: Frame, dpr: number, w: WindJson, worldPerPixel: number, color: string): void {
  if (!w.enabled) return;
  const len = worldPerPixel * 72;
  ctx.strokeStyle = color;
  ctx.fillStyle = color;
  ctx.lineWidth = 2 * dpr;
  ctx.lineCap = 'round';
  ctx.lineJoin = 'round';
  if (w.kind === 'radial') {
    for (const d of RADIAL) arrow(ctx, f, w.origin, d, len * 0.6, dpr);
  } else {
    const n = Math.hypot(w.direction[0], w.direction[1], w.direction[2]);
    if (n < 1e-9) return;
    arrow(ctx, f, w.origin, [w.direction[0] / n, w.direction[1] / n, w.direction[2] / n], len, dpr);
  }
  if (f.project(w.origin[0], w.origin[1], w.origin[2], a)) {
    ctx.beginPath();
    ctx.arc(a[0] * dpr, a[1] * dpr, 3.5 * dpr, 0, Math.PI * 2);
    ctx.fill();
  }
}

export function drawTapes(ctx: CanvasRenderingContext2D, f: Frame, scene: Scene, tapes: TapeMark[], dpr: number, color: string, ink: string, format: (mm: number) => string): void {
  if (tapes.length === 0) return;
  ctx.strokeStyle = color;
  ctx.lineWidth = 1.5 * dpr;
  ctx.setLineDash([5 * dpr, 3 * dpr]);
  ctx.font = `${Math.round(11 * dpr)}px ui-monospace, monospace`;
  ctx.textBaseline = 'middle';
  for (let i = 0; i < tapes.length; i++) {
    const t = tapes[i];
    const buf = scene.get(t.panel);
    const v = t.vertex;
    if (!buf || !buf.ready || v * 3 + 2 >= buf.pos.length) continue;
    const px = buf.pos[v * 3];
    const py = buf.pos[v * 3 + 1];
    const pz = buf.pos[v * 3 + 2];
    if (!f.project(px, py, pz, a) || !f.project(t.point[0], t.point[1], t.point[2], b)) continue;
    ctx.beginPath();
    ctx.moveTo(a[0] * dpr, a[1] * dpr);
    ctx.lineTo(b[0] * dpr, b[1] * dpr);
    ctx.stroke();
    const text = format(Math.hypot(px - t.point[0], py - t.point[1], pz - t.point[2]));
    const mx = (a[0] + b[0]) * 0.5 * dpr;
    const my = (a[1] + b[1]) * 0.5 * dpr;
    const wd = ctx.measureText(text).width;
    ctx.setLineDash([]);
    ctx.globalAlpha = 0.85;
    ctx.fillStyle = color;
    ctx.fillRect(mx - wd / 2 - 4 * dpr, my - 8 * dpr, wd + 8 * dpr, 16 * dpr);
    ctx.globalAlpha = 1;
    ctx.fillStyle = ink;
    ctx.fillText(text, mx - wd / 2, my);
    ctx.setLineDash([5 * dpr, 3 * dpr]);
  }
  ctx.setLineDash([]);
}
