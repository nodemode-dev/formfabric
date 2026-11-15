import { add, bezAt, dist, hitInner, hitTest, len, lerp, mul, segLength, sub, unionBounds, type Bounds, type Hit, type Seg } from '../geom';
import { fmtArea, fmtLen } from '../state';
import type { Step } from '../doc';
import type { PanelMeasure, Vec } from '../types';
import type { Colors, Ptr, ViewCtx } from './ctx';
import { worldLabel } from './render';
import { BaseTool, constrain, moved, nearestVertex } from './tools-common';

type Mode = 'idle' | 'pending' | 'move' | 'marquee' | 'scale' | 'rotate';

const HANDLE_CURSOR = ['nesw-resize', 'nwse-resize', 'nesw-resize', 'nwse-resize', 'ns-resize', 'ew-resize', 'ns-resize', 'ew-resize'];
const ROTATE_GAP = 24;

export class SelectTool extends BaseTool {
  id = 'select';
  label = 'Select';
  key = 'A';
  icon = 'tSelect';
  private mode: Mode = 'idle';
  private start: Ptr | null = null;
  private anchor: Vec = [0, 0];
  private applied: Vec = [0, 0];
  private appliedRot = 0;
  private handle = -1;
  private pivot: Vec = [0, 0];
  private h0: Vec = [0, 0];
  private a0 = 0;
  private marquee: [Vec, Vec] | null = null;
  private toggleOff: number | null = null;
  private warm = false;

  private bounds(): Bounds | null {
    let b: Bounds | null = null;
    for (const geo of this.ctx.doc.geos()) if (this.ctx.sel.panels.has(geo.panel.id)) b = unionBounds(b, geo.bounds);
    return b;
  }

  private handles(b: Bounds): Vec[] {
    const [x0, y0, x1, y1] = b;
    const mx = (x0 + x1) / 2;
    const my = (y0 + y1) / 2;
    return [[x0, y0], [x1, y0], [x1, y1], [x0, y1], [mx, y0], [x1, my], [mx, y1], [x0, my]];
  }

  private rotatePoint(b: Bounds): Vec {
    const [x, y] = this.ctx.toScreen([(b[0] + b[2]) / 2, b[3]]);
    return this.ctx.toWorld(x, y - ROTATE_GAP);
  }

  private handleAt(p: Ptr, b: Bounds): number {
    const reach = this.ctx.tol(8);
    if (dist(this.rotatePoint(b), p.w) <= reach) return 8;
    const hs = this.handles(b);
    for (let i = 0; i < hs.length; i++) if (dist(hs[i], p.w) <= reach) return i;
    return -1;
  }

  private ids(): number[] { return this.ctx.sel.list(); }

  down(p: Ptr): void {
    this.start = p;
    this.toggleOff = null;
    const b = this.bounds();
    if (b) {
      const h = this.handleAt(p, b);
      if (h >= 0) {
        this.handle = h;
        const c: Vec = [(b[0] + b[2]) / 2, (b[1] + b[3]) / 2];
        if (h === 8) {
          this.mode = 'rotate';
          this.pivot = c;
          this.a0 = Math.atan2(p.w[1] - c[1], p.w[0] - c[0]);
          this.appliedRot = 0;
        } else {
          const hs = this.handles(b);
          this.mode = 'scale';
          this.h0 = hs[h];
          this.pivot = h < 4 ? hs[(h + 2) % 4] : hs[4 + ((h - 4 + 2) % 4)];
          this.applied = [1, 1];
        }
        this.ctx.doc.beginPreview('Transform');
        return;
      }
    }
    const hit = hitTest(this.ctx.doc.geos(), p.w, this.ctx.tol(4), { handles: false, points: false, segments: true, panels: true });
    if (hit) {
      const picked = this.ctx.sel.panels.has(hit.panel);
      if (p.shift) {
        if (picked) this.toggleOff = hit.panel;
        else this.ctx.sel.panels.add(hit.panel);
      } else if (!picked) this.ctx.sel.only(hit.panel);
      const inner = hitInner(this.ctx.doc.geos(), p.w, this.ctx.tol(5), false);
      this.ctx.sel.inner = inner ? { panel: inner.panel, id: inner.inner } : null;
      this.ctx.selected();
      this.mode = 'pending';
      this.anchor = nearestVertex(this.ctx.doc.geos().filter((g) => this.ctx.sel.panels.has(g.panel.id)).flatMap((g) => g.pts), p.w) ?? p.w;
      this.warm = true;
      this.ctx.doc.beginPreview('Move Panels');
      this.ctx.doc.prime(this.ids(), 'pattern.transform', { panels: this.ids(), translate: [0, 0] });
    } else {
      if (!p.shift) { this.ctx.sel.clear(); this.ctx.selected(); }
      this.mode = 'marquee';
      this.marquee = [p.w, p.w];
    }
    this.ctx.redraw();
  }

  move(p: Ptr): void {
    const s = this.start;
    if (this.mode === 'idle' || !s) {
      const b = this.bounds();
      const h = b ? this.handleAt(p, b) : -1;
      if (h >= 0) { this.ctx.cursor(h === 8 ? 'crosshair' : HANDLE_CURSOR[h]); this.ctx.setHover(null); return; }
      const hit = hitTest(this.ctx.doc.geos(), p.w, this.ctx.tol(4), { handles: false, points: false, segments: true, panels: true });
      this.ctx.cursor(hit ? 'move' : 'default');
      this.ctx.setHover(hit ? { ...hit, kind: 'panel', index: -1 } : null);
      return;
    }
    if (this.mode === 'pending' && moved(s, p)) {
      if (!this.warm) this.ctx.doc.beginPreview('Move Panels');
      this.warm = false;
      this.mode = 'move';
      this.applied = [0, 0];
    }
    if (this.mode === 'move') {
      let d = sub(p.w, s.w);
      if (p.shift || p.mod) d = sub(constrain(s.w, p.w, p), s.w);
      const want = add(this.anchor, d);
      const snapped = p.shift || p.mod ? { pt: want, kind: null } : this.ctx.snap(want, this.ids());
      const target = sub(snapped.pt, this.anchor);
      const inc = sub(target, this.applied);
      if (inc[0] !== 0 || inc[1] !== 0) {
        this.ctx.doc.preview('pattern.transform', { panels: this.ids(), translate: inc }, this.ids());
        this.applied = target;
      }
    } else if (this.mode === 'scale') {
      const v = sub(p.w, this.pivot);
      const v0 = sub(this.h0, this.pivot);
      const safe = (n: number, d: number): number => (Math.abs(d) < 1e-6 ? 1 : Math.max(0.02, n / d));
      let sx = 1;
      let sy = 1;
      if (this.handle < 4) {
        if (p.shift) { const k = Math.max(0.02, (v[0] * v0[0] + v[1] * v0[1]) / (v0[0] * v0[0] + v0[1] * v0[1])); sx = k; sy = k; }
        else { sx = safe(v[0], v0[0]); sy = safe(v[1], v0[1]); }
      } else if (this.handle % 2 === 0) sy = safe(v[1], v0[1]);
      else sx = safe(v[0], v0[0]);
      const inc: Vec = [sx / this.applied[0], sy / this.applied[1]];
      if (Math.abs(inc[0] - 1) > 1e-9 || Math.abs(inc[1] - 1) > 1e-9) {
        this.ctx.doc.preview('pattern.transform', { panels: this.ids(), scale: inc, pivot: this.pivot }, this.ids());
        this.applied = [sx, sy];
      }
    } else if (this.mode === 'rotate') {
      let deg = ((Math.atan2(p.w[1] - this.pivot[1], p.w[0] - this.pivot[0]) - this.a0) * 180) / Math.PI;
      if (p.shift) deg = Math.round(deg / 15) * 15;
      const inc = deg - this.appliedRot;
      if (Math.abs(inc) > 1e-9) {
        this.ctx.doc.preview('pattern.transform', { panels: this.ids(), rotate: inc, pivot: this.pivot }, this.ids());
        this.appliedRot = deg;
      }
    } else if (this.mode === 'marquee' && this.marquee) {
      this.marquee[1] = p.w;
    }
    this.ctx.redraw();
  }

  up(p: Ptr): void {
    const mode = this.mode;
    const ids = this.ids();
    this.mode = 'idle';
    this.start = null;
    this.ctx.setSnapMark(null);
    const still = (mode === 'move' && this.applied[0] === 0 && this.applied[1] === 0) || (mode === 'scale' && this.applied[0] === 1 && this.applied[1] === 1) || (mode === 'rotate' && this.appliedRot === 0);
    if (still) {
      void this.ctx.doc.cancelPreview();
    } else if (mode === 'move') {
      void this.ctx.doc.commitPreview([['pattern.transform', { panels: ids, translate: this.applied }]], 'Move Panels');
    } else if (mode === 'scale') {
      const [sx, sy] = this.applied;
      void this.ctx.doc.commitPreview([['pattern.transform', { panels: ids, scale: sx === sy ? sx : [sx, sy], pivot: this.pivot }]], 'Scale Panels');
    } else if (mode === 'rotate') {
      void this.ctx.doc.commitPreview([['pattern.transform', { panels: ids, rotate: this.appliedRot, pivot: this.pivot }]], 'Rotate Panels');
    } else if (mode === 'marquee' && this.marquee) {
      const [a, b] = this.marquee;
      const box: Bounds = [Math.min(a[0], b[0]), Math.min(a[1], b[1]), Math.max(a[0], b[0]), Math.max(a[1], b[1])];
      if (box[2] - box[0] > this.ctx.tol(2) || box[3] - box[1] > this.ctx.tol(2)) {
        for (const g of this.ctx.doc.geos()) {
          const inside = g.bounds[0] >= box[0] && g.bounds[1] >= box[1] && g.bounds[2] <= box[2] && g.bounds[3] <= box[3];
          const touch = g.bounds[2] >= box[0] && g.bounds[0] <= box[2] && g.bounds[3] >= box[1] && g.bounds[1] <= box[3];
          if (p.alt ? touch : inside) this.ctx.sel.panels.add(g.panel.id);
        }
        this.ctx.selected();
      }
      this.marquee = null;
    } else if (mode === 'pending') {
      if (this.warm) this.ctx.doc.discardPreview();
      if (this.toggleOff !== null) {
        this.ctx.sel.panels.delete(this.toggleOff);
        this.ctx.selected();
      }
    }
    this.warm = false;
    this.toggleOff = null;
    this.ctx.redraw();
  }

  keydown(e: KeyboardEvent): boolean {
    const ids = this.ids();
    if (e.key === 'Escape') {
      if (this.busy()) this.cancel();
      else if (ids.length) { this.ctx.sel.clear(); this.ctx.selected(); this.ctx.redraw(); }
      return true;
    }
    if (e.key === 'Enter') { this.exact(); return true; }
    if (!ids.length) return false;
    if (e.key === 'Delete' || e.key === 'Backspace') {
      void this.ctx.doc.run('pattern.delete', { panels: ids });
      return true;
    }
    const arrows: Record<string, Vec> = { ArrowLeft: [-1, 0], ArrowRight: [1, 0], ArrowUp: [0, 1], ArrowDown: [0, -1] };
    const a = arrows[e.key];
    if (a) {
      void this.ctx.doc.run('pattern.transform', { panels: ids, translate: mul(a, e.shiftKey ? 10 : 1) });
      return true;
    }
    return false;
  }

  exact(): void {
    const ids = this.ids();
    if (!ids.length) { this.ctx.say('Select a panel first'); return; }
    void this.ctx.exact('pattern.transform', { panels: ids }).then((v) => { if (v) void this.ctx.doc.run('pattern.transform', v); });
  }

  cancel(): void {
    if (this.mode === 'move' || this.mode === 'scale' || this.mode === 'rotate') void this.ctx.doc.cancelPreview();
    else if (this.warm) this.ctx.doc.discardPreview();
    this.warm = false;
    this.mode = 'idle';
    this.marquee = null;
    this.start = null;
    this.ctx.setSnapMark(null);
    this.ctx.redraw();
  }

  busy(): boolean { return this.mode !== 'idle'; }

  leave(): void { this.cancel(); }

  draw(g: CanvasRenderingContext2D, c: Colors): void {
    if (this.marquee) {
      const a = this.ctx.toScreen(this.marquee[0]);
      const b = this.ctx.toScreen(this.marquee[1]);
      g.save();
      g.setLineDash([4, 3]);
      g.strokeStyle = c.accent;
      g.fillStyle = c.accent;
      g.globalAlpha = 0.1;
      g.fillRect(Math.min(a[0], b[0]), Math.min(a[1], b[1]), Math.abs(a[0] - b[0]), Math.abs(a[1] - b[1]));
      g.globalAlpha = 1;
      g.strokeRect(Math.min(a[0], b[0]) + 0.5, Math.min(a[1], b[1]) + 0.5, Math.abs(a[0] - b[0]), Math.abs(a[1] - b[1]));
      g.restore();
    }
    const b = this.bounds();
    if (!b) return;
    const tl = this.ctx.toScreen([b[0], b[3]]);
    const br = this.ctx.toScreen([b[2], b[1]]);
    g.save();
    g.strokeStyle = c.accent;
    g.lineWidth = 1;
    g.setLineDash([5, 4]);
    g.strokeRect(Math.round(tl[0]) + 0.5, Math.round(tl[1]) + 0.5, Math.round(br[0] - tl[0]), Math.round(br[1] - tl[1]));
    g.setLineDash([]);
    g.fillStyle = c.bg;
    for (const h of this.handles(b)) {
      const s = this.ctx.toScreen(h);
      g.fillRect(s[0] - 4, s[1] - 4, 8, 8);
      g.strokeRect(s[0] - 4 + 0.5, s[1] - 4 + 0.5, 8, 8);
    }
    const top = this.ctx.toScreen([(b[0] + b[2]) / 2, b[3]]);
    g.beginPath();
    g.moveTo(top[0], top[1]);
    g.lineTo(top[0], top[1] - ROTATE_GAP);
    g.stroke();
    g.beginPath();
    g.arc(top[0], top[1] - ROTATE_GAP, 5, 0, Math.PI * 2);
    g.fill();
    g.stroke();
    g.restore();
  }
}

type EditMode = 'idle' | 'pending' | 'point' | 'segment' | 'handle' | 'pull' | 'curve' | 'marquee';

export class EditTool extends BaseTool {
  id = 'edit';
  label = 'Edit Points';
  key = 'Z';
  icon = 'editPoint';
  private mode: EditMode = 'idle';
  private start: Ptr | null = null;
  private hit: Hit | null = null;
  private live = false;
  private origin: Vec = [0, 0];
  private applied: Vec = [0, 0];
  private last: Step[] = [];
  private marquee: [Vec, Vec] | null = null;
  private warm = false;

  private warmUp(): void {
    const hit = this.hit;
    const sel = this.ctx.sel;
    if (!hit || this.mode === 'idle' || this.mode === 'marquee') return;
    this.warm = true;
    this.ctx.doc.beginPreview('Edit Points');
    if (this.mode === 'point' && sel.pts) this.ctx.doc.prime([sel.pts.panel], 'pattern.move_point', { panel: sel.pts.panel, indices: sel.pts.idx, delta: [0, 0], space: 'world' });
    else if (this.mode === 'segment' && sel.seg) {
      const n = this.ctx.doc.panel(sel.seg.panel)?.outline.length ?? 1;
      this.ctx.doc.prime([sel.seg.panel], 'pattern.move_point', { panel: sel.seg.panel, indices: [sel.seg.i, (sel.seg.i + 1) % n], delta: [0, 0], space: 'world' });
    } else this.ctx.doc.prime([hit.panel]);
  }

  private pointPos(panel: number, i: number): Vec | null {
    const g = this.ctx.doc.geos().find((x) => x.panel.id === panel);
    return g?.pts[i] ?? null;
  }

  down(p: Ptr): void {
    this.start = p;
    this.live = false;
    this.last = [];
    const hit = hitTest(this.ctx.doc.geos(), p.w, this.ctx.tol(7), { handles: true, points: true, segments: true, panels: true });
    this.hit = hit;
    const sel = this.ctx.sel;
    if (!hit) {
      if (!p.shift) { sel.clear(); this.ctx.selected(); }
      this.mode = 'marquee';
      this.marquee = [p.w, p.w];
    } else if (hit.kind === 'handle') {
      sel.only(hit.panel);
      sel.pts = { panel: hit.panel, idx: [hit.index] };
      this.mode = 'handle';
      this.origin = this.pointPos(hit.panel, hit.index) ?? p.w;
    } else if (hit.kind === 'point') {
      if (sel.pts && sel.pts.panel === hit.panel && p.shift) {
        const at = sel.pts.idx.indexOf(hit.index);
        if (at >= 0) sel.pts.idx.splice(at, 1); else sel.pts.idx.push(hit.index);
      } else if (!(sel.pts && sel.pts.panel === hit.panel && sel.pts.idx.includes(hit.index))) {
        sel.only(hit.panel);
        sel.pts = { panel: hit.panel, idx: [hit.index] };
      }
      this.origin = hit.pt ?? p.w;
      this.mode = p.alt ? 'pull' : 'point';
    } else if (hit.kind === 'segment') {
      sel.only(hit.panel);
      sel.seg = { panel: hit.panel, i: hit.index };
      this.mode = p.alt ? 'curve' : 'segment';
    } else {
      sel.only(hit.panel);
      this.mode = 'idle';
    }
    this.ctx.selected();
    this.warmUp();
    this.ctx.redraw();
  }

  private begin(label: string): void {
    if (this.live) return;
    this.live = true;
    this.applied = [0, 0];
    if (!this.warm) this.ctx.doc.beginPreview(label);
    this.warm = false;
  }

  private run(name: string, args: Record<string, unknown>, panel: number): void {
    this.ctx.doc.preview(name, args, [panel]);
  }

  move(p: Ptr): void {
    const s = this.start;
    if (this.mode === 'idle' || !s) {
      const hit = hitTest(this.ctx.doc.geos(), p.w, this.ctx.tol(7), { handles: true, points: true, segments: true, panels: false });
      this.ctx.setHover(hit && hit.kind === 'segment' ? hit : null);
      this.ctx.cursor(hit ? (hit.kind === 'segment' ? 'copy' : 'move') : 'default');
      return;
    }
    const hit = this.hit;
    const sel = this.ctx.sel;
    if (this.mode === 'marquee' && this.marquee) {
      this.marquee[1] = p.w;
    } else if (!hit || (!moved(s, p) && !this.live)) {
      return;
    } else if (this.mode === 'handle' && hit.which) {
      this.begin('Edit Handle');
      let v = sub(p.w, this.origin);
      if (p.shift) v = sub(constrain(this.origin, p.w, { shift: true, mod: false }), this.origin);
      const args = { panel: hit.panel, index: hit.index, which: hit.which, vector: v, space: 'world', independent: p.alt };
      this.run('pattern.set_handle', args, hit.panel);
      this.last = [['pattern.set_handle', args]];
    } else if (this.mode === 'pull') {
      const idx = sel.pts?.idx[0] ?? hit.index;
      if (!this.live) {
        this.begin('Curve Point');
        this.run('pattern.set_point_kind', { panel: hit.panel, index: idx, kind: 'smooth' }, hit.panel);
      }
      const args = { panel: hit.panel, index: idx, which: 'out', vector: sub(p.w, this.origin), space: 'world' };
      this.run('pattern.set_handle', args, hit.panel);
      this.last = [['pattern.set_point_kind', { panel: hit.panel, index: idx, kind: 'smooth' }], ['pattern.set_handle', args]];
    } else if (this.mode === 'point' && sel.pts) {
      this.begin('Move Point');
      const panel = sel.pts.panel;
      const idx = sel.pts.idx;
      if (idx.length === 1) {
        let want = add(this.origin, sub(p.w, s.w));
        if (p.shift || p.mod) want = constrain(this.origin, want, p);
        else want = this.ctx.snap(want, [panel]).pt;
        const args = { panel, index: idx[0], to: want, space: 'world' };
        this.run('pattern.move_point', args, panel);
        this.last = [['pattern.move_point', args]];
      } else {
        let want = sub(p.w, s.w);
        if (p.shift || p.mod) want = sub(constrain([0, 0], want, p), [0, 0]);
        const inc = sub(want, this.applied);
        this.run('pattern.move_point', { panel, indices: idx, delta: inc, space: 'world' }, panel);
        this.applied = want;
        this.last = [['pattern.move_point', { panel, indices: idx, delta: want, space: 'world' }]];
      }
    } else if (this.mode === 'segment' && sel.seg) {
      this.begin('Move Segment');
      const n = this.ctx.doc.panel(sel.seg.panel)?.outline.length ?? 0;
      const indices = [sel.seg.i, (sel.seg.i + 1) % n];
      let want = sub(p.w, s.w);
      if (p.shift || p.mod) want = sub(constrain([0, 0], want, p), [0, 0]);
      const inc = sub(want, this.applied);
      this.run('pattern.move_point', { panel: sel.seg.panel, indices, delta: inc, space: 'world' }, sel.seg.panel);
      this.applied = want;
      this.last = [['pattern.move_point', { panel: sel.seg.panel, indices, delta: want, space: 'world' }]];
    } else if (this.mode === 'curve' && sel.seg) {
      this.begin('Curvature');
      const amount = curveAmount(this.ctx, sel.seg.panel, sel.seg.i, p.w);
      const args = { panel: sel.seg.panel, segment: sel.seg.i, amount };
      this.run('pattern.curvature', args, sel.seg.panel);
      this.last = [['pattern.curvature', args]];
    }
    this.ctx.redraw();
  }

  up(p: Ptr): void {
    const mode = this.mode;
    const live = this.live;
    this.mode = 'idle';
    this.start = null;
    this.ctx.setSnapMark(null);
    if (this.warm && !live) { this.warm = false; this.ctx.doc.discardPreview(); }
    if (mode === 'marquee' && this.marquee) {
      const [a, b] = this.marquee;
      const box: Bounds = [Math.min(a[0], b[0]), Math.min(a[1], b[1]), Math.max(a[0], b[0]), Math.max(a[1], b[1])];
      if (box[2] - box[0] > this.ctx.tol(2) || box[3] - box[1] > this.ctx.tol(2)) {
        for (const g of this.ctx.doc.geos()) {
          const idx: number[] = [];
          g.pts.forEach((q, i) => { if (q[0] >= box[0] && q[0] <= box[2] && q[1] >= box[1] && q[1] <= box[3]) idx.push(i); });
          if (idx.length) { this.ctx.sel.panels.add(g.panel.id); this.ctx.sel.pts = { panel: g.panel.id, idx }; }
        }
        this.ctx.selected();
      }
      this.marquee = null;
    } else if (live) {
      const steps = this.last;
      this.live = false;
      void this.ctx.doc.commitPreview(steps, this.stepLabel(mode));
    }
    void p;
    this.ctx.redraw();
  }

  private stepLabel(mode: EditMode): string {
    return mode === 'handle' ? 'Edit Handle' : mode === 'pull' ? 'Curve Point' : mode === 'segment' ? 'Move Segment' : mode === 'curve' ? 'Curvature' : 'Move Point';
  }

  dbl(p: Ptr): void {
    const hit = hitTest(this.ctx.doc.geos(), p.w, this.ctx.tol(7), { handles: false, points: false, segments: true, panels: false });
    if (hit && hit.t !== undefined) void this.ctx.doc.run('pattern.add_point', { panel: hit.panel, segment: hit.index, t: hit.t });
  }

  keydown(e: KeyboardEvent): boolean {
    const sel = this.ctx.sel;
    if (e.key === 'Escape') {
      if (this.busy()) this.cancel();
      else { sel.pts = null; sel.seg = null; this.ctx.selected(); this.ctx.redraw(); }
      return true;
    }
    if (e.key === 'Enter') { this.exact(); return true; }
    if ((e.key === 'Delete' || e.key === 'Backspace') && sel.pts && sel.pts.idx.length) {
      const panel = sel.pts.panel;
      const steps: Step[] = [...sel.pts.idx].sort((a, b) => b - a).map((index) => ['pattern.delete_point', { panel, index }]);
      sel.pts = null;
      void this.ctx.doc.commit(steps, 'Delete Points');
      return true;
    }
    const arrows: Record<string, Vec> = { ArrowLeft: [-1, 0], ArrowRight: [1, 0], ArrowUp: [0, 1], ArrowDown: [0, -1] };
    const a = arrows[e.key];
    if (a && sel.pts) {
      void this.ctx.doc.run('pattern.move_point', { panel: sel.pts.panel, indices: sel.pts.idx, delta: mul(a, e.shiftKey ? 10 : 1) });
      return true;
    }
    return false;
  }

  exact(): void {
    const sel = this.ctx.sel;
    if (sel.pts && sel.pts.idx.length === 1) {
      const pos = this.pointPos(sel.pts.panel, sel.pts.idx[0]);
      void this.ctx.exact('pattern.move_point', { panel: sel.pts.panel, index: sel.pts.idx[0], to: pos ?? undefined }).then((v) => { if (v) void this.ctx.doc.run('pattern.move_point', v); });
    } else if (sel.seg) {
      void this.ctx.exact('pattern.curvature', { panel: sel.seg.panel, segment: sel.seg.i }).then((v) => { if (v) void this.ctx.doc.run('pattern.curvature', v); });
    } else this.ctx.say('Select a point first');
  }

  cancel(): void {
    if (this.live) { this.live = false; void this.ctx.doc.cancelPreview(); }
    else if (this.warm) this.ctx.doc.discardPreview();
    this.warm = false;
    this.mode = 'idle';
    this.marquee = null;
    this.start = null;
    this.ctx.setSnapMark(null);
    this.ctx.redraw();
  }

  busy(): boolean { return this.mode !== 'idle'; }

  leave(): void { this.cancel(); }

  draw(g: CanvasRenderingContext2D, c: Colors): void {
    if (!this.marquee) return;
    const a = this.ctx.toScreen(this.marquee[0]);
    const b = this.ctx.toScreen(this.marquee[1]);
    g.save();
    g.setLineDash([4, 3]);
    g.strokeStyle = c.accent;
    g.strokeRect(Math.min(a[0], b[0]) + 0.5, Math.min(a[1], b[1]) + 0.5, Math.abs(a[0] - b[0]), Math.abs(a[1] - b[1]));
    g.restore();
  }
}

export function curveAmount(ctx: ViewCtx, panel: number, seg: number, w: Vec): number {
  const geo = ctx.doc.geos().find((x) => x.panel.id === panel);
  if (!geo) return 0;
  const a = geo.pts[seg];
  const b = geo.pts[(seg + 1) % geo.pts.length];
  const d = sub(b, a);
  const l = len(d);
  if (l < 1e-9) return 0;
  const right: Vec = [d[1] / l, -d[0] / l];
  const amount = (w[0] - a[0]) * right[0] + (w[1] - a[1]) * right[1];
  return geo.panel.mirror ? -amount : amount;
}

export class SplitTool extends BaseTool {
  id = 'split';
  label = 'Split';
  key = 'S';
  icon = 'split';
  private cur: { panel: number; seg: number; t: number; pt: Vec; a: number; b: number } | null = null;

  private locate(p: Ptr): void {
    const hit = hitTest(this.ctx.doc.geos(), p.w, this.ctx.tol(10), { handles: false, points: false, segments: true, panels: false });
    if (!hit || hit.t === undefined) { this.cur = null; this.ctx.setHover(null); return; }
    const geo = this.ctx.doc.geos().find((x) => x.panel.id === hit.panel);
    const seg = geo?.outline[hit.index];
    if (!geo || !seg) { this.cur = null; return; }
    let t = hit.t;
    const total = segLength(seg);
    const midPt = bezAt(seg, 0.5);
    if (dist(midPt, p.w) <= this.ctx.tol(8)) t = 0.5;
    const pt = t === hit.t ? (hit.pt ?? p.w) : midPt;
    const first = splitLen(seg, t);
    this.cur = { panel: hit.panel, seg: hit.index, t, pt, a: first, b: total - first };
    this.ctx.setHover(hit);
  }

  move(p: Ptr): void {
    this.locate(p);
    this.ctx.cursor(this.cur ? 'copy' : 'default');
    this.ctx.redraw();
  }

  down(p: Ptr): void {
    this.locate(p);
  }

  up(_p: Ptr): void {
    const c = this.cur;
    if (!c) return;
    this.cur = null;
    void this.ctx.doc.run('pattern.split_line', { panel: c.panel, segment: c.seg, t: c.t });
  }

  keydown(e: KeyboardEvent): boolean {
    if (e.key === 'Enter') { this.exact(); return true; }
    if (e.key === 'Escape') { this.cur = null; this.ctx.redraw(); return true; }
    return false;
  }

  exact(): void {
    const c = this.cur;
    const init = c ? { panel: c.panel, segment: c.seg, t: c.t } : {};
    void this.ctx.exact('pattern.split_line', init).then((v) => { if (v) void this.ctx.doc.run('pattern.split_line', v); });
  }

  cancel(): void { this.cur = null; this.ctx.redraw(); }

  leave(): void { this.cur = null; }

  draw(g: CanvasRenderingContext2D, c: Colors): void {
    const cur = this.cur;
    if (!cur) return;
    const s = this.ctx.toScreen(cur.pt);
    g.save();
    g.fillStyle = c.accent;
    g.beginPath();
    g.arc(s[0], s[1], 4.5, 0, Math.PI * 2);
    g.fill();
    g.restore();
    const unit = this.ctx.unit();
    worldLabel(g, `${fmtLen(cur.a, unit)} | ${fmtLen(cur.b, unit)}`, s[0] + 12, s[1] - 10, c);
  }
}

function hitMid(seg: Seg): Vec {
  let total = 0;
  const pts: Vec[] = [seg.a];
  for (let i = 1; i <= 24; i++) {
    const prev = pts[pts.length - 1];
    const t = i / 24;
    const u = 1 - t;
    const p: Vec = [u * u * u * seg.a[0] + 3 * u * u * t * seg.c1[0] + 3 * u * t * t * seg.c2[0] + t * t * t * seg.b[0], u * u * u * seg.a[1] + 3 * u * u * t * seg.c1[1] + 3 * u * t * t * seg.c2[1] + t * t * t * seg.b[1]];
    total += dist(prev, p);
    pts.push(p);
  }
  let acc = 0;
  for (let i = 1; i < pts.length; i++) {
    const d = dist(pts[i - 1], pts[i]);
    if (acc + d >= total / 2) return lerp(pts[i - 1], pts[i], d === 0 ? 0 : (total / 2 - acc) / d);
    acc += d;
  }
  return seg.b;
}

function splitLen(seg: Seg, t: number): number {
  if (!seg.curved) return dist(seg.a, seg.b) * t;
  let total = 0;
  let prev = seg.a;
  const n = Math.max(2, Math.ceil(24 * t));
  for (let i = 1; i <= n; i++) {
    const tt = (t * i) / n;
    const u = 1 - tt;
    const p: Vec = [u * u * u * seg.a[0] + 3 * u * u * tt * seg.c1[0] + 3 * u * tt * tt * seg.c2[0] + tt * tt * tt * seg.b[0], u * u * u * seg.a[1] + 3 * u * u * tt * seg.c1[1] + 3 * u * tt * tt * seg.c2[1] + tt * tt * tt * seg.b[1]];
    total += dist(prev, p);
    prev = p;
  }
  return total;
}

export class CurvatureTool extends BaseTool {
  id = 'curvature';
  label = 'Curvature';
  key = 'C';
  icon = 'curve';
  private target: { panel: number; seg: number } | null = null;
  private live = false;
  private amount = 0;
  private start: Ptr | null = null;

  down(p: Ptr): void {
    const hit = hitTest(this.ctx.doc.geos(), p.w, this.ctx.tol(8), { handles: false, points: false, segments: true, panels: false });
    if (!hit) return;
    this.target = { panel: hit.panel, seg: hit.index };
    this.ctx.sel.only(hit.panel);
    this.ctx.sel.seg = { panel: hit.panel, i: hit.index };
    this.ctx.selected();
    this.start = p;
    this.live = false;
  }

  move(p: Ptr): void {
    const t = this.target;
    if (!t || !this.start) {
      const hit = hitTest(this.ctx.doc.geos(), p.w, this.ctx.tol(8), { handles: false, points: false, segments: true, panels: false });
      this.ctx.setHover(hit);
      this.ctx.cursor(hit ? 'pointer' : 'default');
      return;
    }
    if (!this.live) {
      if (!moved(this.start, p)) return;
      this.live = true;
      this.ctx.doc.beginPreview('Curvature');
    }
    this.amount = curveAmount(this.ctx, t.panel, t.seg, p.w);
    this.ctx.doc.preview('pattern.curvature', { panel: t.panel, segment: t.seg, amount: this.amount }, [t.panel]);
    this.ctx.redraw();
  }

  up(_p: Ptr): void {
    const t = this.target;
    const live = this.live;
    this.target = null;
    this.start = null;
    this.live = false;
    if (t && live) void this.ctx.doc.commitPreview([['pattern.curvature', { panel: t.panel, segment: t.seg, amount: this.amount }]], 'Curvature');
  }

  keydown(e: KeyboardEvent): boolean {
    if (e.key === 'Enter') { this.exact(); return true; }
    if (e.key === 'Escape') { this.cancel(); return true; }
    return false;
  }

  exact(): void {
    const s = this.ctx.sel.seg;
    if (!s) { this.ctx.say('Select a segment first'); return; }
    void this.ctx.exact('pattern.curvature', { panel: s.panel, segment: s.i }).then((v) => { if (v) void this.ctx.doc.run('pattern.curvature', v); });
  }

  cancel(): void {
    if (this.live) void this.ctx.doc.cancelPreview();
    this.target = null;
    this.start = null;
    this.live = false;
  }

  busy(): boolean { return this.target !== null; }

  leave(): void { this.cancel(); }
}

export class MeasureTool extends BaseTool {
  id = 'measure';
  label = 'Measure';
  key = 'M';
  icon = 'ruler';
  private hover: { pt: Vec; length: number } | null = null;
  private shown: { panel: number; labels: Array<{ pt: Vec; length: number }> } | null = null;

  move(p: Ptr): void {
    const hit = hitTest(this.ctx.doc.geos(), p.w, this.ctx.tol(8), { handles: false, points: false, segments: true, panels: false });
    this.ctx.setHover(hit);
    if (hit) {
      const geo = this.ctx.doc.geos().find((x) => x.panel.id === hit.panel);
      const seg = geo?.outline[hit.index];
      this.hover = seg ? { pt: p.w, length: segLength(seg) } : null;
    } else this.hover = null;
    this.ctx.cursor(hit ? 'pointer' : 'default');
    this.ctx.redraw();
  }

  down(p: Ptr): void {
    const hit = hitTest(this.ctx.doc.geos(), p.w, this.ctx.tol(8), { handles: false, points: false, segments: true, panels: true });
    if (!hit) { this.shown = null; this.ctx.say(''); this.ctx.redraw(); return; }
    const geo = this.ctx.doc.geos().find((x) => x.panel.id === hit.panel);
    const m = this.ctx.doc.measure(hit.panel);
    if (!geo) return;
    this.shown = {
      panel: hit.panel,
      labels: geo.outline.map((s, i) => ({ pt: hitMid(s), length: m?.segments[i]?.length ?? segLength(s) })),
    };
    if (m) this.report(m);
    this.ctx.redraw();
  }

  private report(m: PanelMeasure): void {
    const u = this.ctx.unit();
    this.ctx.say(`${m.name}: ${fmtLen(m.perimeter, u)} · ${fmtArea(m.area, u)}`);
  }

  keydown(e: KeyboardEvent): boolean {
    if (e.key === 'Escape') { this.shown = null; this.ctx.say(''); this.ctx.redraw(); return true; }
    if (e.key === 'Enter') { this.exact(); return true; }
    return false;
  }

  exact(): void {
    const id = this.shown?.panel ?? this.ctx.sel.list()[0];
    void this.ctx.exact('pattern.measure', id ? { panel: id } : {}).then((v) => {
      if (!v) return;
      const m = this.ctx.doc.measure(Number(v.panel ?? id));
      if (m) this.report(m);
    });
  }

  cancel(): void { this.shown = null; this.hover = null; this.ctx.redraw(); }

  leave(): void { this.shown = null; this.hover = null; }

  draw(g: CanvasRenderingContext2D, c: Colors): void {
    const u = this.ctx.unit();
    if (this.shown) {
      const geo = this.ctx.doc.geos().find((x) => x.panel.id === this.shown?.panel);
      if (!geo) { this.shown = null; return; }
      for (const l of this.shown.labels) {
        const s = this.ctx.toScreen(l.pt);
        worldLabel(g, fmtLen(l.length, u, false), s[0] - 16, s[1] + 4, c);
      }
    }
    if (this.hover) {
      const s = this.ctx.toScreen(this.hover.pt);
      worldLabel(g, fmtLen(this.hover.length, u), s[0] + 12, s[1] - 10, c);
    }
  }
}
