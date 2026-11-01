import { add, dist, hitInner, len, sub } from '../geom';
import { fmtLen } from '../state';
import type { Vec } from '../types';
import type { Colors, Ptr } from './ctx';
import { pathSegs, worldLabel } from './render';
import { BaseTool, constrain, firstAdded, moved, panelOrSegHit, segHit, strokeSeg } from './tools-common';

abstract class TrialTool extends BaseTool {
  protected abstract command: string;
  protected abstract step: string;
  protected creates = false;
  protected start: Ptr | null = null;
  protected live = false;
  protected last: Record<string, unknown> | null = null;

  protected abstract pick(p: Ptr): boolean;
  protected abstract compute(p: Ptr): Record<string, unknown> | null;
  protected abstract clicked(): void;
  protected hover(_p: Ptr): void { return; }
  protected reset(): void { return; }

  down(p: Ptr): void {
    if (!this.pick(p)) return;
    this.start = p;
  }

  move(p: Ptr): void {
    if (!this.start) { this.hover(p); return; }
    if (!this.live) {
      if (!moved(this.start, p)) return;
      this.live = true;
      this.ctx.doc.beginTrial();
    }
    const args = this.compute(p);
    if (args) {
      this.last = args;
      const err = this.ctx.doc.trial(this.command, args);
      this.ctx.say(err ?? '');
    }
    this.ctx.redraw();
  }

  up(_p: Ptr): void {
    const live = this.live;
    const args = this.last;
    const had = this.start !== null;
    this.start = null;
    this.live = false;
    this.last = null;
    this.ctx.setSnapMark(null);
    if (!had) return;
    if (live && args) {
      const before = new Set(this.ctx.doc.garment.panels.map((p) => p.id));
      const done = this.ctx.doc.commitTrial([[this.command, args]], this.step);
      if (this.creates) void done.then(() => firstAdded(this.ctx, before));
      this.ctx.say('');
    } else if (live) {
      void this.ctx.doc.cancelTrial();
    } else this.clicked();
    this.reset();
    this.ctx.redraw();
  }

  keydown(e: KeyboardEvent): boolean {
    if (e.key === 'Enter') { this.exact(); return true; }
    if (e.key === 'Escape') { this.cancel(); return true; }
    return false;
  }

  cancel(): void {
    if (this.live) void this.ctx.doc.cancelTrial();
    this.start = null;
    this.live = false;
    this.last = null;
    this.reset();
    this.ctx.setSnapMark(null);
    this.ctx.say('');
    this.ctx.redraw();
  }

  busy(): boolean { return this.start !== null; }

  leave(): void { this.cancel(); }
}

export class DartTool extends TrialTool {
  id = 'dart';
  label = 'Dart';
  key = 'D';
  icon = 'tDart';
  protected command = 'pattern.dart';
  protected step = 'Dart';
  private width = Number(localStorage.getItem('formfabric.dart.width')) || 20;
  private at: { panel: number; segment: number; t: number; pt: Vec } | null = null;
  private apex: Vec | null = null;

  protected pick(p: Ptr): boolean {
    const hit = segHit(this.ctx, p);
    if (!hit || hit.t === undefined) { this.ctx.say('Select a segment first'); return false; }
    this.at = { panel: hit.panel, segment: hit.index, t: hit.t, pt: hit.pt ?? p.w };
    this.ctx.sel.only(hit.panel);
    this.ctx.selected();
    return true;
  }

  protected compute(p: Ptr): Record<string, unknown> | null {
    const a = this.at;
    if (!a) return null;
    this.apex = p.w;
    return { panel: a.panel, segment: a.segment, t: a.t, width: this.width, apex: p.w, space: 'world', internal: p.alt };
  }

  protected clicked(): void { this.exact(); }

  protected reset(): void { this.at = null; this.apex = null; }

  protected hover(p: Ptr): void {
    const hit = segHit(this.ctx, p);
    this.ctx.setHover(hit);
    this.ctx.cursor(hit ? 'crosshair' : 'default');
  }

  exact(): void {
    const a = this.at;
    const panel = a?.panel ?? this.firstPanel();
    if (panel === null) { this.ctx.say('Select a panel first'); return; }
    const init: Record<string, unknown> = { panel, width: this.width };
    if (a) { init.segment = a.segment; init.t = a.t; }
    void this.ask('pattern.dart', init).then((v) => {
      if (v && typeof v.width === 'number') { this.width = v.width; localStorage.setItem('formfabric.dart.width', String(v.width)); }
    });
  }

  draw(g: CanvasRenderingContext2D, c: Colors): void {
    const a = this.at;
    if (!a || !this.live || !this.apex) return;
    const s = this.ctx.toScreen(this.apex);
    const b = this.ctx.toScreen(a.pt);
    g.save();
    g.strokeStyle = c.accent;
    g.lineWidth = 1;
    g.setLineDash([4, 3]);
    g.beginPath();
    g.moveTo(b[0], b[1]);
    g.lineTo(s[0], s[1]);
    g.stroke();
    g.restore();
    worldLabel(g, `${fmtLen(this.width, this.ctx.unit(), false)} x ${fmtLen(dist(a.pt, this.apex), this.ctx.unit())}`, s[0] + 12, s[1] - 10, c);
  }
}

export class SlashTool extends BaseTool {
  id = 'slash';
  label = 'Slash and Spread';
  key = 'X';
  icon = 'tSlash';
  private panel: number | null = null;
  private from: Vec = [0, 0];
  private to: Vec = [0, 0];
  private phase: 'line' | 'spread' = 'line';
  private start: Ptr | null = null;
  private live = false;
  private angle = 0;
  private side = 'left';
  private drawing = false;

  private args(): Record<string, unknown> {
    return { panel: this.panel, from: this.from, to: this.to, angle: Math.abs(this.angle), side: this.side, space: 'world' };
  }

  down(p: Ptr): void {
    this.start = p;
    if (this.phase === 'line') {
      const hit = panelOrSegHit(this.ctx, p, 6);
      if (!hit) { this.start = null; this.ctx.say('Start inside a panel'); return; }
      this.panel = hit.panel;
      this.from = this.ctx.snap(p.w).pt;
      this.to = this.from;
      this.drawing = true;
      this.ctx.sel.only(hit.panel);
      this.ctx.selected();
    }
  }

  private spreadFor(w: Vec): void {
    const d = sub(this.to, this.from);
    const v = sub(w, this.from);
    const ang = (Math.atan2(d[0] * v[1] - d[1] * v[0], d[0] * v[0] + d[1] * v[1]) * 180) / Math.PI;
    this.angle = Math.min(170, Math.abs(ang));
    this.side = ang >= 0 ? 'left' : 'right';
  }

  move(p: Ptr): void {
    const s = this.start;
    if (!s) {
      this.ctx.cursor(this.phase === 'line' ? 'crosshair' : 'ew-resize');
      if (this.phase === 'line') this.ctx.snap(p.w);
      this.ctx.redraw();
      return;
    }
    if (this.phase === 'line') {
      if (!this.drawing) return;
      this.to = constrain(this.from, p.shift || p.mod ? p.w : this.ctx.snap(p.w).pt, p);
      this.ctx.redraw();
      return;
    }
    if (!this.live) {
      if (!moved(s, p)) return;
      this.live = true;
      this.ctx.doc.beginTrial();
    }
    this.spreadFor(p.w);
    const err = this.ctx.doc.trial('pattern.slash_spread', this.args());
    this.ctx.say(err ?? '');
    this.ctx.redraw();
  }

  up(p: Ptr): void {
    const s = this.start;
    this.start = null;
    this.ctx.setSnapMark(null);
    if (!s) return;
    if (this.phase === 'line') {
      this.drawing = false;
      if (moved(s, p) && dist(this.from, this.to) > this.ctx.tol(4)) {
        this.phase = 'spread';
        this.angle = 0;
        this.ctx.say('Drag to spread');
      } else { this.exact(); }
      this.ctx.redraw();
      return;
    }
    if (this.live) {
      this.live = false;
      void this.ctx.doc.commitTrial([['pattern.slash_spread', this.args()]], 'Slash and Spread');
      this.phase = 'line';
      this.panel = null;
      this.ctx.say('');
    } else this.exact();
    this.ctx.redraw();
  }

  keydown(e: KeyboardEvent): boolean {
    if (e.key === 'Enter') { this.exact(); return true; }
    if (e.key === 'Escape') { this.cancel(); return true; }
    return false;
  }

  exact(): void {
    const panel = this.panel ?? this.firstPanel();
    if (panel === null) { this.ctx.say('Select a panel first'); return; }
    const init: Record<string, unknown> = { panel, space: 'world' };
    if (this.phase === 'spread') { init.from = this.from; init.to = this.to; }
    this.cancel();
    void this.ask('pattern.slash_spread', init);
  }

  cancel(): void {
    if (this.live) void this.ctx.doc.cancelTrial();
    this.live = false;
    this.start = null;
    this.drawing = false;
    this.phase = 'line';
    this.panel = null;
    this.ctx.setSnapMark(null);
    this.ctx.say('');
    this.ctx.redraw();
  }

  busy(): boolean { return this.phase === 'spread' || this.start !== null; }

  leave(): void { this.cancel(); }

  draw(g: CanvasRenderingContext2D, c: Colors): void {
    if (this.panel === null) return;
    const a = this.ctx.toScreen(this.from);
    const b = this.ctx.toScreen(this.to);
    const d = sub(this.to, this.from);
    const l = len(d);
    g.save();
    g.strokeStyle = c.accent;
    g.lineWidth = 1.5;
    g.setLineDash([8, 4]);
    g.beginPath();
    g.moveTo(a[0], a[1]);
    g.lineTo(b[0], b[1]);
    g.stroke();
    g.setLineDash([]);
    g.fillStyle = c.accent;
    g.beginPath();
    g.arc(a[0], a[1], 4, 0, Math.PI * 2);
    g.fill();
    g.restore();
    if (this.phase === 'line' && l > 0) worldLabel(g, fmtLen(l, this.ctx.unit()), b[0] + 12, b[1] - 10, c);
    else if (this.live) worldLabel(g, `${this.angle.toFixed(1)}°`, b[0] + 12, b[1] - 10, c);
  }
}

export class MirrorTool extends TrialTool {
  id = 'mirror';
  label = 'Mirror Edit';
  key = 'Y';
  icon = 'tMirror';
  protected command = 'pattern.mirror_link';
  protected step = 'Mirror Edit';
  protected creates = true;
  private panel: number | null = null;
  private seg: number | null = null;
  private from: Vec = [0, 0];
  private to: Vec = [0, 0];

  protected pick(p: Ptr): boolean {
    const hit = panelOrSegHit(this.ctx, p, 8);
    if (!hit) { this.ctx.say('Start inside a panel'); return false; }
    this.panel = hit.panel;
    this.seg = hit.kind === 'segment' ? hit.index : null;
    this.from = this.ctx.snap(p.w).pt;
    this.to = this.from;
    this.ctx.sel.only(hit.panel);
    this.ctx.selected();
    if (this.seg !== null) {
      this.live = true;
      this.ctx.doc.beginTrial();
      this.last = { panel: this.panel, axis_segment: this.seg };
      const err = this.ctx.doc.trial(this.command, this.last);
      this.ctx.say(err ?? '');
    }
    return true;
  }

  protected compute(p: Ptr): Record<string, unknown> | null {
    if (this.panel === null) return null;
    if (this.seg !== null && !moved(this.start ?? p, p)) return { panel: this.panel, axis_segment: this.seg };
    this.seg = null;
    this.to = constrain(this.from, p.shift || p.mod ? p.w : this.ctx.snap(p.w).pt, p);
    if (dist(this.from, this.to) < 1) return null;
    return { panel: this.panel, axis: [this.from, this.to] };
  }

  protected clicked(): void {
    if (this.panel !== null && this.seg === null) this.exact();
  }

  protected reset(): void { this.panel = null; this.seg = null; }

  protected hover(p: Ptr): void {
    const hit = panelOrSegHit(this.ctx, p, 8);
    this.ctx.setHover(hit && hit.kind === 'segment' ? hit : null);
    this.ctx.cursor(hit ? 'crosshair' : 'default');
  }

  exact(): void {
    const panel = this.panel ?? this.firstPanel();
    if (panel === null) { this.ctx.say('Select a panel first'); return; }
    const init: Record<string, unknown> = { panel };
    if (this.seg !== null) init.axis_segment = this.seg;
    void this.ask('pattern.mirror_link', init);
  }

  draw(g: CanvasRenderingContext2D, c: Colors): void {
    if (!this.live || this.seg !== null) return;
    const a = this.ctx.toScreen(this.from);
    const b = this.ctx.toScreen(this.to);
    g.save();
    g.strokeStyle = c.accent;
    g.lineWidth = 1.5;
    g.setLineDash([8, 4]);
    g.beginPath();
    g.moveTo(a[0], a[1]);
    g.lineTo(b[0], b[1]);
    g.stroke();
    g.restore();
  }
}

export class InstanceTool extends TrialTool {
  id = 'instance';
  label = 'Instances';
  key = 'J';
  icon = 'tInstance';
  protected command = 'pattern.instance';
  protected step = 'Instances';
  protected creates = true;
  private count = 2;
  private panel: number | null = null;
  private origin: Vec = [0, 0];

  protected pick(p: Ptr): boolean {
    const hit = panelOrSegHit(this.ctx, p, 4);
    if (!hit) { this.ctx.say('Start inside a panel'); return false; }
    this.panel = hit.panel;
    this.origin = p.w;
    this.ctx.sel.only(hit.panel);
    this.ctx.selected();
    return true;
  }

  protected compute(p: Ptr): Record<string, unknown> | null {
    if (this.panel === null) return null;
    const to = constrain(this.origin, p.w, p);
    const interval = sub(to, this.origin);
    if (len(interval) < 1) return null;
    return { panel: this.panel, count: this.count, interval };
  }

  protected clicked(): void { this.exact(); }

  protected reset(): void { this.panel = null; }

  protected hover(p: Ptr): void {
    const hit = panelOrSegHit(this.ctx, p, 4);
    this.ctx.setHover(hit ? { ...hit, kind: 'panel', index: -1 } : null);
    this.ctx.cursor(hit ? 'copy' : 'default');
  }

  keydown(e: KeyboardEvent): boolean {
    const n = Number(e.key);
    if (!e.shiftKey && Number.isInteger(n) && n >= 1 && n <= 9) { this.count = n; return true; }
    return super.keydown(e);
  }

  exact(): void {
    const panel = this.panel ?? this.firstPanel();
    if (panel === null) { this.ctx.say('Select a panel first'); return; }
    const geo = this.ctx.doc.geos().find((x) => x.panel.id === panel);
    const w = geo ? geo.bounds[2] - geo.bounds[0] : 100;
    void this.ask('pattern.instance', { panel, count: this.count, interval: [Math.round(w + 20), 0] }).then((v) => {
      if (v && typeof v.count === 'number') this.count = v.count;
    });
  }

  draw(g: CanvasRenderingContext2D, c: Colors): void {
    if (!this.live || !this.last) return;
    const iv = this.last.interval as Vec;
    const s = this.ctx.toScreen(add(this.origin, iv));
    worldLabel(g, `${this.count} x ${fmtLen(len(iv), this.ctx.unit())}`, s[0] + 12, s[1] - 10, c);
  }
}

export class UnfoldTool extends BaseTool {
  id = 'unfold';
  label = 'Unfold';
  key = 'O';
  icon = 'tUnfold';
  private cur: { panel: number; seg: number } | null = null;
  private trialOn = false;

  private straight(p: Ptr): { panel: number; seg: number } | null {
    const hit = segHit(this.ctx, p);
    if (!hit) return null;
    const seg = this.ctx.doc.geos().find((x) => x.panel.id === hit.panel)?.outline[hit.index];
    return seg && !seg.curved ? { panel: hit.panel, seg: hit.index } : null;
  }

  move(p: Ptr): void {
    if (this.trialOn) return;
    const hit = segHit(this.ctx, p);
    const ok = this.straight(p);
    this.cur = ok;
    this.ctx.setHover(ok ? hit : null);
    this.ctx.cursor(ok ? 'pointer' : hit ? 'not-allowed' : 'default');
    this.ctx.redraw();
  }

  down(p: Ptr): void {
    const ok = this.straight(p);
    if (!ok) { if (segHit(this.ctx, p)) this.ctx.say('Pick a straight segment'); return; }
    this.cur = ok;
    this.trialOn = true;
    this.ctx.doc.beginTrial();
    const err = this.ctx.doc.trial('pattern.unfold', { panel: ok.panel, segment: ok.seg });
    if (err) { this.ctx.say(err); }
  }

  up(p: Ptr): void {
    if (!this.trialOn) return;
    this.trialOn = false;
    const c = this.cur;
    this.cur = null;
    const still = this.ctx.doc.panel(c?.panel ?? -1);
    void p;
    if (!c || !still) { void this.ctx.doc.cancelTrial(); return; }
    void this.ctx.doc.commitTrial([['pattern.unfold', { panel: c.panel, segment: c.seg }]], 'Unfold').then(() => { this.ctx.say(''); });
  }

  keydown(e: KeyboardEvent): boolean {
    if (e.key === 'Enter') { this.exact(); return true; }
    if (e.key === 'Escape') { this.cancel(); return true; }
    return false;
  }

  exact(): void {
    const panel = this.firstPanel();
    if (panel === null) { this.ctx.say('Select a panel first'); return; }
    const init: Record<string, unknown> = { panel };
    if (this.ctx.sel.seg && this.ctx.sel.seg.panel === panel) init.segment = this.ctx.sel.seg.i;
    void this.ask('pattern.unfold', init);
  }

  cancel(): void {
    if (this.trialOn) void this.ctx.doc.cancelTrial();
    this.trialOn = false;
    this.cur = null;
    this.ctx.redraw();
  }

  busy(): boolean { return this.trialOn; }

  leave(): void { this.cancel(); }

  draw(g: CanvasRenderingContext2D, c: Colors): void {
    if (this.cur && !this.trialOn) strokeSeg(this.ctx, g, c, this.cur.panel, this.cur.seg, 3.5, 0.9);
  }
}

export class SpiralTool extends TrialTool {
  id = 'spiral';
  label = 'Spiral';
  key = 'Q';
  icon = 'tSpiral';
  protected command = 'pattern.spiral';
  protected step = 'Spiral';
  protected creates = true;
  private center: Vec = [0, 0];
  private radius = 0;

  protected pick(p: Ptr): boolean {
    this.center = this.ctx.snap(p.w).pt;
    this.radius = 0;
    return true;
  }

  protected compute(p: Ptr): Record<string, unknown> | null {
    const r = dist(this.center, p.w);
    if (r < 12) return null;
    this.radius = r;
    const inner = Math.max(2, r * 0.2);
    const pitch = (r - inner) / 3;
    return { center: this.center, inner_radius: inner, outer_radius: r, width: Math.max(0.5, pitch * 0.6), turns: 3 };
  }

  protected clicked(): void { void this.ask('pattern.spiral', { center: this.center }); }

  protected hover(p: Ptr): void {
    this.ctx.snap(p.w);
    this.ctx.cursor('crosshair');
    this.ctx.redraw();
  }

  exact(): void {
    void this.ask('pattern.spiral', {});
  }

  draw(g: CanvasRenderingContext2D, c: Colors): void {
    if (!this.live || this.radius === 0) return;
    const o = this.ctx.toScreen(this.center);
    g.save();
    g.strokeStyle = c.accent;
    g.lineWidth = 1;
    g.setLineDash([3, 3]);
    g.beginPath();
    g.arc(o[0], o[1], this.radius * this.ctx.cam.s, 0, Math.PI * 2);
    g.stroke();
    g.restore();
    worldLabel(g, fmtLen(this.radius, this.ctx.unit()), o[0] + this.radius * this.ctx.cam.s + 8, o[1] - 6, c);
  }
}

export class TraceTool extends BaseTool {
  id = 'trace';
  label = 'Trace';
  key = 'T';
  icon = 'tTrace';
  private cur: { panel: number; inner: number } | null = null;

  private locate(p: Ptr): void {
    const hit = hitInner(this.ctx.doc.geos(), p.w, this.ctx.tol(8), true);
    this.cur = hit ? { panel: hit.panel, inner: hit.inner } : null;
  }

  move(p: Ptr): void {
    this.locate(p);
    this.ctx.cursor(this.cur ? 'pointer' : 'default');
    this.ctx.redraw();
  }

  down(p: Ptr): void { this.locate(p); }

  up(p: Ptr): void {
    const c = this.cur;
    if (!c) return;
    this.cur = null;
    const before = new Set(this.ctx.doc.garment.panels.map((x) => x.id));
    void this.ctx.doc.run('pattern.trace', { panel: c.panel, inner: c.inner, remove: p.shift }).then(() => firstAdded(this.ctx, before));
  }

  keydown(e: KeyboardEvent): boolean {
    if (e.key === 'Enter') { this.exact(); return true; }
    if (e.key === 'Escape') { this.cur = null; this.ctx.redraw(); return true; }
    return false;
  }

  exact(): void {
    const panel = this.firstPanel();
    const init: Record<string, unknown> = {};
    if (panel !== null) init.panel = panel;
    if (this.ctx.sel.inner) init.inner = this.ctx.sel.inner.id;
    void this.ask('pattern.trace', init);
  }

  cancel(): void { this.cur = null; this.ctx.redraw(); }

  leave(): void { this.cur = null; }

  draw(g: CanvasRenderingContext2D, c: Colors): void {
    const cur = this.cur;
    if (!cur) return;
    const geo = this.ctx.doc.geos().find((x) => x.panel.id === cur.panel);
    const inner = geo?.inner.find((x) => x.id === cur.inner);
    if (!geo || !inner) return;
    g.save();
    g.beginPath();
    pathSegs(g, inner.segs, (x, y) => this.ctx.toScreen([x, y]), inner.closed);
    g.globalAlpha = 0.2;
    g.fillStyle = c.accent;
    if (inner.closed) g.fill();
    g.globalAlpha = 0.95;
    g.lineWidth = 2.5;
    g.strokeStyle = c.accent;
    g.stroke();
    g.restore();
  }
}
