import type { Vec } from '../types';
import type { Colors } from './ctx';
import { pointAt, type Chain, type SeamGeo, type SewScene } from './sew-geom';

type Map2 = (x: number, y: number) => [number, number];

const STUB = 22;
const FAR = 110;
const GAP = 40;

function trace(g: CanvasRenderingContext2D, c: Chain, map: Map2): void {
  c.pts.forEach((p, i) => {
    const s = map(p[0], p[1]);
    if (i === 0) g.moveTo(s[0], s[1]);
    else g.lineTo(s[0], s[1]);
  });
}

function tag(g: CanvasRenderingContext2D, text: string, x: number, y: number, c: Colors, color: string): void {
  g.font = '11px Inter, sans-serif';
  const wid = g.measureText(text).width + 10;
  g.fillStyle = c.bg;
  g.globalAlpha = 0.92;
  g.fillRect(x, y - 13, wid, 18);
  g.globalAlpha = 1;
  g.strokeStyle = color;
  g.lineWidth = 1;
  g.strokeRect(x + 0.5, y - 12.5, wid - 1, 17);
  g.fillStyle = c.text;
  g.textAlign = 'start';
  g.fillText(text, x + 5, y);
}

function connectors(g: CanvasRenderingContext2D, sg: SeamGeo, map: Map2, zoom: number): void {
  if (sg.a.pts.length < 2 || sg.b.pts.length < 2) return;
  const n = Math.max(1, Math.min(40, Math.round((Math.min(sg.a.total, sg.b.total) * zoom) / GAP)));
  g.beginPath();
  for (let k = 0; k <= n; k++) {
    const fa = k / n;
    const fb = sg.seam.reversed ? 1 - fa : fa;
    const pa = map(...pointAt(sg.a, fa));
    const pb = map(...pointAt(sg.b, fb));
    const dx = pb[0] - pa[0];
    const dy = pb[1] - pa[1];
    const l = Math.hypot(dx, dy);
    if (l < 1) continue;
    if (l <= FAR) {
      g.moveTo(pa[0], pa[1]);
      g.lineTo(pb[0], pb[1]);
    } else {
      const ux = dx / l;
      const uy = dy / l;
      g.moveTo(pa[0], pa[1]);
      g.lineTo(pa[0] + ux * STUB, pa[1] + uy * STUB);
      g.moveTo(pb[0], pb[1]);
      g.lineTo(pb[0] - ux * STUB, pb[1] - uy * STUB);
    }
  }
  g.stroke();
}

function ends(g: CanvasRenderingContext2D, ch: Chain, map: Map2, color: string, on: boolean, c: Colors): void {
  if (ch.pts.length < 2) return;
  const r = on ? 5 : 4;
  const s = map(ch.pts[0][0], ch.pts[0][1]);
  const e = map(ch.pts[ch.pts.length - 1][0], ch.pts[ch.pts.length - 1][1]);
  g.lineWidth = 1.5;
  g.strokeStyle = c.bg;
  g.fillStyle = color;
  g.beginPath();
  g.arc(s[0], s[1], r, 0, Math.PI * 2);
  g.fill();
  g.stroke();
  g.beginPath();
  g.rect(e[0] - r + 0.5, e[1] - r + 0.5, r * 2 - 1, r * 2 - 1);
  g.fill();
  g.stroke();
}

export function mmText(v: number): string {
  return `${v > 0 ? '+' : ''}${v.toFixed(1)} mm`;
}

export function drawSew(g: CanvasRenderingContext2D, scene: SewScene, map: Map2, zoom: number, seam: number | null, tack: number | null, c: Colors): void {
  g.save();
  g.lineJoin = 'round';
  g.lineCap = 'round';
  for (const sg of scene.seams) {
    const on = sg.seam.id === seam;
    const bad = sg.mismatch !== null;
    const color = bad ? c.warn : sg.color;
    if (on) {
      g.globalAlpha = 0.28;
      g.strokeStyle = color;
      g.lineWidth = 9;
      for (const ch of [sg.a, sg.b]) {
        g.beginPath();
        trace(g, ch, map);
        g.stroke();
      }
    }
    g.globalAlpha = on ? 0.8 : 0.5;
    g.strokeStyle = color;
    g.lineWidth = 1;
    g.setLineDash([]);
    connectors(g, sg, map, zoom);
    g.globalAlpha = 0.95;
    g.lineWidth = on ? 4 : 3;
    for (const ch of [sg.a, sg.b]) {
      g.beginPath();
      trace(g, ch, map);
      g.stroke();
    }
    g.globalAlpha = 1;
    ends(g, sg.a, map, color, on, c);
    ends(g, sg.b, map, color, on, c);
    if (bad && sg.mismatch !== null && sg.b.pts.length > 1) {
      const m = map(...pointAt(sg.b, 0.5));
      tag(g, mmText(sg.mismatch), m[0] + 10, m[1] - 8, c, color);
    }
  }
  g.lineWidth = 1.6;
  for (const tg of scene.tacks) {
    if (!tg.a && !tg.b) continue;
    const on = tg.tack.id === tack;
    g.strokeStyle = on ? c.accent : c.text;
    g.fillStyle = on ? c.accent : c.text;
    g.globalAlpha = on ? 1 : 0.85;
    const pts: Vec[] = [];
    if (tg.a) pts.push(tg.a);
    if (tg.b) pts.push(tg.b);
    if (pts.length === 2) {
      const a = map(pts[0][0], pts[0][1]);
      const b = map(pts[1][0], pts[1][1]);
      g.setLineDash([6, 4]);
      g.beginPath();
      g.moveTo(a[0], a[1]);
      g.lineTo(b[0], b[1]);
      g.stroke();
      g.setLineDash([]);
    }
    for (const p of pts) {
      const s = map(p[0], p[1]);
      g.beginPath();
      g.arc(s[0], s[1], on ? 4.5 : 3.5, 0, Math.PI * 2);
      g.fill();
    }
  }
  g.restore();
}
