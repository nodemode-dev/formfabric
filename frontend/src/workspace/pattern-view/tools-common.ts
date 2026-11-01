import { bezAt, dist, hitTest, len, pointInPoly, sub, type Hit, type PanelGeo } from '../geom';
import type { Vec } from '../types';
import type { Colors, Ptr, Tool, ViewCtx } from './ctx';
import { worldLabel } from './render';

export const DRAG_PX = 3;

export function constrain(from: Vec, to: Vec, p: { shift: boolean; mod: boolean }): Vec {
  if (!p.shift && !p.mod) return to;
  const dx = to[0] - from[0];
  const dy = to[1] - from[1];
  if (p.mod && !p.shift) return Math.abs(dx) >= Math.abs(dy) ? [to[0], from[1]] : [from[0], to[1]];
  const step = Math.PI / 4;
  const ang = Math.round(Math.atan2(dy, dx) / step) * step;
  const l = Math.hypot(dx, dy);
  return [from[0] + Math.cos(ang) * l, from[1] + Math.sin(ang) * l];
}

export function moved(a: Ptr, b: Ptr): boolean {
  return Math.hypot(a.sx - b.sx, a.sy - b.sy) > DRAG_PX;
}

export abstract class BaseTool implements Tool {
  abstract id: string;
  abstract label: string;
  abstract key: string;
  abstract icon: string;
  constructor(protected readonly ctx: ViewCtx) {}
  down(_p: Ptr): void { return; }
  move(_p: Ptr): void { return; }
  up(_p: Ptr): void { return; }
  exact(): void { return; }
  cancel(): void { return; }
  busy(): boolean { return false; }
  draw(_g: CanvasRenderingContext2D, _c: Colors): void { return; }

  protected async create(command: string, args: Record<string, unknown>, select = true): Promise<void> {
    const r = await this.ctx.doc.run<{ id?: number }>(command, args);
    if (select && r && typeof r.id === 'number') {
      this.ctx.sel.only(r.id);
      this.ctx.selected();
      this.ctx.redraw();
    }
  }

  protected async asExact(command: string, initial: Record<string, unknown>): Promise<void> {
    const v = await this.ctx.exact(command, initial);
    if (v) await this.create(command, v);
  }

  protected async ask(command: string, initial: Record<string, unknown>): Promise<Record<string, unknown> | null> {
    const v = await this.ctx.exact(command, initial);
    if (!v) return null;
    const before = new Set(this.ctx.doc.garment.panels.map((p) => p.id));
    await this.ctx.doc.run(command, v);
    firstAdded(this.ctx, before);
    return v;
  }

  protected firstPanel(): number | null {
    const id = this.ctx.sel.list().find((x) => this.ctx.doc.panel(x));
    return id ?? null;
  }
}

export function markHover(ctx: ViewCtx, hit: Hit | null): void {
  ctx.setHover(hit);
}

export function nearestVertex(points: Vec[], w: Vec): Vec | null {
  let best: Vec | null = null;
  let bd = Infinity;
  for (const p of points) {
    const d = dist(p, w);
    if (d < bd) { bd = d; best = p; }
  }
  return best;
}

export const normalAt = (geo: PanelGeo, seg: number, t: number): Vec => {
  const s = geo.outline[seg];
  const a = bezAt(s, Math.max(0, t - 0.01));
  const b = bezAt(s, Math.min(1, t + 0.01));
  const d = sub(b, a);
  const l = len(d) || 1;
  const n: Vec = [d[1] / l, -d[0] / l];
  const at = bezAt(s, t);
  return pointInPoly([at[0] + n[0] * 2, at[1] + n[1] * 2], geo.poly) ? [-n[0], -n[1]] : n;
};

export function segHit(ctx: ViewCtx, p: Ptr, px = 8): Hit | null {
  return hitTest(ctx.doc.geos(), p.w, ctx.tol(px), { handles: false, points: false, segments: true, panels: false });
}

export function panelOrSegHit(ctx: ViewCtx, p: Ptr, px = 8): Hit | null {
  return hitTest(ctx.doc.geos(), p.w, ctx.tol(px), { handles: false, points: false, segments: true, panels: true });
}

export function strokeSeg(ctx: ViewCtx, g: CanvasRenderingContext2D, c: Colors, panel: number, index: number, width = 3, alpha = 0.85): void {
  const geo = ctx.doc.geos().find((x) => x.panel.id === panel);
  const seg = geo?.outline[index];
  if (!seg) return;
  g.save();
  g.beginPath();
  const a = ctx.toScreen(seg.a);
  g.moveTo(a[0], a[1]);
  const b = ctx.toScreen(seg.b);
  if (seg.curved) {
    const c1 = ctx.toScreen(seg.c1);
    const c2 = ctx.toScreen(seg.c2);
    g.bezierCurveTo(c1[0], c1[1], c2[0], c2[1], b[0], b[1]);
  } else g.lineTo(b[0], b[1]);
  g.lineWidth = width;
  g.globalAlpha = alpha;
  g.strokeStyle = c.accent;
  g.stroke();
  g.restore();
}

export function lineLabel(ctx: ViewCtx, g: CanvasRenderingContext2D, c: Colors, at: Vec, text: string): void {
  const s = ctx.toScreen(at);
  worldLabel(g, text, s[0] + 12, s[1] - 10, c);
}

export function firstAdded(ctx: ViewCtx, before: Set<number>): void {
  const fresh = ctx.doc.garment.panels.find((p) => !before.has(p.id));
  if (!fresh) return;
  ctx.sel.only(fresh.id);
  ctx.selected();
  ctx.redraw();
}
