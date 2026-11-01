import { dist, sub } from '../geom';
import { fmtLen } from '../state';
import type { Vec } from '../types';
import type { Colors, Ptr } from './ctx';
import { panelAt, worldLabel } from './render';
import { BaseTool, constrain, moved } from './tools-common';

function boxOf(a: Vec, b: Vec, p: { shift: boolean; alt: boolean }, kind: 'rect' | 'ellipse'): { x0: number; y0: number; x1: number; y1: number } {
  let dx = b[0] - a[0];
  let dy = b[1] - a[1];
  if (p.shift) {
    const m = Math.max(Math.abs(dx), Math.abs(dy));
    dx = Math.sign(dx || 1) * m;
    dy = Math.sign(dy || 1) * m;
  }
  if (kind === 'ellipse' && p.alt) return { x0: a[0] - Math.abs(dx), y0: a[1] - Math.abs(dy), x1: a[0] + Math.abs(dx), y1: a[1] + Math.abs(dy) };
  return { x0: Math.min(a[0], a[0] + dx), y0: Math.min(a[1], a[1] + dy), x1: Math.max(a[0], a[0] + dx), y1: Math.max(a[1], a[1] + dy) };
}

type Box = { x0: number; y0: number; x1: number; y1: number };

abstract class BoxTool extends BaseTool {
  protected a: Vec | null = null;
  protected b: Vec | null = null;
  protected first: Ptr | null = null;
  protected box: Box | null = null;
  protected panel: number | null = null;
  protected ellipseMode = false;

  protected abstract finish(box: Box, p: Ptr): void;
  protected abstract clicked(at: Vec): void;
  protected begin(_p: Ptr): boolean { return true; }

  down(p: Ptr): void {
    if (!this.begin(p)) return;
    this.first = p;
    this.a = this.ctx.snap(p.w).pt;
    this.b = this.a;
    this.box = null;
  }

  move(p: Ptr): void {
    if (!this.a || !this.first) {
      this.ctx.snap(p.w);
      this.ctx.cursor('crosshair');
      this.ctx.redraw();
      return;
    }
    this.b = p.shift ? p.w : this.ctx.snap(p.w).pt;
    this.box = boxOf(this.a, this.b, p, this.ellipseMode ? 'ellipse' : 'rect');
    this.ctx.redraw();
  }

  up(p: Ptr): void {
    const a = this.a;
    const first = this.first;
    const box = this.box;
    this.a = null;
    this.first = null;
    this.box = null;
    this.ctx.setSnapMark(null);
    if (!a || !first) return;
    if (!moved(first, p) || !box) { this.clicked(a); this.ctx.redraw(); return; }
    if (box.x1 - box.x0 >= 0.5 && box.y1 - box.y0 >= 0.5) this.finish(box, p);
    this.ctx.redraw();
  }

  keydown(e: KeyboardEvent): boolean {
    if (e.key === 'Enter') { this.exact(); return true; }
    if (e.key === 'Escape') { this.cancel(); return true; }
    return false;
  }

  cancel(): void {
    this.a = null;
    this.first = null;
    this.box = null;
    this.ctx.setSnapMark(null);
    this.ctx.redraw();
  }

  busy(): boolean { return this.a !== null; }

  leave(): void { this.cancel(); }

  draw(g: CanvasRenderingContext2D, c: Colors): void {
    const b = this.box;
    if (!b) return;
    const tl = this.ctx.toScreen([b.x0, b.y1]);
    const br = this.ctx.toScreen([b.x1, b.y0]);
    g.save();
    g.strokeStyle = c.accent;
    g.fillStyle = c.accent;
    g.lineWidth = 1.5;
    g.beginPath();
    if (this.ellipseMode) g.ellipse((tl[0] + br[0]) / 2, (tl[1] + br[1]) / 2, (br[0] - tl[0]) / 2, (br[1] - tl[1]) / 2, 0, 0, Math.PI * 2);
    else g.rect(tl[0], tl[1], br[0] - tl[0], br[1] - tl[1]);
    g.globalAlpha = 0.12;
    g.fill();
    g.globalAlpha = 1;
    g.stroke();
    g.restore();
    const unit = this.ctx.unit();
    worldLabel(g, `${fmtLen(b.x1 - b.x0, unit, false)} x ${fmtLen(b.y1 - b.y0, unit)}`, br[0] + 8, br[1] + 18, c);
  }
}

export class RectangleTool extends BoxTool {
  id = 'rectangle';
  label = 'Rectangle';
  key = 'R';
  icon = 'rectangle';

  protected finish(b: Box): void {
    void this.create('pattern.rectangle', { x: b.x0, y: b.y0, width: b.x1 - b.x0, height: b.y1 - b.y0 });
  }

  protected clicked(at: Vec): void {
    void this.asExact('pattern.rectangle', { x: at[0], y: at[1] });
  }

  exact(): void {
    void this.asExact('pattern.rectangle', this.a ? { x: this.a[0], y: this.a[1] } : {});
  }
}

export class EllipseTool extends BoxTool {
  id = 'ellipse';
  label = 'Ellipse';
  key = 'E';
  icon = 'ellipse';
  protected ellipseMode = true;

  protected finish(b: Box): void {
    void this.create('pattern.ellipse', { cx: (b.x0 + b.x1) / 2, cy: (b.y0 + b.y1) / 2, rx: (b.x1 - b.x0) / 2, ry: (b.y1 - b.y0) / 2 });
  }

  protected clicked(at: Vec): void {
    void this.asExact('pattern.ellipse', { cx: at[0], cy: at[1] });
  }

  exact(): void {
    void this.asExact('pattern.ellipse', this.a ? { cx: this.a[0], cy: this.a[1] } : {});
  }
}

export class InnerShapeTool extends BoxTool {
  id = 'inner_shape';
  label = 'Inner Shape';
  key = 'I';
  icon = 'innerShape';

  protected begin(p: Ptr): boolean {
    this.panel = panelAt(this.ctx.doc.geos(), p.w) ?? this.ctx.sel.list()[0] ?? null;
    this.ellipseMode = p.alt;
    if (this.panel === null) { this.ctx.say('Start inside a panel'); return false; }
    return true;
  }

  protected finish(b: Box, p: Ptr): void {
    if (this.panel === null) return;
    const panel = this.panel;
    if (p.alt) void this.create('pattern.inner_shape', { panel, kind: 'ellipse', ellipse: { cx: (b.x0 + b.x1) / 2, cy: (b.y0 + b.y1) / 2, rx: (b.x1 - b.x0) / 2, ry: (b.y1 - b.y0) / 2 } }, false);
    else void this.create('pattern.inner_shape', { panel, kind: 'rectangle', rect: { x: b.x0, y: b.y0, width: b.x1 - b.x0, height: b.y1 - b.y0 } }, false);
  }

  protected clicked(): void {
    this.exact();
  }

  exact(): void {
    const panel = this.panel ?? this.ctx.sel.list()[0];
    if (panel === undefined || panel === null) { this.ctx.say('Select a panel first'); return; }
    void this.ctx.exact('pattern.inner_shape', { panel, kind: 'rectangle' }).then((v) => { if (v) void this.ctx.doc.run('pattern.inner_shape', v); });
  }
}

interface Draft { p: Vec; h: Vec | null }

export class DrawPanelTool extends BaseTool {
  id = 'draw_panel';
  label = 'Draw Panel';
  key = 'P';
  icon = 'pen';
  private pts: Draft[] = [];
  private cur: Vec = [0, 0];
  private dragging = false;
  private downAt: Ptr | null = null;

  private place(p: Ptr): Vec {
    const last = this.pts[this.pts.length - 1];
    if (this.pts.length >= 3 && dist(this.pts[0].p, p.w) <= this.ctx.tol(8)) return this.pts[0].p;
    if (last && (p.shift || p.mod)) return constrain(last.p, p.w, p);
    return this.ctx.snap(p.w).pt;
  }

  down(p: Ptr): void {
    const at = this.place(p);
    if (this.pts.length >= 3 && at === this.pts[0].p) { this.close(); return; }
    const last = this.pts[this.pts.length - 1];
    if (last && dist(last.p, at) < this.ctx.tol(2)) return;
    this.pts.push({ p: at, h: null });
    this.dragging = true;
    this.downAt = p;
    this.cur = at;
    this.ctx.redraw();
  }

  move(p: Ptr): void {
    this.ctx.cursor('crosshair');
    if (this.dragging && this.downAt && this.pts.length) {
      const d = this.pts[this.pts.length - 1];
      if (moved(this.downAt, p)) d.h = sub(p.w, d.p);
      this.cur = d.p;
    } else {
      this.cur = this.pts.length ? this.place(p) : this.ctx.snap(p.w).pt;
    }
    this.ctx.redraw();
  }

  up(_p: Ptr): void {
    this.dragging = false;
    this.downAt = null;
    this.ctx.setSnapMark(null);
  }

  private points(): unknown[] {
    return this.pts.map((d) => (d.h ? { p: d.p, kind: 'smooth', h_in: [-d.h[0], -d.h[1]], h_out: d.h } : d.p));
  }

  private close(): void {
    if (this.pts.length < 3) return;
    const points = this.points();
    this.reset();
    void this.create('pattern.draw_panel', { points });
  }

  private reset(): void {
    this.pts = [];
    this.dragging = false;
    this.downAt = null;
    this.ctx.setSnapMark(null);
    this.ctx.redraw();
  }

  keydown(e: KeyboardEvent): boolean {
    if (e.key === 'Enter') {
      if (this.pts.length >= 3) this.close(); else this.exact();
      return true;
    }
    if (e.key === 'Escape') { this.reset(); return true; }
    if ((e.key === 'Delete' || e.key === 'Backspace') && this.pts.length) {
      this.pts.pop();
      this.ctx.redraw();
      return true;
    }
    return false;
  }

  exact(): void {
    const init = this.pts.length >= 3 ? { points: this.pts.map((d) => d.p) } : {};
    void this.ctx.exact('pattern.draw_panel', init).then((v) => {
      if (!v) return;
      this.reset();
      void this.create('pattern.draw_panel', v);
    });
  }

  cancel(): void { this.reset(); }

  busy(): boolean { return this.pts.length > 0; }

  leave(): void { this.reset(); }

  draw(g: CanvasRenderingContext2D, c: Colors): void {
    if (!this.pts.length) return;
    const unit = this.ctx.unit();
    g.save();
    g.lineWidth = 1.5;
    g.strokeStyle = c.accent;
    g.fillStyle = c.accent;
    g.beginPath();
    const s0 = this.ctx.toScreen(this.pts[0].p);
    g.moveTo(s0[0], s0[1]);
    for (let i = 1; i < this.pts.length; i++) this.edge(g, this.pts[i - 1], this.pts[i].p, this.pts[i].h ? [-this.pts[i].h![0], -this.pts[i].h![1]] : null);
    const last = this.pts[this.pts.length - 1];
    if (!this.dragging) {
      g.stroke();
      g.setLineDash([5, 4]);
      g.beginPath();
      const sl = this.ctx.toScreen(last.p);
      g.moveTo(sl[0], sl[1]);
      this.edge(g, last, this.cur, null);
    }
    g.stroke();
    g.setLineDash([]);
    for (const d of this.pts) {
      const s = this.ctx.toScreen(d.p);
      g.beginPath();
      g.arc(s[0], s[1], 4, 0, Math.PI * 2);
      g.fillStyle = c.bg;
      g.fill();
      g.stroke();
      if (d.h) {
        const hs = this.ctx.toScreen([d.p[0] + d.h[0], d.p[1] + d.h[1]]);
        const hi = this.ctx.toScreen([d.p[0] - d.h[0], d.p[1] - d.h[1]]);
        g.beginPath();
        g.moveTo(hi[0], hi[1]);
        g.lineTo(hs[0], hs[1]);
        g.stroke();
        g.fillStyle = c.accent;
        g.beginPath();
        g.arc(hs[0], hs[1], 3, 0, Math.PI * 2);
        g.arc(hi[0], hi[1], 3, 0, Math.PI * 2);
        g.fill();
      }
    }
    if (this.pts.length >= 3 && dist(this.cur, this.pts[0].p) < 1e-6) {
      g.beginPath();
      g.arc(s0[0], s0[1], 8, 0, Math.PI * 2);
      g.stroke();
    }
    g.restore();
    if (!this.dragging) {
      const sc = this.ctx.toScreen(this.cur);
      worldLabel(g, fmtLen(dist(last.p, this.cur), unit), sc[0] + 12, sc[1] - 10, c);
    }
  }

  private edge(g: CanvasRenderingContext2D, from: Draft, to: Vec, hIn: Vec | null): void {
    const b = this.ctx.toScreen(to);
    if (from.h || hIn) {
      const c1 = this.ctx.toScreen(from.h ? [from.p[0] + from.h[0], from.p[1] + from.h[1]] : from.p);
      const c2 = this.ctx.toScreen(hIn ? [to[0] + hIn[0], to[1] + hIn[1]] : to);
      g.bezierCurveTo(c1[0], c1[1], c2[0], c2[1], b[0], b[1]);
    } else g.lineTo(b[0], b[1]);
  }
}

export class InnerLineTool extends BaseTool {
  id = 'inner_line';
  label = 'Inner Line';
  key = 'L';
  icon = 'innerLine';
  private pts: Vec[] = [];
  private panel: number | null = null;
  private cur: Vec = [0, 0];

  down(p: Ptr): void {
    if (this.panel === null) {
      this.panel = panelAt(this.ctx.doc.geos(), p.w) ?? this.ctx.sel.list()[0] ?? null;
      if (this.panel === null) { this.ctx.say('Start inside a panel'); return; }
    }
    const last = this.pts[this.pts.length - 1];
    const at = last && (p.shift || p.mod) ? constrain(last, p.w, p) : this.ctx.snap(p.w).pt;
    if (last && dist(last, at) < this.ctx.tol(3)) return;
    this.pts.push(at);
    this.cur = at;
    this.ctx.redraw();
  }

  move(p: Ptr): void {
    this.ctx.cursor('crosshair');
    const last = this.pts[this.pts.length - 1];
    this.cur = last && (p.shift || p.mod) ? constrain(last, p.w, p) : this.ctx.snap(p.w).pt;
    this.ctx.redraw();
  }

  up(): void { this.ctx.setSnapMark(null); }

  dbl(): void { this.finishLine(); }

  private finishLine(): void {
    if (this.pts.length < 2 || this.panel === null) return;
    const args = { panel: this.panel, points: this.pts };
    this.reset();
    void this.ctx.doc.run('pattern.inner_line', args);
  }

  private reset(): void {
    this.pts = [];
    this.panel = null;
    this.ctx.setSnapMark(null);
    this.ctx.redraw();
  }

  keydown(e: KeyboardEvent): boolean {
    if (e.key === 'Enter') {
      if (this.pts.length >= 2) this.finishLine(); else this.exact();
      return true;
    }
    if (e.key === 'Escape') { this.reset(); return true; }
    if ((e.key === 'Delete' || e.key === 'Backspace') && this.pts.length) {
      this.pts.pop();
      if (!this.pts.length) this.panel = null;
      this.ctx.redraw();
      return true;
    }
    return false;
  }

  exact(): void {
    const panel = this.panel ?? this.ctx.sel.list()[0];
    const init: Record<string, unknown> = {};
    if (panel !== undefined && panel !== null) init.panel = panel;
    if (this.pts.length >= 2) init.points = this.pts;
    void this.ctx.exact('pattern.inner_line', init).then((v) => {
      if (!v) return;
      this.reset();
      void this.ctx.doc.run('pattern.inner_line', v);
    });
  }

  cancel(): void { this.reset(); }

  busy(): boolean { return this.pts.length > 0; }

  leave(): void { this.reset(); }

  draw(g: CanvasRenderingContext2D, c: Colors): void {
    if (!this.pts.length) return;
    g.save();
    g.lineWidth = 1.5;
    g.strokeStyle = c.accent;
    g.beginPath();
    this.pts.forEach((p, i) => {
      const s = this.ctx.toScreen(p);
      if (i === 0) g.moveTo(s[0], s[1]); else g.lineTo(s[0], s[1]);
    });
    g.stroke();
    g.setLineDash([5, 4]);
    g.beginPath();
    const l = this.ctx.toScreen(this.pts[this.pts.length - 1]);
    const n = this.ctx.toScreen(this.cur);
    g.moveTo(l[0], l[1]);
    g.lineTo(n[0], n[1]);
    g.stroke();
    g.setLineDash([]);
    g.fillStyle = c.bg;
    for (const p of this.pts) {
      const s = this.ctx.toScreen(p);
      g.beginPath();
      g.arc(s[0], s[1], 3.5, 0, Math.PI * 2);
      g.fill();
      g.stroke();
    }
    g.restore();
    worldLabel(g, fmtLen(dist(this.pts[this.pts.length - 1], this.cur), this.ctx.unit()), n[0] + 12, n[1] - 10, c);
  }
}
