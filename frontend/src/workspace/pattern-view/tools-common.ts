import { dist, type Hit } from '../geom';
import type { Vec } from '../types';
import type { Colors, Ptr, Tool, ViewCtx } from './ctx';

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
