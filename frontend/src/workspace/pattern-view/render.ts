import { pointInPoly, type Hit, type NotchGeo, type PanelGeo, type Seg } from '../geom';
import type { Selection } from '../state';
import { toUnit } from '../state';
import type { Cam, Colors, Tool } from './ctx';
import { drawSew } from './render-sew';
import type { SewScene } from './sew-geom';
import { drawPrints, drawTrims } from '../trims/draw';
import type { TrimScene } from '../trims/scene';
import { trimState } from '../trims/state';

export interface Scene {
  w: number;
  h: number;
  cam: Cam;
  geos: PanelGeo[];
  sel: Selection;
  hover: Hit | null;
  showPoints: boolean;
  rulers: boolean;
  unit: string;
  colors: Colors;
  sew: SewScene;
  trims: TrimScene;
}

const NICE = [1, 2, 5];

export function gridStep(s: number): { minor: number; major: number } {
  for (let mag = 0.01; mag < 1e7; mag *= 10) {
    for (const m of NICE) {
      const step = m * mag;
      if (step * s >= 10) return { minor: step, major: step * (m === 1 ? 10 : m === 2 ? 5 : 2) };
    }
  }
  return { minor: 1e7, major: 1e8 };
}

export function pathSegs(g: CanvasRenderingContext2D, segs: Seg[], map: (x: number, y: number) => [number, number], close: boolean): void {
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
  if (close) g.closePath();
}

function grid(g: CanvasRenderingContext2D, sc: Scene): void {
  const { cam, w, h, colors } = sc;
  const { minor, major } = gridStep(cam.s);
  const ox = w / 2 - cam.x * cam.s;
  const oy = h / 2 + cam.y * cam.s;
  const x0 = (0 - ox) / cam.s;
  const x1 = (w - ox) / cam.s;
  const y1 = (oy - 0) / cam.s;
  const y0 = (oy - h) / cam.s;
  const lines = (step: number, alpha: number): void => {
    g.globalAlpha = alpha;
    g.beginPath();
    for (let x = Math.ceil(x0 / step) * step; x <= x1; x += step) {
      const px = Math.round(ox + x * cam.s) + 0.5;
      g.moveTo(px, 0);
      g.lineTo(px, h);
    }
    for (let y = Math.ceil(y0 / step) * step; y <= y1; y += step) {
      const py = Math.round(oy - y * cam.s) + 0.5;
      g.moveTo(0, py);
      g.lineTo(w, py);
    }
    g.stroke();
  };
  g.lineWidth = 1;
  g.strokeStyle = colors.grid;
  lines(minor, 0.35);
  lines(major, 0.8);
  g.globalAlpha = 1;
  g.strokeStyle = colors.muted;
  g.globalAlpha = 0.6;
  g.beginPath();
  const ax = Math.round(ox) + 0.5;
  const ay = Math.round(oy) + 0.5;
  g.moveTo(ax, 0);
  g.lineTo(ax, h);
  g.moveTo(0, ay);
  g.lineTo(w, ay);
  g.stroke();
  g.globalAlpha = 1;
}

function rulers(g: CanvasRenderingContext2D, sc: Scene): void {
  const { cam, w, h, colors, unit } = sc;
  const T = 18;
  const { minor, major } = gridStep(cam.s);
  const ox = w / 2 - cam.x * cam.s;
  const oy = h / 2 + cam.y * cam.s;
  g.save();
  g.fillStyle = colors.bg;
  g.globalAlpha = 0.92;
  g.fillRect(0, 0, w, T);
  g.fillRect(0, 0, T, h);
  g.globalAlpha = 1;
  g.strokeStyle = colors.grid;
  g.fillStyle = colors.muted;
  g.font = '10px Inter, sans-serif';
  g.textBaseline = 'top';
  g.beginPath();
  g.moveTo(0, T + 0.5);
  g.lineTo(w, T + 0.5);
  g.moveTo(T + 0.5, 0);
  g.lineTo(T + 0.5, h);
  const dec = Math.max(unit === 'mm' ? 0 : unit === 'cm' ? 1 : 2, Math.ceil(-Math.log10(toUnit(major, unit)) - 1e-9));
  for (let x = Math.ceil((0 - ox) / cam.s / minor) * minor; ox + x * cam.s <= w; x += minor) {
    const px = Math.round(ox + x * cam.s) + 0.5;
    if (px < T) continue;
    const big = Math.abs(x / major - Math.round(x / major)) < 1e-6;
    g.moveTo(px, T);
    g.lineTo(px, big ? 4 : 12);
    if (big) g.fillText(toUnit(x, unit).toFixed(dec), px + 3, 2);
  }
  g.textBaseline = 'alphabetic';
  for (let y = Math.ceil((oy - h) / cam.s / minor) * minor; oy - y * cam.s >= 0; y += minor) {
    const py = Math.round(oy - y * cam.s) + 0.5;
    if (py < T) continue;
    const big = Math.abs(y / major - Math.round(y / major)) < 1e-6;
    g.moveTo(T, py);
    g.lineTo(big ? 4 : 12, py);
    if (big) {
      g.save();
      g.translate(11, py - 3);
      g.rotate(-Math.PI / 2);
      g.textBaseline = 'top';
      g.fillText(toUnit(y, unit).toFixed(dec), 0, -9);
      g.restore();
    }
  }
  g.stroke();
  g.restore();
}

export function notch(g: CanvasRenderingContext2D, n: NotchGeo, map: (x: number, y: number) => [number, number], s: number, color: string, alpha: number): void {
  const p = map(n.pt[0], n.pt[1]);
  const l = Math.hypot(n.n[0], n.n[1]) || 1;
  const nx = n.n[0] / l;
  const ny = -n.n[1] / l;
  const px = -ny;
  const py = nx;
  const d = Math.max(5, n.depth * s);
  const w = Math.max(4, n.width * s);
  g.save();
  g.globalAlpha = alpha;
  g.strokeStyle = color;
  g.lineWidth = 1.4;
  g.lineCap = 'round';
  g.beginPath();
  if (n.kind === 'slit') {
    g.moveTo(p[0], p[1]);
    g.lineTo(p[0] + nx * d, p[1] + ny * d);
  } else if (n.kind === 'v') {
    g.moveTo(p[0] - px * w / 2, p[1] - py * w / 2);
    g.lineTo(p[0] + nx * d, p[1] + ny * d);
    g.lineTo(p[0] + px * w / 2, p[1] + py * w / 2);
  } else if (n.kind === 't') {
    g.moveTo(p[0], p[1]);
    g.lineTo(p[0] + nx * d, p[1] + ny * d);
    g.moveTo(p[0] + nx * d - px * w / 2, p[1] + ny * d - py * w / 2);
    g.lineTo(p[0] + nx * d + px * w / 2, p[1] + ny * d + py * w / 2);
  } else {
    const r = w / 2;
    const c = [p[0] + nx * Math.max(d - r, 0), p[1] + ny * Math.max(d - r, 0)];
    g.moveTo(p[0] - px * r, p[1] - py * r);
    g.lineTo(c[0] - px * r, c[1] - py * r);
    const cross = -px * ny - -py * nx;
    g.arc(c[0], c[1], r, Math.atan2(-py, -px), Math.atan2(py, px), cross < 0);
    g.lineTo(p[0] + px * r, p[1] + py * r);
  }
  g.stroke();
  g.restore();
}

function linkBadge(g: CanvasRenderingContext2D, geo: PanelGeo, map: (x: number, y: number) => [number, number], c: Colors): void {
  const tl = map(geo.bounds[0], geo.bounds[3]);
  const x = Math.round(tl[0]) + 6;
  const y = Math.round(tl[1]) + 6;
  const link = geo.panel.link;
  const ord = link && link.kind === 'instance' && Array.isArray(link.axis) && Array.isArray(link.axis[1]) ? Number(link.axis[1][0]) : 0;
  g.save();
  g.fillStyle = c.bg;
  g.globalAlpha = 0.92;
  g.fillRect(x, y, ord > 0 ? 30 : 22, 16);
  g.globalAlpha = 1;
  g.strokeStyle = c.muted;
  g.lineWidth = 1;
  g.strokeRect(x + 0.5, y + 0.5, ord > 0 ? 29 : 21, 15);
  g.strokeStyle = c.text;
  g.lineWidth = 1.3;
  g.beginPath();
  g.roundRect(x + 4, y + 5, 8, 6, 3);
  g.roundRect(x + 9, y + 5, 8, 6, 3);
  g.stroke();
  if (ord > 0) {
    g.font = '10px Inter, sans-serif';
    g.fillStyle = c.text;
    g.textBaseline = 'middle';
    g.fillText(String(ord), x + 20, y + 9);
  } else {
    g.setLineDash([2, 2]);
    g.beginPath();
    g.moveTo(x + 10.5, y + 2);
    g.lineTo(x + 10.5, y + 14);
    g.stroke();
  }
  g.restore();
}

export function drawScene(g: CanvasRenderingContext2D, sc: Scene, tool: Tool): void {
  const { w, h, cam, colors } = sc;
  g.fillStyle = colors.bg;
  g.fillRect(0, 0, w, h);
  grid(g, sc);
  const ox = w / 2 - cam.x * cam.s;
  const oy = h / 2 + cam.y * cam.s;
  const map = (x: number, y: number): [number, number] => [ox + x * cam.s, oy - y * cam.s];
  const geos = [...sc.geos].sort((a, b) => a.panel.layer - b.panel.layer);
  g.lineJoin = 'round';

  for (const geo of geos) {
    const picked = sc.sel.panels.has(geo.panel.id);
    const dim = geo.linked ? 0.6 : 1;
    if (geo.seam) {
      g.beginPath();
      pathSegs(g, geo.seam, map, true);
      g.globalAlpha = (picked ? 1 : 0.85) * dim;
      g.lineWidth = 1;
      g.setLineDash([7, 4]);
      g.strokeStyle = picked ? colors.accent : colors.muted;
      g.stroke();
      g.setLineDash([]);
      g.globalAlpha = 1;
    }
    g.beginPath();
    pathSegs(g, geo.outline, map, true);
    g.globalAlpha = picked ? 0.2 : geo.linked ? 0.05 : 0.09;
    g.fillStyle = picked ? colors.accent : colors.text;
    g.fill();
    const printed = sc.trims.prints.get(geo.panel.id);
    if (printed) {
      g.globalAlpha = dim;
      drawPrints(g, geo, printed, map, cam.s, colors, trimState.item);
      g.beginPath();
      pathSegs(g, geo.outline, map, true);
    }
    g.globalAlpha = dim;
    g.lineWidth = picked ? 2 : 1.5;
    g.strokeStyle = picked ? colors.accent : colors.line;
    g.stroke();
    g.globalAlpha = 1;

    for (const inner of geo.inner) {
      const chosen = sc.sel.inner !== null && sc.sel.inner.panel === geo.panel.id && sc.sel.inner.id === inner.id;
      g.beginPath();
      pathSegs(g, inner.segs, map, inner.closed);
      g.globalAlpha = dim;
      const tone = chosen || picked ? colors.accent : colors.muted;
      if (inner.role === 'fold') {
        g.lineWidth = 1.8;
        g.setLineDash([12, 3, 2, 3]);
        g.strokeStyle = chosen || picked ? colors.accent : colors.text;
        g.stroke();
      } else if (inner.role === 'dart') {
        g.globalAlpha = 0.14 * dim;
        g.fillStyle = tone;
        if (inner.closed) g.fill();
        g.globalAlpha = dim;
        g.lineWidth = 1.4;
        g.setLineDash([]);
        g.strokeStyle = chosen || picked ? colors.accent : colors.line;
        g.stroke();
      } else if (inner.role === 'reference') {
        g.lineWidth = 1;
        g.setLineDash([1.5, 4]);
        g.strokeStyle = tone;
        g.stroke();
      } else {
        g.lineWidth = chosen ? 2 : 1;
        g.setLineDash([5, 4]);
        g.strokeStyle = tone;
        g.stroke();
      }
      g.setLineDash([]);
      g.globalAlpha = 1;
    }

    for (const n of geo.notches) notch(g, n, map, cam.s, picked ? colors.accent : colors.text, dim);

    const [cx, cy] = map(geo.center[0], geo.center[1]);
    const small = Math.min(geo.bounds[2] - geo.bounds[0], geo.bounds[3] - geo.bounds[1]) * cam.s;
    let labelY = cy + 16;
    if (geo.grain) {
      const a = map(geo.grain.a[0], geo.grain.a[1]);
      const b = map(geo.grain.b[0], geo.grain.b[1]);
      const px = Math.hypot(b[0] - a[0], b[1] - a[1]);
      if (px > 14) {
        g.globalAlpha = (geo.grain.set ? 1 : 0.38) * dim;
        g.strokeStyle = geo.grain.set ? colors.text : colors.muted;
        g.lineWidth = geo.grain.set ? 1.4 : 1;
        g.beginPath();
        g.moveTo(a[0], a[1]);
        g.lineTo(b[0], b[1]);
        for (const [e, o] of [[b, a], [a, b]]) {
          const ang = Math.atan2(e[1] - o[1], e[0] - o[0]);
          g.moveTo(e[0], e[1]);
          g.lineTo(e[0] - Math.cos(ang - 0.45) * 8, e[1] - Math.sin(ang - 0.45) * 8);
          g.moveTo(e[0], e[1]);
          g.lineTo(e[0] - Math.cos(ang + 0.45) * 8, e[1] - Math.sin(ang + 0.45) * 8);
        }
        g.stroke();
        g.globalAlpha = 1;
        labelY = Math.max(a[1], b[1]) + 16;
      }
    }
    if (small > 36) {
      g.font = '11px Inter, sans-serif';
      g.textAlign = 'center';
      g.globalAlpha = dim;
      g.fillStyle = picked ? colors.accent : colors.muted;
      g.fillText(geo.panel.name, cx, Math.max(labelY, cy + 16));
      g.textAlign = 'start';
      g.globalAlpha = 1;
      if (geo.linked) linkBadge(g, geo, map, colors);
    }
  }

  drawSew(g, sc.sew, map, cam.s, sc.sel.seam, sc.sel.tack, colors);
  drawTrims(g, sc.trims, map, cam.s, colors, trimState.item);

  const hov = sc.hover;
  if (hov) {
    const geo = geos.find((x) => x.panel.id === hov.panel);
    if (geo && (hov.kind === 'segment' || hov.kind === 'panel')) {
      g.beginPath();
      if (hov.kind === 'segment') pathSegs(g, geo.outline[hov.index] ? [geo.outline[hov.index]] : [], map, false);
      else pathSegs(g, geo.outline, map, true);
      g.lineWidth = hov.kind === 'segment' ? 3 : 2;
      g.globalAlpha = 0.7;
      g.strokeStyle = colors.accent;
      g.stroke();
      g.globalAlpha = 1;
    }
  }

  if (sc.showPoints) {
    for (const geo of geos) {
      const chosen = sc.sel.pts && sc.sel.pts.panel === geo.panel.id ? new Set(sc.sel.pts.idx) : null;
      g.lineWidth = 1;
      for (let i = 0; i < geo.pts.length; i++) {
        const [px, py] = map(geo.pts[i][0], geo.pts[i][1]);
        for (const hp of [geo.hin[i], geo.hout[i]]) {
          if (!hp) continue;
          const [hx, hy] = map(hp[0], hp[1]);
          g.beginPath();
          g.moveTo(px, py);
          g.lineTo(hx, hy);
          g.strokeStyle = colors.muted;
          g.stroke();
          g.beginPath();
          g.arc(hx, hy, 3.5, 0, Math.PI * 2);
          g.fillStyle = colors.bg;
          g.fill();
          g.strokeStyle = colors.accent;
          g.stroke();
        }
        const on = chosen?.has(i) === true;
        const curve = geo.panel.outline[i].kind !== 'corner';
        g.beginPath();
        if (curve) g.arc(px, py, on ? 5 : 4, 0, Math.PI * 2);
        else g.rect(px - (on ? 4.5 : 3.5), py - (on ? 4.5 : 3.5), on ? 9 : 7, on ? 9 : 7);
        g.fillStyle = on ? colors.accent : colors.bg;
        g.fill();
        g.strokeStyle = on ? colors.accent : colors.text;
        g.stroke();
      }
    }
  }

  tool.draw(g, colors);
  if (sc.rulers) rulers(g, sc);
}

export function panelAt(geos: PanelGeo[], w: [number, number]): number | null {
  const sorted = [...geos].sort((a, b) => b.panel.layer - a.panel.layer);
  for (const geo of sorted) if (pointInPoly(w, geo.poly)) return geo.panel.id;
  return null;
}

export function worldLabel(g: CanvasRenderingContext2D, text: string, x: number, y: number, c: Colors): void {
  g.font = '11px Inter, sans-serif';
  const wid = g.measureText(text).width + 10;
  g.fillStyle = c.bg;
  g.globalAlpha = 0.88;
  g.fillRect(x, y - 13, wid, 18);
  g.globalAlpha = 1;
  g.strokeStyle = c.grid;
  g.lineWidth = 1;
  g.strokeRect(x + 0.5, y - 12.5, wid - 1, 17);
  g.fillStyle = c.text;
  g.textAlign = 'start';
  g.fillText(text, x + 5, y);
}
