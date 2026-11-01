import { pointInPoly, type Hit, type PanelGeo, type Seg } from '../geom';
import type { Selection } from '../state';
import { toUnit } from '../state';
import type { Cam, Colors, Tool } from './ctx';

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
  const dec = unit === 'mm' ? 0 : unit === 'cm' ? 1 : 2;
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
    g.beginPath();
    pathSegs(g, geo.outline, map, true);
    g.globalAlpha = picked ? 0.2 : 0.09;
    g.fillStyle = picked ? colors.accent : colors.text;
    g.fill();
    g.globalAlpha = 1;
    g.lineWidth = picked ? 2 : 1.5;
    g.strokeStyle = picked ? colors.accent : colors.line;
    g.stroke();

    for (const inner of geo.inner) {
      g.beginPath();
      pathSegs(g, inner.segs, map, inner.closed);
      g.lineWidth = 1;
      g.setLineDash([5, 4]);
      g.strokeStyle = picked ? colors.accent : colors.muted;
      g.stroke();
      g.setLineDash([]);
    }

    const [cx, cy] = map(geo.center[0], geo.center[1]);
    const gr = (geo.panel.rotation + (geo.panel.mirror ? 180 - geo.panel.grain : geo.panel.grain)) * (Math.PI / 180);
    const half = Math.min(geo.bounds[2] - geo.bounds[0], geo.bounds[3] - geo.bounds[1]) * 0.22 * cam.s;
    if (half > 8) {
      const dx = Math.cos(gr) * half;
      const dy = -Math.sin(gr) * half;
      g.beginPath();
      g.moveTo(cx - dx, cy - dy);
      g.lineTo(cx + dx, cy + dy);
      g.lineWidth = 1;
      g.strokeStyle = colors.muted;
      g.stroke();
      for (const k of [-1, 1]) {
        const ex = cx + dx * k;
        const ey = cy + dy * k;
        const ang = Math.atan2(dy * k, dx * k);
        g.beginPath();
        g.moveTo(ex, ey);
        g.lineTo(ex - Math.cos(ang - 0.45) * 7, ey - Math.sin(ang - 0.45) * 7);
        g.moveTo(ex, ey);
        g.lineTo(ex - Math.cos(ang + 0.45) * 7, ey - Math.sin(ang + 0.45) * 7);
        g.stroke();
      }
    }
    if (Math.min(geo.bounds[2] - geo.bounds[0], geo.bounds[3] - geo.bounds[1]) * cam.s > 36) {
      g.font = '11px Inter, sans-serif';
      g.textAlign = 'center';
      g.fillStyle = picked ? colors.accent : colors.muted;
      g.fillText(geo.panel.name, cx, cy + Math.max(half, 0) + 16);
      g.textAlign = 'start';
    }
  }

  const hov = sc.hover;
  if (hov) {
    const geo = geos.find((x) => x.panel.id === hov.panel);
    if (geo && (hov.kind === 'segment' || hov.kind === 'panel')) {
      g.beginPath();
      if (hov.kind === 'segment') pathSegs(g, [geo.outline[hov.index]], map, false);
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
