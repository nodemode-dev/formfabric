import { dist, place, unplace } from '../geom';
import { fmtLen } from '../state';
import type { Vec } from '../types';
import type { Colors, Ptr, ViewCtx } from '../pattern-view/ctx';
import { panelAt, worldLabel } from '../pattern-view/render';
import { BaseTool, constrain } from '../pattern-view/tools-common';
import { styleLines, type StyleLineInfo } from './store';

const round3 = (v: number): number => Math.round(v * 1000) / 1000;

export class StyleLineTool extends BaseTool {
  id = 'style_line';
  label = 'Style Line';
  key = 'Shift+M';
  icon = 'tStyleLine';
  private pts: Vec[] = [];
  private panel: number | null = null;
  private closed = false;
  private cur: Vec = [0, 0];

  constructor(ctx: ViewCtx) {
    super(ctx);
    styleLines.attach(ctx.doc);
    styleLines.onChange(() => this.ctx.redraw());
  }

  private world = (panelId: number, p: Vec): Vec => {
    const panel = this.ctx.doc.panel(panelId);
    return panel ? place(panel, p) : p;
  };

  down(p: Ptr): void {
    if (!this.pts.length) {
      const hit = styleLines.hit(p.w, this.world, this.ctx.tol(7));
      if (hit) {
        styleLines.select(hit.id);
        this.ctx.redraw();
        return;
      }
      this.panel = panelAt(this.ctx.doc.geos(), p.w) ?? this.ctx.sel.list()[0] ?? null;
      if (this.panel === null) { this.ctx.say('Start inside a panel'); return; }
      styleLines.select(null);
      this.ctx.say('Click points. Enter or double-click finishes, C closes.');
    }
    const last = this.pts[this.pts.length - 1];
    if (this.pts.length >= 3 && dist(p.w, this.pts[0]) < this.ctx.tol(8)) {
      this.closed = true;
      this.finish();
      return;
    }
    const at = last && (p.shift || p.mod) ? constrain(last, p.w, p) : this.ctx.snap(p.w).pt;
    if (last && dist(last, at) < this.ctx.tol(3)) return;
    this.pts.push(at);
    this.cur = at;
    this.ctx.redraw();
  }

  move(p: Ptr): void {
    this.ctx.cursor('crosshair');
    if (!this.pts.length) return;
    const last = this.pts[this.pts.length - 1];
    this.cur = last && (p.shift || p.mod) ? constrain(last, p.w, p) : this.ctx.snap(p.w).pt;
    this.ctx.redraw();
  }

  up(): void { this.ctx.setSnapMark(null); }

  dbl(): void { this.finish(); }

  exact(): void { this.finish(); }

  private finish(): void {
    const panel = this.panel === null ? undefined : this.ctx.doc.panel(this.panel);
    if (this.pts.length < 2 || !panel) return;
    const closed = this.closed && this.pts.length >= 3;
    const points = this.pts.map((w) => unplace(panel, w).map(round3));
    const args = { panel: panel.id, points, closed };
    this.reset();
    void this.ctx.doc.run<{ id?: number }>('styleline.add', args).then((r) => {
      if (r && typeof r.id === 'number') styleLines.select(r.id);
      this.ctx.redraw();
    });
  }

  private reset(): void {
    this.pts = [];
    this.panel = null;
    this.closed = false;
    this.ctx.setSnapMark(null);
    this.ctx.redraw();
  }

  private removeSelected(): void {
    const id = styleLines.selected;
    if (id === null) return;
    styleLines.select(null);
    void this.ctx.doc.run('styleline.remove', { id });
  }

  keydown(e: KeyboardEvent): boolean {
    if (e.ctrlKey || e.metaKey || e.altKey) return false;
    if (this.pts.length) {
      if (e.key === 'Enter') { this.finish(); return true; }
      if (e.key === 'Escape') { this.reset(); return true; }
      if (e.key === 'Delete' || e.key === 'Backspace') {
        this.pts.pop();
        if (!this.pts.length) this.panel = null;
        this.ctx.redraw();
        return true;
      }
      if (e.key.toLowerCase() === 'c' && !e.shiftKey) { this.closed = !this.closed; this.ctx.redraw(); return true; }
      return false;
    }
    if (styleLines.selected !== null) {
      if (e.key === 'Delete' || e.key === 'Backspace') { this.removeSelected(); return true; }
      if (e.key === 'Escape') { styleLines.select(null); this.ctx.redraw(); return true; }
    }
    return false;
  }

  cancel(): void { this.reset(); }

  busy(): boolean { return this.pts.length > 0; }

  leave(): void { this.reset(); }

  draw(g: CanvasRenderingContext2D, c: Colors): void {
    g.save();
    for (const l of styleLines.lines) this.stroke(g, c, l);
    if (this.pts.length) {
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
      if (this.closed && this.pts.length >= 3) {
        const f = this.ctx.toScreen(this.pts[0]);
        g.lineTo(f[0], f[1]);
      }
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
      worldLabel(g, fmtLen(dist(this.pts[this.pts.length - 1], this.cur), this.ctx.unit()), this.ctx.toScreen(this.cur)[0] + 12, this.ctx.toScreen(this.cur)[1] - 10, c);
      return;
    }
    g.restore();
  }

  private stroke(g: CanvasRenderingContext2D, c: Colors, l: StyleLineInfo): void {
    const on = l.id === styleLines.selected;
    g.lineWidth = on ? 2.5 : 1.5;
    g.strokeStyle = on ? c.accent : c.line;
    g.setLineDash(l.mode === 'none' ? [] : l.mode === 'stiffen' ? [8, 3] : [2, 3]);
    g.beginPath();
    l.points.forEach((pt, i) => {
      const s = this.ctx.toScreen(this.world(l.panel, pt));
      if (i === 0) g.moveTo(s[0], s[1]); else g.lineTo(s[0], s[1]);
    });
    if (l.closed) g.closePath();
    g.stroke();
    g.setLineDash([]);
    if (!on) return;
    g.fillStyle = c.bg;
    for (const pt of l.points) {
      const s = this.ctx.toScreen(this.world(l.panel, pt));
      g.beginPath();
      g.arc(s[0], s[1], 3.5, 0, Math.PI * 2);
      g.fill();
      g.stroke();
    }
  }
}
