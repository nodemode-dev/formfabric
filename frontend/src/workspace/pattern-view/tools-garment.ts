import { bezAt, dist, nearestOnSeg, pointInPoly } from '../geom';
import { loadPrefs } from '../../ui/prefs';
import { fmtLen } from '../state';
import type { CornerStyle, NotchKind, Vec } from '../types';
import type { Colors, Ptr } from './ctx';
import { notch, worldLabel } from './render';
import { BaseTool, constrain, moved, normalAt, panelOrSegHit, segHit, strokeSeg } from './tools-common';

const NOTCH_KEY = 'formfabric.notch.kind';
const KINDS: NotchKind[] = ['slit', 'v', 't', 'u'];
const STEP = 0.5;

function storedKind(): NotchKind {
  const v = localStorage.getItem(NOTCH_KEY);
  return KINDS.includes(v as NotchKind) ? (v as NotchKind) : 'v';
}

export class SeamTool extends BaseTool {
  id = 'seam';
  label = 'Seam Allowance';
  key = 'W';
  icon = 'tSeam';
  private target: { panel: number; seg: number | null } | null = null;
  private start: Ptr | null = null;
  private live = false;
  private width = 0;
  private at: Vec = [0, 0];

  private argsFor(width: number): Record<string, unknown> | null {
    const t = this.target;
    const panel = t ? this.ctx.doc.panel(t.panel) : undefined;
    if (!t || !panel) return null;
    const sa = panel.seam_allowance;
    const args: Record<string, unknown> = { panels: [t.panel] };
    if (sa) args.corners = sa.corners;
    if (t.seg === null) {
      args.width = width;
      if (sa && sa.per_segment.length) args.per_segment = sa.per_segment.map(() => null);
    } else {
      const per: Array<number | null> = panel.outline.map((_, i) => sa?.per_segment[i] ?? null);
      per[t.seg] = width;
      args.width = sa?.width ?? 0;
      args.per_segment = per;
    }
    return args;
  }

  private distance(w: Vec): number {
    const t = this.target;
    const geo = t ? this.ctx.doc.geos().find((x) => x.panel.id === t.panel) : undefined;
    if (!geo || pointInPoly(w, geo.poly)) return 0;
    let d = Infinity;
    for (const s of geo.outline) d = Math.min(d, nearestOnSeg(s, w).d);
    return Math.min(500, Math.round(d / STEP) * STEP);
  }

  down(p: Ptr): void {
    const hit = panelOrSegHit(this.ctx, p);
    if (!hit) return;
    this.target = { panel: hit.panel, seg: hit.kind === 'segment' ? hit.index : null };
    this.start = p;
    this.live = false;
    this.ctx.sel.only(hit.panel);
    this.ctx.selected();
  }

  move(p: Ptr): void {
    const t = this.target;
    if (!t || !this.start) {
      const hit = panelOrSegHit(this.ctx, p);
      this.ctx.setHover(hit);
      this.ctx.cursor(hit ? 'pointer' : 'default');
      return;
    }
    if (!this.live) {
      if (!moved(this.start, p)) return;
      this.live = true;
      this.ctx.doc.beginTrial();
    }
    this.width = this.distance(p.w);
    this.at = p.w;
    const args = this.argsFor(this.width);
    const err = args ? this.ctx.doc.trial('pattern.seam_allowance', args) : 'unavailable';
    if (err) this.ctx.say(err);
    this.ctx.redraw();
  }

  up(_p: Ptr): void {
    const t = this.target;
    const live = this.live;
    const args = live ? this.argsFor(this.width) : null;
    this.target = null;
    this.start = null;
    this.live = false;
    if (!t) return;
    if (live) {
      if (args && this.width > 0) void this.ctx.doc.commitTrial([['pattern.seam_allowance', args]], 'Seam Allowance');
      else void this.ctx.doc.cancelTrial();
      return;
    }
    this.target = t;
    this.exact();
    this.target = null;
  }

  keydown(e: KeyboardEvent): boolean {
    if (e.key === 'Enter') { this.exact(); return true; }
    if (e.key === 'Escape') { this.cancel(); return true; }
    return false;
  }

  exact(): void {
    const panelId = this.target?.panel ?? this.firstPanel();
    const panel = panelId === null ? undefined : this.ctx.doc.panel(panelId);
    if (!panel) { this.ctx.say('Select a panel first'); return; }
    const sa = panel.seam_allowance;
    const seg = this.target?.seg ?? null;
    const base = loadPrefs().seamAllowance;
    const init: Record<string, unknown> = { panels: [panel.id], width: sa?.width ?? base, corners: (sa?.corners ?? 'miter') as CornerStyle };
    if (seg !== null) init.per_segment = panel.outline.map((_, i) => (i === seg ? (sa?.per_segment[i] ?? sa?.width ?? base) : (sa?.per_segment[i] ?? null)));
    void this.ask('pattern.seam_allowance', init);
  }

  cancel(): void {
    if (this.live) void this.ctx.doc.cancelTrial();
    this.target = null;
    this.start = null;
    this.live = false;
    this.ctx.redraw();
  }

  busy(): boolean { return this.target !== null; }

  leave(): void { this.cancel(); }

  draw(g: CanvasRenderingContext2D, c: Colors): void {
    if (!this.live) return;
    const s = this.ctx.toScreen(this.at);
    worldLabel(g, fmtLen(this.width, this.ctx.unit()), s[0] + 12, s[1] - 10, c);
  }
}

export class NotchTool extends BaseTool {
  id = 'notch';
  label = 'Notch';
  key = 'K';
  icon = 'tNotch';
  private kind: NotchKind = storedKind();
  private cur: { panel: number; seg: number; t: number; pt: Vec; n: Vec } | null = null;

  private locate(p: Ptr): void {
    const hit = segHit(this.ctx, p, 10);
    const geo = hit ? this.ctx.doc.geos().find((x) => x.panel.id === hit.panel) : undefined;
    const seg = geo && hit ? geo.outline[hit.index] : undefined;
    if (!hit || hit.t === undefined || !geo || !seg) { this.cur = null; this.ctx.setHover(null); return; }
    let t = hit.t;
    const mid = bezAt(seg, 0.5);
    if (dist(mid, p.w) <= this.ctx.tol(8)) t = 0.5;
    this.cur = { panel: hit.panel, seg: hit.index, t, pt: t === 0.5 ? mid : (hit.pt ?? p.w), n: normalAt(geo, hit.index, t) };
    this.ctx.setHover(hit);
  }

  move(p: Ptr): void {
    this.locate(p);
    this.ctx.cursor(this.cur ? 'copy' : 'default');
    this.ctx.redraw();
  }

  down(p: Ptr): void { this.locate(p); }

  up(_p: Ptr): void {
    const c = this.cur;
    if (!c) return;
    this.cur = null;
    this.ctx.sel.only(c.panel);
    this.ctx.selected();
    void this.ctx.doc.run('pattern.notch_add', { panel: c.panel, segment: c.seg, t: c.t, kind: this.kind });
  }

  keydown(e: KeyboardEvent): boolean {
    if (e.key === 'Enter') { this.exact(); return true; }
    if (e.key === 'Escape') { this.cur = null; this.ctx.redraw(); return true; }
    const at = Number(e.key) - 1;
    if (!e.shiftKey && at >= 0 && at < KINDS.length) {
      this.kind = KINDS[at];
      localStorage.setItem(NOTCH_KEY, this.kind);
      this.ctx.redraw();
      return true;
    }
    return false;
  }

  exact(): void {
    const c = this.cur;
    const panel = c?.panel ?? this.firstPanel();
    if (panel === null) { this.ctx.say('Select a panel first'); return; }
    const init: Record<string, unknown> = { panel, kind: this.kind };
    if (c) { init.segment = c.seg; init.t = c.t; } else if (this.ctx.sel.seg && this.ctx.sel.seg.panel === panel) init.segment = this.ctx.sel.seg.i;
    void this.ctx.exact('pattern.notch_add', init).then((v) => {
      if (!v) return;
      if (typeof v.kind === 'string' && KINDS.includes(v.kind as NotchKind)) { this.kind = v.kind as NotchKind; localStorage.setItem(NOTCH_KEY, this.kind); }
      void this.ctx.doc.run('pattern.notch_add', v);
    });
  }

  cancel(): void { this.cur = null; this.ctx.redraw(); }

  leave(): void { this.cur = null; }

  draw(g: CanvasRenderingContext2D, c: Colors): void {
    const cur = this.cur;
    if (!cur) return;
    notch(g, { id: 0, kind: this.kind, pt: cur.pt, n: cur.n, depth: 6, width: 3 }, (x, y) => this.ctx.toScreen([x, y]), this.ctx.cam.s, c.accent, 1);
    const s = this.ctx.toScreen(cur.pt);
    g.save();
    g.fillStyle = c.accent;
    g.beginPath();
    g.arc(s[0], s[1], 3, 0, Math.PI * 2);
    g.fill();
    g.restore();
  }
}

export class GrainTool extends BaseTool {
  id = 'grain';
  label = 'Grain Line';
  key = 'G';
  icon = 'tGrain';
  private panel: number | null = null;
  private from: Vec = [0, 0];
  private to: Vec = [0, 0];
  private start: Ptr | null = null;
  private live = false;

  down(p: Ptr): void {
    const hit = panelOrSegHit(this.ctx, p, 4);
    if (!hit) return;
    this.panel = hit.panel;
    this.from = this.ctx.snap(p.w, []).pt;
    this.to = this.from;
    this.start = p;
    this.live = false;
    this.ctx.sel.only(hit.panel);
    this.ctx.selected();
  }

  move(p: Ptr): void {
    if (this.panel === null || !this.start) {
      const hit = panelOrSegHit(this.ctx, p, 4);
      this.ctx.setHover(hit ? { ...hit, kind: 'panel', index: -1 } : null);
      this.ctx.cursor(hit ? 'crosshair' : 'default');
      return;
    }
    if (!this.live) {
      if (!moved(this.start, p)) return;
      this.live = true;
      this.ctx.doc.beginTrial();
    }
    this.to = constrain(this.from, p.shift || p.mod ? p.w : this.ctx.snap(p.w).pt, p);
    if (dist(this.from, this.to) > 0.5) {
      const err = this.ctx.doc.trial('pattern.grain_line', { panel: this.panel, from: this.from, to: this.to, space: 'world' });
      if (err) this.ctx.say(err);
    }
    this.ctx.redraw();
  }

  up(_p: Ptr): void {
    const panel = this.panel;
    const live = this.live;
    this.panel = null;
    this.start = null;
    this.live = false;
    this.ctx.setSnapMark(null);
    if (panel === null) return;
    if (live && dist(this.from, this.to) > 0.5) void this.ctx.doc.commitTrial([['pattern.grain_line', { panel, from: this.from, to: this.to, space: 'world' }]], 'Grain Line');
    else if (live) void this.ctx.doc.cancelTrial();
    else { this.panel = panel; this.exact(); this.panel = null; }
  }

  keydown(e: KeyboardEvent): boolean {
    if (e.key === 'Enter') { this.exact(); return true; }
    if (e.key === 'Escape') { this.cancel(); return true; }
    return false;
  }

  exact(): void {
    const panel = this.panel ?? this.firstPanel();
    if (panel === null) { this.ctx.say('Select a panel first'); return; }
    void this.ask('pattern.grain_line', { panel });
  }

  cancel(): void {
    if (this.live) void this.ctx.doc.cancelTrial();
    this.panel = null;
    this.start = null;
    this.live = false;
    this.ctx.setSnapMark(null);
    this.ctx.redraw();
  }

  busy(): boolean { return this.panel !== null; }

  leave(): void { this.cancel(); }

  draw(g: CanvasRenderingContext2D, c: Colors): void {
    if (!this.live) return;
    const a = this.ctx.toScreen(this.from);
    const b = this.ctx.toScreen(this.to);
    const deg = (Math.atan2(this.to[1] - this.from[1], this.to[0] - this.from[0]) * 180) / Math.PI;
    worldLabel(g, `${deg.toFixed(1)}°  ${fmtLen(dist(this.from, this.to), this.ctx.unit())}`, b[0] + 12, b[1] - 10, c);
    g.save();
    g.fillStyle = c.accent;
    g.beginPath();
    g.arc(a[0], a[1], 3, 0, Math.PI * 2);
    g.fill();
    g.restore();
  }
}

export class MatchTool extends BaseTool {
  id = 'match';
  label = 'Match Lengths';
  key = 'H';
  icon = 'tMatch';
  private target: { panel: number; seg: number } | null = null;
  private ref: { panel: number; seg: number } | null = null;
  private trialOn = false;
  private how = 'move_end';

  private args(): Record<string, unknown> | null {
    const t = this.target;
    const r = this.ref;
    if (!t || !r) return null;
    return { panel: t.panel, segments: [t.seg], reference_panel: r.panel, reference_segments: [r.seg], how: this.how };
  }

  private same(a: { panel: number; seg: number } | null, b: { panel: number; seg: number }): boolean {
    return !!a && a.panel === b.panel && a.seg === b.seg;
  }

  private closeTrial(): void {
    if (!this.trialOn) return;
    this.trialOn = false;
    void this.ctx.doc.cancelTrial();
  }

  down(p: Ptr): void {
    const hit = segHit(this.ctx, p);
    if (!hit) return;
    const at = { panel: hit.panel, seg: hit.index };
    if (!this.target) {
      this.target = at;
      this.ctx.sel.only(hit.panel);
      this.ctx.sel.seg = { panel: hit.panel, i: hit.index };
      this.ctx.selected();
      this.ctx.say('Pick the reference segment');
      this.ctx.redraw();
      return;
    }
    if (this.same(this.target, at)) return;
    this.ref = at;
    this.how = p.shift ? 'scale' : 'move_end';
    const args = this.args();
    if (args && this.trialOn) void this.ctx.doc.commitTrial([['pattern.match_length', args]], 'Match Lengths');
    else if (args) void this.ctx.doc.run('pattern.match_length', args);
    this.trialOn = false;
    this.target = null;
    this.ref = null;
    this.ctx.say('');
    this.ctx.redraw();
  }

  move(p: Ptr): void {
    const hit = segHit(this.ctx, p);
    this.ctx.setHover(hit);
    this.ctx.cursor(hit ? 'pointer' : 'default');
    if (!this.target) return;
    const at = hit ? { panel: hit.panel, seg: hit.index } : null;
    if (!at || this.same(this.target, at)) {
      if (this.ref) { this.ref = null; this.closeTrial(); }
      this.ctx.redraw();
      return;
    }
    if (this.same(this.ref, at)) return;
    this.ref = at;
    this.how = p.shift ? 'scale' : 'move_end';
    if (!this.trialOn) { this.ctx.doc.beginTrial(); this.trialOn = true; }
    const args = this.args();
    const err = args ? this.ctx.doc.trial('pattern.match_length', args) : null;
    if (err) this.ctx.say(err);
    this.ctx.redraw();
  }

  keydown(e: KeyboardEvent): boolean {
    if (e.key === 'Enter') { this.exact(); return true; }
    if (e.key === 'Escape') { this.cancel(); return true; }
    return false;
  }

  exact(): void {
    const t = this.target ?? (this.ctx.sel.seg ? { panel: this.ctx.sel.seg.panel, seg: this.ctx.sel.seg.i } : null);
    if (!t) {
      const panel = this.firstPanel();
      if (panel === null) { this.ctx.say('Select a segment first'); return; }
      void this.ask('pattern.match_length', { panel });
      return;
    }
    const init: Record<string, unknown> = { panel: t.panel, segments: [t.seg] };
    if (this.ref) { init.reference_panel = this.ref.panel; init.reference_segments = [this.ref.seg]; }
    this.closeTrial();
    this.target = null;
    this.ref = null;
    void this.ask('pattern.match_length', init);
  }

  cancel(): void {
    this.closeTrial();
    this.target = null;
    this.ref = null;
    this.ctx.say('');
    this.ctx.redraw();
  }

  busy(): boolean { return this.target !== null; }

  leave(): void { this.cancel(); }

  draw(g: CanvasRenderingContext2D, c: Colors): void {
    if (this.target) strokeSeg(this.ctx, g, c, this.target.panel, this.target.seg, 3.5, 0.95);
  }
}
