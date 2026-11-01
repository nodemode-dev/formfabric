import type { PanelGeo, Seg } from '../geom';
import type { Colors } from '../pattern-view/ctx';
import type { Vec } from '../types';
import { imageFailed, imageFor } from './images';
import { atLength, walkOf, type FinishGeo, type HwGeo, type PrintGeo, type StitchGeo, type TrimScene, type ZipGeo } from './scene';
import { num, str, type FinishKind, type TrimItem } from './types';

type Map2 = (x: number, y: number) => [number, number];

export function finishHue(kind: FinishKind): string {
  const h: Record<FinishKind, number> = { piping: 28, binding: 205, elastic: 330, tape: 150, bond: 52, press: 268 };
  return `hsl(${h[kind]}, 72%, 62%)`;
}

export const ZIPPER_HUE = 'hsl(188, 70%, 58%)';

const on = (picked: TrimItem | null, kind: TrimItem['kind'], id: number): boolean => picked !== null && picked.kind === kind && picked.id === id;

function trace(g: CanvasRenderingContext2D, pts: Vec[], map: Map2): void {
  pts.forEach((p, i) => {
    const s = map(p[0], p[1]);
    if (i === 0) g.moveTo(s[0], s[1]);
    else g.lineTo(s[0], s[1]);
  });
}

function outline(g: CanvasRenderingContext2D, segs: Seg[], map: Map2): void {
  if (!segs.length) return;
  const s0 = map(segs[0].a[0], segs[0].a[1]);
  g.moveTo(s0[0], s0[1]);
  for (const s of segs) {
    const b = map(s.b[0], s.b[1]);
    if (s.curved) {
      const c1 = map(s.c1[0], s.c1[1]);
      const c2 = map(s.c2[0], s.c2[1]);
      g.bezierCurveTo(c1[0], c1[1], c2[0], c2[1], b[0], b[1]);
    } else g.lineTo(b[0], b[1]);
  }
  g.closePath();
}

function halo(g: CanvasRenderingContext2D, lines: Vec[][], map: Map2, c: Colors, width: number): void {
  g.save();
  g.globalAlpha = 0.3;
  g.strokeStyle = c.accent;
  g.lineWidth = width + 6;
  g.lineCap = 'round';
  g.setLineDash([]);
  g.beginPath();
  for (const l of lines) trace(g, l, map);
  g.stroke();
  g.restore();
}

export function drawPrints(g: CanvasRenderingContext2D, geo: PanelGeo, prints: PrintGeo[], map: Map2, zoom: number, c: Colors, picked: TrimItem | null): void {
  const [ox, oy] = map(0, 0);
  const panel = geo.panel;
  g.save();
  g.beginPath();
  outline(g, geo.outline, map);
  g.clip();
  for (const { print } of prints) {
    g.save();
    g.translate(ox, oy);
    g.scale(zoom, -zoom);
    g.translate(panel.position[0], panel.position[1]);
    g.rotate((panel.rotation * Math.PI) / 180);
    if (panel.mirror) g.scale(-1, 1);
    g.translate(print.position[0], print.position[1]);
    g.rotate((print.rotation * Math.PI) / 180);
    g.scale(1, -1);
    const w = print.size[0];
    const h = print.size[1];
    const img = imageFor(print.image);
    g.globalAlpha = print.side === 'back' ? 0.4 : 1;
    if (img) g.drawImage(img, -w / 2, -h / 2, w, h);
    else {
      g.fillStyle = c.muted;
      g.globalAlpha = 0.14;
      g.fillRect(-w / 2, -h / 2, w, h);
      g.globalAlpha = 0.6;
      g.strokeStyle = c.muted;
      g.lineWidth = 1 / zoom;
      g.setLineDash([4 / zoom, 3 / zoom]);
      g.strokeRect(-w / 2, -h / 2, w, h);
      if (imageFailed(print.image)) {
        g.setLineDash([]);
        g.beginPath();
        g.moveTo(-w / 2, -h / 2);
        g.lineTo(w / 2, h / 2);
        g.moveTo(w / 2, -h / 2);
        g.lineTo(-w / 2, h / 2);
        g.stroke();
      }
    }
    if (on(picked, 'print', print.id)) {
      g.globalAlpha = 1;
      g.setLineDash([]);
      g.strokeStyle = c.accent;
      g.lineWidth = 1.6 / zoom;
      g.strokeRect(-w / 2, -h / 2, w, h);
    }
    g.restore();
  }
  g.restore();
}

function drawFinish(g: CanvasRenderingContext2D, f: FinishGeo, map: Map2, zoom: number, c: Colors, picked: TrimItem | null): void {
  const fin = f.finish;
  const color = finishHue(fin.kind);
  const band = fin.kind === 'tape' || fin.kind === 'bond' || fin.kind === 'press';
  const px = band ? Math.max(6, fin.width * 2 * zoom) : fin.kind === 'piping' ? Math.max(3, fin.width * zoom * 0.8) : Math.max(4, fin.width * zoom * (fin.kind === 'binding' ? 1 : 0.6));
  const selected = on(picked, 'finish', fin.id);
  if (selected) halo(g, f.lines, map, c, px);
  g.save();
  g.lineCap = 'round';
  g.lineJoin = 'round';
  g.strokeStyle = color;
  g.lineWidth = px;
  g.globalAlpha = band ? 0.2 : fin.kind === 'piping' ? 0.85 : 0.5;
  g.setLineDash(fin.kind === 'elastic' ? [2, 3] : []);
  g.beginPath();
  for (const l of f.lines) trace(g, l, map);
  g.stroke();
  g.globalAlpha = 0.95;
  g.lineWidth = fin.kind === 'piping' ? 1 : 1.2;
  g.strokeStyle = fin.kind === 'piping' ? c.bg : color;
  g.setLineDash(fin.kind === 'bond' ? [1.5, 3] : fin.kind === 'press' ? [8, 3] : []);
  g.beginPath();
  for (const l of f.lines) trace(g, l, map);
  g.stroke();
  g.restore();
}

function drawStitch(g: CanvasRenderingContext2D, s: StitchGeo, map: Map2, zoom: number, c: Colors, picked: TrimItem | null): void {
  const p = s.style?.params;
  const selected = on(picked, 'stitch', s.stitch.id);
  g.save();
  g.lineCap = 'butt';
  g.lineJoin = 'round';
  if (s.mode === 'topstitch') {
    const len = (25.4 / Math.max(1, num(p, 'spi', 10))) * zoom;
    const px = Math.min(2.6, Math.max(1.1, num(p, 'thickness', 0.6) * zoom * 1.4));
    if (selected) halo(g, s.lines, map, c, px);
    g.strokeStyle = selected ? c.accent : c.text;
    g.lineWidth = px;
    g.globalAlpha = 0.9;
    g.setLineDash([Math.max(2.2, len * 0.66), Math.max(1.6, len * 0.34)]);
  } else if (s.mode === 'puckering') {
    if (selected) halo(g, s.lines, map, c, 1.4);
    g.strokeStyle = selected ? c.accent : c.muted;
    g.lineWidth = 1.2;
    g.globalAlpha = 0.95;
    g.setLineDash([]);
  } else {
    if (selected) halo(g, s.lines, map, c, 1.4);
    g.strokeStyle = selected ? c.accent : c.warn;
    g.lineWidth = Math.max(1.4, num(p, 'fray', 4) * zoom * 0.5);
    g.globalAlpha = 0.7;
    g.setLineDash([1, 2.4]);
  }
  g.beginPath();
  for (const l of s.lines) trace(g, l, map);
  g.stroke();
  g.restore();
}

function screenPts(pts: Vec[], map: Map2): Vec[] {
  return pts.map((p) => map(p[0], p[1]));
}

function teeth(g: CanvasRenderingContext2D, pts: Vec[], to: number, tooth: number, step: number, phase: number): void {
  const w = walkOf(pts);
  const end = Math.min(w.total, to);
  for (let s = phase * step, k = 0; s <= end; s += step, k++) {
    const { p, d } = atLength(w, s);
    const len = tooth * (k % 2 === 0 ? 1 : 0.7);
    g.moveTo(p[0] - d[1] * len, p[1] + d[0] * len);
    g.lineTo(p[0] + d[1] * len, p[1] - d[0] * len);
  }
}

function drawZipper(g: CanvasRenderingContext2D, z: ZipGeo, map: Map2, zoom: number, c: Colors, picked: TrimItem | null): void {
  const sides = [screenPts(z.a, map), screenPts(z.b, map)];
  const selected = on(picked, 'zipper', z.zipper.id);
  const tooth = Math.min(6, Math.max(2.2, z.width * zoom * 0.28));
  const step = Math.max(3, tooth * 0.9);
  g.save();
  g.lineCap = 'round';
  g.lineJoin = 'round';
  for (let i = 0; i < 2; i++) {
    const pts = sides[i];
    if (pts.length < 2) continue;
    const w = walkOf(pts);
    const closedAt = w.total * Math.min(1, Math.max(0, z.zipper.closed));
    if (selected) {
      g.globalAlpha = 0.3;
      g.strokeStyle = c.accent;
      g.lineWidth = 9;
      g.setLineDash([]);
      g.beginPath();
      trace(g, pts, (x, y) => [x, y]);
      g.stroke();
    }
    g.globalAlpha = 0.9;
    g.strokeStyle = selected ? c.accent : ZIPPER_HUE;
    g.lineWidth = 3;
    g.setLineDash([]);
    g.beginPath();
    trace(g, pts, (x, y) => [x, y]);
    g.stroke();
    g.strokeStyle = c.bg;
    g.lineWidth = 1;
    g.setLineDash(closedAt < w.total ? [3, 3] : []);
    g.beginPath();
    trace(g, pts, (x, y) => [x, y]);
    g.stroke();
    g.setLineDash([]);
    g.strokeStyle = selected ? c.accent : c.text;
    g.lineWidth = 1.3;
    g.beginPath();
    teeth(g, pts, closedAt, tooth, step, i * 0.5);
    g.stroke();
    if (i === 0 && w.total > 0) {
      const { p, d } = atLength(w, closedAt);
      g.fillStyle = c.bg;
      g.strokeStyle = selected ? c.accent : c.text;
      g.lineWidth = 1.5;
      g.beginPath();
      g.arc(p[0], p[1], 4.2, 0, Math.PI * 2);
      g.fill();
      g.stroke();
      g.beginPath();
      g.moveTo(p[0], p[1]);
      g.lineTo(p[0] + d[0] * 9, p[1] + d[1] * 9);
      g.stroke();
    }
  }
  if (sides[0].length > 1 && sides[1].length > 1) {
    g.globalAlpha = 0.55;
    g.strokeStyle = ZIPPER_HUE;
    g.lineWidth = 1;
    g.setLineDash([3, 4]);
    g.beginPath();
    for (const [a, b] of [[sides[0][0], sides[1][0]], [sides[0][sides[0].length - 1], sides[1][sides[1].length - 1]]]) {
      g.moveTo(a[0], a[1]);
      g.lineTo(b[0], b[1]);
    }
    g.stroke();
  }
  g.restore();
}

function slot(g: CanvasRenderingContext2D, len: number, height: number): void {
  const r = height / 2;
  g.beginPath();
  g.moveTo(-len / 2 + r, -r);
  g.lineTo(len / 2 - r, -r);
  g.arc(len / 2 - r, 0, r, -Math.PI / 2, Math.PI / 2);
  g.lineTo(-len / 2 + r, r);
  g.arc(-len / 2 + r, 0, r, Math.PI / 2, (Math.PI * 3) / 2);
  g.closePath();
}

export function drawHardware(g: CanvasRenderingContext2D, h: HwGeo, map: Map2, zoom: number, c: Colors, picked: TrimItem | null): void {
  const s = map(h.at[0], h.at[1]);
  const selected = on(picked, 'hardware', h.hw.id);
  const kind = h.style?.kind ?? 'button';
  g.save();
  g.translate(s[0], s[1]);
  g.rotate(-h.angle);
  g.lineJoin = 'round';
  if (kind === 'buttonhole') {
    const len = Math.max(9, h.length * zoom);
    const hh = Math.max(4, 2.6 * zoom);
    if (selected) {
      slot(g, len + 7, hh + 7);
      g.globalAlpha = 0.3;
      g.fillStyle = c.accent;
      g.fill();
      g.globalAlpha = 1;
    }
    slot(g, len, hh);
    g.fillStyle = c.bg;
    g.globalAlpha = 0.8;
    g.fill();
    g.globalAlpha = 1;
    g.strokeStyle = selected ? c.accent : c.text;
    g.lineWidth = 1.5;
    g.stroke();
    g.setLineDash([1.6, 1.8]);
    g.lineWidth = 1;
    g.beginPath();
    g.moveTo(-len / 2 + hh / 2, 0);
    g.lineTo(len / 2 - hh / 2, 0);
    g.stroke();
  } else if (kind === 'trim') {
    const r = Math.max(5, h.width * zoom * 0.5);
    g.beginPath();
    for (let i = 0; i < 6; i++) g.lineTo(Math.cos((i * Math.PI) / 3) * r, Math.sin((i * Math.PI) / 3) * r);
    g.closePath();
    g.fillStyle = selected ? c.accent : c.bg;
    g.globalAlpha = selected ? 0.4 : 0.8;
    g.fill();
    g.globalAlpha = 1;
    g.strokeStyle = selected ? c.accent : c.text;
    g.lineWidth = 1.5;
    g.stroke();
  } else {
    const r = Math.max(3.5, (h.width * zoom) / 2);
    if (selected) {
      g.beginPath();
      g.arc(0, 0, r + 4, 0, Math.PI * 2);
      g.globalAlpha = 0.3;
      g.fillStyle = c.accent;
      g.fill();
      g.globalAlpha = 1;
    }
    g.beginPath();
    g.arc(0, 0, r, 0, Math.PI * 2);
    g.fillStyle = c.bg;
    g.globalAlpha = 0.85;
    g.fill();
    g.globalAlpha = 1;
    g.strokeStyle = selected ? c.accent : c.text;
    g.lineWidth = 1.5;
    g.stroke();
    if (r > 5) {
      g.beginPath();
      g.arc(0, 0, r * 0.66, 0, Math.PI * 2);
      g.lineWidth = 0.9;
      g.globalAlpha = 0.6;
      g.stroke();
      g.globalAlpha = 1;
    }
    const thread = str(h.style?.params, 'thread', 'cross');
    const holes: Vec[] = thread === 'shank' ? [] : thread === 'parallel' ? [[-0.28, -0.28], [0.28, -0.28], [-0.28, 0.28], [0.28, 0.28]] : [[-0.28, -0.28], [0.28, -0.28], [-0.28, 0.28], [0.28, 0.28]];
    g.fillStyle = selected ? c.accent : c.text;
    g.beginPath();
    for (const [x, y] of holes) { g.moveTo(x * r * 2 + 0.9, y * r * 2); g.arc(x * r * 2, y * r * 2, 0.9, 0, Math.PI * 2); }
    g.fill();
    if (thread === 'cross' && r > 5) {
      g.lineWidth = 0.8;
      g.strokeStyle = selected ? c.accent : c.text;
      g.beginPath();
      g.moveTo(-0.28 * r * 2, -0.28 * r * 2);
      g.lineTo(0.28 * r * 2, 0.28 * r * 2);
      g.moveTo(0.28 * r * 2, -0.28 * r * 2);
      g.lineTo(-0.28 * r * 2, 0.28 * r * 2);
      g.stroke();
    }
  }
  g.restore();
}

export function drawTrims(g: CanvasRenderingContext2D, scene: TrimScene, map: Map2, zoom: number, c: Colors, picked: TrimItem | null): void {
  if (scene.hardware.length + scene.zippers.length + scene.stitches.length + scene.finishes.length === 0) return;
  for (const f of scene.finishes) drawFinish(g, f, map, zoom, c, picked);
  for (const s of scene.stitches) drawStitch(g, s, map, zoom, c, picked);
  for (const z of scene.zippers) drawZipper(g, z, map, zoom, c, picked);
  g.save();
  g.setLineDash([5, 4]);
  g.strokeStyle = c.accent;
  g.lineWidth = 1.2;
  g.globalAlpha = 0.9;
  g.beginPath();
  for (const h of scene.hardware) {
    const other = h.hw.fastened_to;
    if (other === null || other < h.hw.id) continue;
    const o = scene.byHardware.get(other);
    if (!o) continue;
    const a = map(h.at[0], h.at[1]);
    const b = map(o.at[0], o.at[1]);
    g.moveTo(a[0], a[1]);
    g.lineTo(b[0], b[1]);
  }
  g.stroke();
  g.restore();
  for (const h of scene.hardware) drawHardware(g, h, map, zoom, c, picked);
}
