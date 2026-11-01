import { dist, nearestOnSeg, type PanelGeo } from '../geom';
import { fmtLen } from '../state';
import type { Vec } from '../types';
import type { Colors, Ptr, ViewCtx } from './ctx';
import { worldLabel } from './render';
import {
  chainOf, lineSegs, lineSpan, nearestParam, pickLine, pointAt, runsToEnds, seamAt, spansBetween, tackAt, Tracker, walkSign,
  type LinePick, type LineRef,
} from './sew-geom';
import { spanArg, type SeamEnd } from './sew-types';
import { BaseTool, moved } from './tools-common';

interface Pick { panel: number; inner: number | null; segment: number }

const PLEAT_KEY = 'formfabric.pleat';

function geoMap(ctx: ViewCtx): Map<number, PanelGeo> {
  return new Map(ctx.doc.geos().map((g) => [g.panel.id, g]));
}

function fullEnd(p: Pick, dir = 1): SeamEnd {
  return { panel: p.panel, inner: p.inner, segment: p.segment, t0: dir > 0 ? 0 : 1, t1: dir > 0 ? 1 : 0 };
}

function samePick(a: Pick, b: Pick): boolean {
  return a.panel === b.panel && a.inner === b.inner && a.segment === b.segment;
}

function orient(ctx: ViewCtx, picks: Pick[]): SeamEnd[] {
  const geos = geoMap(ctx);
  const near = (a: Pick, b: Pick | undefined, step: number): boolean => {
    if (!b || a.panel !== b.panel || a.inner !== b.inner) return false;
    const n = lineSegs(geos.get(a.panel), a.inner).length;
    return n > 0 && (((a.segment + step) % n) + n) % n === b.segment;
  };
  return picks.map((p, i) => {
    const next = picks[i + 1];
    const prev = picks[i - 1];
    let dir = 1;
    if (next) dir = near(p, next, 1) ? 1 : near(p, next, -1) ? -1 : 1;
    else if (prev) dir = near(prev, p, 1) ? 1 : near(prev, p, -1) ? -1 : 1;
    return fullEnd(p, dir);
  });
}

function strokeEnds(ctx: ViewCtx, g: CanvasRenderingContext2D, ends: SeamEnd[], color: string, width: number, alpha = 0.95): void {
  const ch = chainOf(geoMap(ctx), ends);
  if (ch.pts.length < 2) return;
  g.save();
  g.beginPath();
  ch.pts.forEach((p, i) => {
    const s = ctx.toScreen(p);
    if (i === 0) g.moveTo(s[0], s[1]);
    else g.lineTo(s[0], s[1]);
  });
  g.lineWidth = width;
  g.globalAlpha = alpha;
  g.strokeStyle = color;
  g.lineCap = 'round';
  g.lineJoin = 'round';
  g.stroke();
  g.restore();
}

function dot(ctx: ViewCtx, g: CanvasRenderingContext2D, at: Vec, color: string, r = 3.5): void {
  const s = ctx.toScreen(at);
  g.save();
  g.fillStyle = color;
  g.beginPath();
  g.arc(s[0], s[1], r, 0, Math.PI * 2);
  g.fill();
  g.restore();
}

abstract class SewBase extends BaseTool {
  protected geos(): Map<number, PanelGeo> { return geoMap(this.ctx); }

  protected async submit(command: string, args: Record<string, unknown>): Promise<void> {
    const r = await this.ctx.doc.run<{ id?: number }>(command, args);
    if (r && typeof r.id === 'number') {
      this.ctx.sel.seam = r.id;
      this.ctx.sel.tack = null;
      this.ctx.selected();
    }
    this.ctx.redraw();
  }

  protected pickAt(p: Ptr, px = 8): LinePick | null {
    return pickLine(this.ctx.doc.geos(), p.w, this.ctx.tol(px));
  }

  protected selectSeam(id: number | null): void {
    this.ctx.sel.seam = id;
    this.ctx.sel.tack = null;
    this.ctx.selected();
  }

  keydown(e: KeyboardEvent): boolean {
    if (e.key === 'Enter') { this.enter(); return true; }
    if (e.key === 'Escape') { this.cancel(); return true; }
    return false;
  }

  protected enter(): void { this.exact(); }
}

export class SewTool extends SewBase {
  id = 'sew';
  label = 'Seam';
  key = 'V';
  icon = 'tSew';
  private a: Pick[] = [];
  private b: Pick[] = [];
  private phase: 0 | 1 = 0;
  private hover: LinePick | null = null;

  private list(): Pick[] { return this.phase === 0 ? this.a : this.b; }

  private hint(): void {
    this.ctx.say(this.phase === 0 ? 'Click a segment on the first panel · Shift-click adds segments · Enter continues' : 'Click the matching segment on the other panel · Shift-click adds segments · Enter finishes');
  }

  down(p: Ptr): void {
    const hit = this.pickAt(p);
    if (!hit) return;
    const pk: Pick = { panel: hit.panel, inner: hit.inner, segment: hit.segment };
    if (this.phase === 1 && this.a.some((x) => samePick(x, pk))) return;
    const list = this.list();
    const at = list.findIndex((x) => samePick(x, pk));
    if (at >= 0) list.splice(at, 1);
    else list.push(pk);
    if (p.shift) { this.hint(); this.ctx.redraw(); return; }
    this.advance();
  }

  private advance(): void {
    if (this.phase === 0) {
      if (!this.a.length) return;
      this.phase = 1;
      this.hint();
      this.ctx.redraw();
      return;
    }
    if (!this.b.length) return;
    this.finish();
  }

  private finish(): void {
    const a = orient(this.ctx, this.a);
    const b = orient(this.ctx, this.b);
    const reversed = walkSign(a) === walkSign(b);
    this.reset();
    if (a.length === 1 && b.length === 1) {
      const args: Record<string, unknown> = { panel_a: a[0].panel, segment_a: a[0].segment, panel_b: b[0].panel, segment_b: b[0].segment, reversed };
      if (a[0].inner !== null) args.inner_a = a[0].inner;
      if (b[0].inner !== null) args.inner_b = b[0].inner;
      void this.submit('sew.segment', args);
    } else void this.submit('sew.multi', { a: a.map(spanArg), b: b.map(spanArg), reversed });
  }

  private reset(): void {
    this.a = [];
    this.b = [];
    this.phase = 0;
    this.hover = null;
    this.ctx.say('');
    this.ctx.redraw();
  }

  move(p: Ptr): void {
    const hit = this.pickAt(p);
    const same = (!hit && !this.hover) || (hit && this.hover && hit.panel === this.hover.panel && hit.inner === this.hover.inner && hit.segment === this.hover.segment);
    this.hover = hit;
    this.ctx.cursor(hit ? 'pointer' : 'default');
    if (!same) this.ctx.redraw();
  }

  protected enter(): void {
    if (!this.a.length && !this.b.length) { this.exact(); return; }
    this.advance();
  }

  keydown(e: KeyboardEvent): boolean {
    if ((e.key === 'Delete' || e.key === 'Backspace') && this.busy()) {
      this.list().pop();
      if (this.phase === 1 && !this.b.length) this.phase = 0;
      this.ctx.redraw();
      return true;
    }
    return super.keydown(e);
  }

  exact(): void {
    const init: Record<string, unknown> = {};
    if (this.a[0]) { init.panel_a = this.a[0].panel; init.segment_a = this.a[0].segment; if (this.a[0].inner !== null) init.inner_a = this.a[0].inner; }
    if (this.b[0]) { init.panel_b = this.b[0].panel; init.segment_b = this.b[0].segment; if (this.b[0].inner !== null) init.inner_b = this.b[0].inner; }
    const sel = this.ctx.sel.seg;
    if (!this.a[0] && sel) { init.panel_a = sel.panel; init.segment_a = sel.i; }
    this.reset();
    void this.ctx.exact('sew.segment', init).then((v) => { if (v) void this.submit('sew.segment', v); });
  }

  cancel(): void { this.reset(); }

  busy(): boolean { return this.a.length > 0 || this.b.length > 0; }

  leave(): void { this.reset(); }

  draw(g: CanvasRenderingContext2D, c: Colors): void {
    const h = this.hover;
    if (h && !this.list().some((x) => samePick(x, h))) strokeEnds(this.ctx, g, [fullEnd(h)], c.accent, 3, 0.55);
    const sets: Array<[Pick[], string, string]> = [[this.a, c.accent, 'A'], [this.b, c.text, 'B']];
    for (const [list, color, name] of sets) {
      if (!list.length) continue;
      const ends = orient(this.ctx, list);
      strokeEnds(this.ctx, g, ends, color, 4);
      const ch = chainOf(this.geos(), ends);
      if (ch.pts.length > 1) {
        const s = this.ctx.toScreen(pointAt(ch, 0.5));
        worldLabel(g, name, s[0] + 10, s[1] - 8, c);
      }
    }
  }
}

interface FreeDrag { ref: LineRef; n: number; closed: boolean; p0: number; tr: Tracker; start: Ptr; pick: LinePick; ends: SeamEnd[]; live: boolean }

export class FreeSeamTool extends SewBase {
  id = 'freeseam';
  label = 'Free Seam';
  key = 'Shift+V';
  icon = 'tFreeSeam';
  private sides: [SeamEnd[], SeamEnd[]] = [[], []];
  private phase: 0 | 1 = 0;
  private drag: FreeDrag | null = null;
  private hover: LinePick | null = null;

  private hint(): void {
    this.ctx.say(this.phase === 0 ? 'Drag along a line for the first side · Shift adds spans · Enter continues' : 'Drag along a line for the other side · Shift adds spans · Enter finishes');
  }

  down(p: Ptr): void {
    const pick = this.pickAt(p);
    if (!pick) return;
    const geo = this.geos().get(pick.panel);
    const n = lineSegs(geo, pick.inner).length;
    if (!geo || n === 0) return;
    const closed = pick.inner === null || (geo.inner.find((i) => i.id === pick.inner)?.closed ?? false);
    const p0 = pick.segment + pick.t;
    this.drag = { ref: { panel: pick.panel, inner: pick.inner }, n, closed, p0, tr: new Tracker(p0, n, closed), start: p, pick, ends: [], live: false };
    this.ctx.doc.prime([pick.panel]);
    this.hint();
  }

  move(p: Ptr): void {
    const d = this.drag;
    if (!d) {
      const hit = this.pickAt(p);
      const same = (!hit && !this.hover) || (hit && this.hover && hit.panel === this.hover.panel && hit.inner === this.hover.inner && hit.segment === this.hover.segment);
      this.hover = hit;
      this.ctx.cursor(hit ? 'crosshair' : 'default');
      if (!same) this.ctx.redraw();
      return;
    }
    if (!d.live && !moved(d.start, p)) return;
    d.live = true;
    const at = this.ctx.snap(p.w).pt;
    const near = nearestParam(this.geos().get(d.ref.panel), d.ref.inner, at);
    if (!near) return;
    const p1 = d.tr.feed(near.p);
    d.ends = runsToEnds(d.ref, spansBetween(d.n, d.p0, p1));
    this.ctx.redraw();
  }

  up(p: Ptr): void {
    const d = this.drag;
    this.drag = null;
    this.ctx.setSnapMark(null);
    if (!d) return;
    let ends = d.ends;
    if (!d.live) ends = [fullEnd({ panel: d.pick.panel, inner: d.pick.inner, segment: d.pick.segment })];
    if (!ends.length) { this.ctx.redraw(); return; }
    this.sides[this.phase].push(...ends);
    if (p.shift) { this.hint(); this.ctx.redraw(); return; }
    this.advance();
  }

  private advance(): void {
    if (!this.sides[this.phase].length) return;
    if (this.phase === 0) {
      this.phase = 1;
      this.hint();
      this.ctx.redraw();
      return;
    }
    this.finish();
  }

  protected enter(): void {
    if (!this.sides[0].length && !this.sides[1].length) { this.exact(); return; }
    this.advance();
  }

  private finish(): void {
    const [a, b] = this.sides;
    const reversed = walkSign(a) === walkSign(b);
    this.reset();
    if (a.length === 1 && b.length === 1) {
      const args: Record<string, unknown> = {
        panel_a: a[0].panel, segment_a: a[0].segment, t0_a: a[0].t0, t1_a: a[0].t1,
        panel_b: b[0].panel, segment_b: b[0].segment, t0_b: b[0].t0, t1_b: b[0].t1, reversed,
      };
      if (a[0].inner !== null) args.inner_a = a[0].inner;
      if (b[0].inner !== null) args.inner_b = b[0].inner;
      void this.submit('sew.free', args);
    } else void this.submit('sew.multi', { a: a.map(spanArg), b: b.map(spanArg), reversed });
  }

  private reset(): void {
    this.sides = [[], []];
    this.phase = 0;
    this.drag = null;
    this.hover = null;
    this.ctx.setSnapMark(null);
    this.ctx.say('');
    this.ctx.redraw();
  }

  exact(): void {
    const init: Record<string, unknown> = {};
    const [a, b] = this.sides;
    if (a[0]) Object.assign(init, { panel_a: a[0].panel, segment_a: a[0].segment, t0_a: a[0].t0, t1_a: a[0].t1 }, a[0].inner !== null ? { inner_a: a[0].inner } : {});
    if (b[0]) Object.assign(init, { panel_b: b[0].panel, segment_b: b[0].segment, t0_b: b[0].t0, t1_b: b[0].t1 }, b[0].inner !== null ? { inner_b: b[0].inner } : {});
    this.reset();
    void this.ctx.exact('sew.free', init).then((v) => { if (v) void this.submit('sew.free', v); });
  }

  cancel(): void { this.reset(); }

  busy(): boolean { return this.drag !== null || this.sides[0].length > 0 || this.sides[1].length > 0; }

  leave(): void { this.reset(); }

  draw(g: CanvasRenderingContext2D, c: Colors): void {
    if (this.hover && !this.drag) strokeEnds(this.ctx, g, [fullEnd({ panel: this.hover.panel, inner: this.hover.inner, segment: this.hover.segment })], c.accent, 3, 0.45);
    const sets: Array<[SeamEnd[], string, string]> = [[this.sides[0], c.accent, 'A'], [this.sides[1], c.text, 'B']];
    for (const [ends, color, name] of sets) {
      if (!ends.length) continue;
      strokeEnds(this.ctx, g, ends, color, 4);
      const ch = chainOf(this.geos(), ends);
      if (ch.pts.length > 1) {
        const s = this.ctx.toScreen(pointAt(ch, 0.5));
        worldLabel(g, name, s[0] + 10, s[1] - 8, c);
      }
    }
    const d = this.drag;
    if (d && d.live && d.ends.length) {
      strokeEnds(this.ctx, g, d.ends, c.accent, 5, 0.9);
      const ch = chainOf(this.geos(), d.ends);
      if (ch.pts.length > 1) {
        dot(this.ctx, g, ch.pts[0], c.accent, 4);
        const e = ch.pts[ch.pts.length - 1];
        worldLabel(g, fmtLen(ch.total, this.ctx.unit()), this.ctx.toScreen(e)[0] + 12, this.ctx.toScreen(e)[1] - 10, c);
      }
    }
  }
}

interface EditDrag { side: 'a' | 'b'; which: 'start' | 'end'; seam: number; start: Ptr; tr: Tracker | null; span: ReturnType<typeof lineSpan>; ends: SeamEnd[]; live: boolean }

export class EditSeamTool extends SewBase {
  id = 'editseam';
  label = 'Edit Seam';
  key = 'B';
  icon = 'tEditSeam';
  private hover: number | null = null;
  private drag: EditDrag | null = null;

  private handles(): Array<{ side: 'a' | 'b'; which: 'start' | 'end'; at: Vec }> {
    const id = this.ctx.sel.seam;
    const sg = id === null ? undefined : this.ctx.sew().seams.find((s) => s.seam.id === id);
    if (!sg) return [];
    const out: Array<{ side: 'a' | 'b'; which: 'start' | 'end'; at: Vec }> = [];
    for (const side of ['a', 'b'] as const) {
      const ch = side === 'a' ? sg.a : sg.b;
      if (ch.pts.length < 2) continue;
      out.push({ side, which: 'start', at: ch.pts[0] }, { side, which: 'end', at: ch.pts[ch.pts.length - 1] });
    }
    return out;
  }

  down(p: Ptr): void {
    const id = this.ctx.sel.seam;
    const tol = this.ctx.tol(10);
    const h = this.handles().find((x) => dist(x.at, p.w) <= tol);
    const seam = this.ctx.doc.sew.seams.find((s) => s.id === id);
    if (h && seam && id !== null) {
      const ends = h.side === 'a' ? seam.a : seam.b;
      const span = lineSpan(this.geos(), ends);
      this.drag = { side: h.side, which: h.which, seam: id, start: p, tr: span ? new Tracker(h.which === 'start' ? span.p0 : span.p1, span.n, span.closed) : null, span, ends, live: false };
      this.ctx.say('Drag an end handle');
      return;
    }
    const hit = seamAt(this.ctx.sew(), p.w, this.ctx.tol(8));
    this.selectSeam(hit ? hit.seam.seam.id : null);
    this.ctx.redraw();
  }

  move(p: Ptr): void {
    const d = this.drag;
    if (!d) {
      const tol = this.ctx.tol(10);
      const onHandle = this.handles().some((x) => dist(x.at, p.w) <= tol);
      const hit = seamAt(this.ctx.sew(), p.w, this.ctx.tol(8));
      const next = hit ? hit.seam.seam.id : null;
      this.ctx.cursor(onHandle ? 'grab' : hit ? 'pointer' : 'default');
      if (next !== this.hover) { this.hover = next; this.ctx.redraw(); }
      return;
    }
    if (!d.live && !moved(d.start, p)) return;
    d.live = true;
    const at = this.ctx.snap(p.w).pt;
    const geos = this.geos();
    const base = this.seamEnds(d.seam, d.side);
    if (d.span && d.tr) {
      const near = nearestParam(geos.get(d.span.ref.panel), d.span.ref.inner, at);
      if (!near) return;
      const q = d.tr.feed(near.p);
      const runs = d.which === 'end' ? spansBetween(d.span.n, d.span.p0, q) : spansBetween(d.span.n, q, d.span.p1);
      if (runs.length) d.ends = runsToEnds(d.span.ref, runs);
    } else if (base.length) {
      const copy = base.map((e) => ({ ...e }));
      const target = d.which === 'start' ? copy[0] : copy[copy.length - 1];
      const seg = lineSegs(geos.get(target.panel), target.inner)[target.segment];
      if (seg) {
        const t = nearestOnSeg(seg, at).t;
        if (d.which === 'start') target.t0 = t;
        else target.t1 = t;
        d.ends = copy;
      }
    }
    this.ctx.redraw();
  }

  private seamEnds(id: number, side: 'a' | 'b'): SeamEnd[] {
    const s = this.ctx.doc.sew.seams.find((x) => x.id === id);
    return s ? (side === 'a' ? s.a : s.b) : [];
  }

  up(_p: Ptr): void {
    const d = this.drag;
    this.drag = null;
    this.ctx.setSnapMark(null);
    this.ctx.say('');
    if (!d) return;
    if (d.live && d.ends.length) void this.submit('sew.edit', { seam: d.seam, [d.side]: d.ends.map(spanArg) });
    this.ctx.redraw();
  }

  keydown(e: KeyboardEvent): boolean {
    if ((e.key === 'Delete' || e.key === 'Backspace') && this.ctx.sel.seam !== null && !this.drag) {
      void this.ctx.doc.run('sew.delete', { seam: this.ctx.sel.seam });
      return true;
    }
    return super.keydown(e);
  }

  exact(): void {
    const id = this.ctx.sel.seam;
    const s = this.ctx.doc.sew.seams.find((x) => x.id === id);
    if (!s) { this.ctx.say('Click a seam'); return; }
    const init = { seam: s.id, fold: s.fold, fold_angle: s.fold_angle, fold_strength: s.fold_strength, ratio: s.ratio, tension: s.tension, side: s.side, reversed: s.reversed };
    void this.ctx.exact('sew.edit', init).then((v) => { if (v) void this.submit('sew.edit', v); });
  }

  cancel(): void {
    this.drag = null;
    this.ctx.setSnapMark(null);
    this.ctx.say('');
    this.ctx.redraw();
  }

  busy(): boolean { return this.drag !== null; }

  leave(): void { this.cancel(); }

  draw(g: CanvasRenderingContext2D, c: Colors): void {
    const scene = this.ctx.sew();
    const hov = this.hover !== null && this.hover !== this.ctx.sel.seam ? scene.seams.find((s) => s.seam.id === this.hover) : undefined;
    if (hov) {
      for (const ch of [hov.a, hov.b]) {
        g.save();
        g.beginPath();
        ch.pts.forEach((pt, i) => {
          const s = this.ctx.toScreen(pt);
          if (i === 0) g.moveTo(s[0], s[1]);
          else g.lineTo(s[0], s[1]);
        });
        g.lineWidth = 6;
        g.globalAlpha = 0.3;
        g.strokeStyle = c.accent;
        g.stroke();
        g.restore();
      }
    }
    const d = this.drag;
    if (d && d.live && d.ends.length) strokeEnds(this.ctx, g, d.ends, c.accent, 5, 0.9);
    for (const h of this.handles()) {
      const s = this.ctx.toScreen(h.at);
      g.save();
      g.fillStyle = c.bg;
      g.strokeStyle = c.accent;
      g.lineWidth = 2;
      g.beginPath();
      if (h.which === 'start') g.arc(s[0], s[1], 6, 0, Math.PI * 2);
      else g.rect(s[0] - 5.5, s[1] - 5.5, 11, 11);
      g.fill();
      g.stroke();
      g.restore();
    }
  }
}

export class TackTool extends SewBase {
  id = 'tack';
  label = 'Tack';
  key = 'Shift+T';
  icon = 'tTack';
  private first: { panel: number; pt: Vec } | null = null;
  private at: Vec = [0, 0];

  private locate(p: Ptr): { panel: number; pt: Vec } | null {
    const snapped = this.ctx.snap(p.w).pt;
    const hit = this.pickAt(p, 10);
    const id = hit ? hit.panel : this.panelUnder(p.w);
    if (id === null) return null;
    return { panel: id, pt: snapped };
  }

  private panelUnder(w: Vec): number | null {
    const hit = this.ctx.doc.geos().find((g) => g.poly.length > 2 && inPoly(w, g.poly));
    return hit ? hit.panel.id : null;
  }

  down(p: Ptr): void {
    const tg = tackAt(this.ctx.sew(), p.w, this.ctx.tol(8));
    if (tg && !this.first) {
      this.ctx.sel.tack = tg.tack.id;
      this.ctx.sel.seam = null;
      this.ctx.selected();
      this.ctx.redraw();
      return;
    }
    const at = this.locate(p);
    if (!at) return;
    if (!this.first) {
      this.first = at;
      this.at = at.pt;
      this.ctx.say('Click the second point');
      this.ctx.redraw();
      return;
    }
    const a = this.first;
    if (dist(a.pt, at.pt) < 0.01 && a.panel === at.panel) return;
    this.first = null;
    this.ctx.say('');
    void this.add({ panel_a: a.panel, point_a: a.pt, panel_b: at.panel, point_b: at.pt, space: 'world', length: 0 });
  }

  private async add(args: Record<string, unknown>): Promise<void> {
    const r = await this.ctx.doc.run<{ id?: number }>('tack.add', args);
    if (r && typeof r.id === 'number') {
      this.ctx.sel.tack = r.id;
      this.ctx.sel.seam = null;
      this.ctx.selected();
    }
    this.ctx.redraw();
  }

  move(p: Ptr): void {
    if (this.first) {
      this.at = this.ctx.snap(p.w).pt;
      this.ctx.redraw();
      return;
    }
    const tg = tackAt(this.ctx.sew(), p.w, this.ctx.tol(8));
    this.ctx.cursor(tg ? 'pointer' : 'crosshair');
  }

  keydown(e: KeyboardEvent): boolean {
    if ((e.key === 'Delete' || e.key === 'Backspace') && !this.first && this.ctx.sel.tack !== null) {
      void this.ctx.doc.run('tack.delete', { tack: this.ctx.sel.tack });
      return true;
    }
    return super.keydown(e);
  }

  exact(): void {
    const init: Record<string, unknown> = { space: 'world', length: 0 };
    if (this.first) { init.panel_a = this.first.panel; init.point_a = this.first.pt; }
    this.cancel();
    void this.ctx.exact('tack.add', init).then((v) => { if (v) void this.add(v); });
  }

  cancel(): void {
    this.first = null;
    this.ctx.say('');
    this.ctx.setSnapMark(null);
    this.ctx.redraw();
  }

  busy(): boolean { return this.first !== null; }

  leave(): void { this.cancel(); }

  draw(g: CanvasRenderingContext2D, c: Colors): void {
    const f = this.first;
    if (!f) return;
    const a = this.ctx.toScreen(f.pt);
    const b = this.ctx.toScreen(this.at);
    g.save();
    g.strokeStyle = c.accent;
    g.lineWidth = 1.6;
    g.setLineDash([6, 4]);
    g.beginPath();
    g.moveTo(a[0], a[1]);
    g.lineTo(b[0], b[1]);
    g.stroke();
    g.restore();
    dot(this.ctx, g, f.pt, c.accent, 4);
    worldLabel(g, fmtLen(dist(f.pt, this.at), this.ctx.unit()), b[0] + 12, b[1] - 10, c);
  }
}

function inPoly(p: Vec, poly: Vec[]): boolean {
  let inside = false;
  for (let i = 0, j = poly.length - 1; i < poly.length; j = i++) {
    const a = poly[i];
    const b = poly[j];
    if ((a[1] > p[1]) !== (b[1] > p[1]) && p[0] < ((b[0] - a[0]) * (p[1] - a[1])) / (b[1] - a[1]) + a[0]) inside = !inside;
  }
  return inside;
}

interface PleatSet { kind: 'knife' | 'box'; count: number; depth: number }

function storedPleat(): PleatSet {
  try {
    const v = JSON.parse(localStorage.getItem(PLEAT_KEY) ?? '{}') as Partial<PleatSet>;
    return {
      kind: v.kind === 'box' ? 'box' : 'knife',
      count: Number.isInteger(v.count) && (v.count ?? 0) >= 1 && (v.count ?? 0) <= 64 ? (v.count as number) : 3,
      depth: typeof v.depth === 'number' && v.depth >= 1 ? v.depth : 15,
    };
  } catch { return { kind: 'knife', count: 3, depth: 15 }; }
}

export class PleatTool extends SewBase {
  id = 'pleat';
  label = 'Pleats';
  key = 'Shift+P';
  icon = 'tPleat';
  private set: PleatSet = storedPleat();
  private panel: number | null = null;
  private from: Vec = [0, 0];
  private to: Vec = [0, 0];
  private start: Ptr | null = null;
  private live = false;

  private keep(): void { localStorage.setItem(PLEAT_KEY, JSON.stringify(this.set)); }

  private side(panel: number): 'left' | 'right' {
    const geo = this.ctx.doc.geos().find((g) => g.panel.id === panel);
    if (!geo) return 'left';
    const cross = (this.to[0] - this.from[0]) * (geo.center[1] - this.from[1]) - (this.to[1] - this.from[1]) * (geo.center[0] - this.from[0]);
    return cross >= 0 ? 'left' : 'right';
  }

  private args(panel: number): Record<string, unknown> {
    return { panel, from: this.from, to: this.to, space: 'world', kind: this.set.kind, count: this.set.count, depth: this.set.depth, side: this.side(panel) };
  }

  down(p: Ptr): void {
    const hit = this.ctx.doc.geos().find((g) => g.poly.length > 2 && inPoly(p.w, g.poly));
    const edge = this.pickAt(p, 6);
    const panel = hit?.panel.id ?? edge?.panel;
    if (panel === undefined) return;
    this.panel = panel;
    this.from = this.ctx.snap(p.w).pt;
    this.to = this.from;
    this.start = p;
    this.live = false;
    this.ctx.sel.only(panel);
    this.ctx.selected();
    this.ctx.say('Drag across a panel · Digits set the count · Enter for exact values');
  }

  move(p: Ptr): void {
    if (this.panel === null || !this.start) {
      this.ctx.cursor('crosshair');
      return;
    }
    if (!this.live && !moved(this.start, p)) return;
    this.live = true;
    this.to = this.ctx.snap(p.w).pt;
    this.ctx.redraw();
  }

  up(_p: Ptr): void {
    const panel = this.panel;
    const live = this.live && dist(this.from, this.to) > 1;
    this.panel = null;
    this.start = null;
    this.live = false;
    this.ctx.setSnapMark(null);
    this.ctx.say('');
    if (panel === null) return;
    if (live) {
      this.keep();
      void this.submit('sew.pleat', this.args(panel));
    } else {
      this.panel = panel;
      this.exact();
    }
    this.ctx.redraw();
  }

  keydown(e: KeyboardEvent): boolean {
    const n = Number(e.key);
    if (!e.shiftKey && Number.isInteger(n) && n >= 1 && n <= 9) {
      this.set.count = n;
      this.keep();
      this.ctx.redraw();
      return true;
    }
    return super.keydown(e);
  }

  exact(): void {
    const panel = this.panel ?? this.firstPanel();
    this.cancel();
    if (panel === null) { this.ctx.say('Select a panel first'); return; }
    const init: Record<string, unknown> = { panel, kind: this.set.kind, count: this.set.count, depth: this.set.depth, space: 'world' };
    if (dist(this.from, this.to) > 1) { init.from = this.from; init.to = this.to; }
    void this.ctx.exact('sew.pleat', init).then((v) => {
      if (!v) return;
      if (v.kind === 'box' || v.kind === 'knife') this.set.kind = v.kind;
      if (typeof v.count === 'number') this.set.count = v.count;
      if (typeof v.depth === 'number') this.set.depth = v.depth;
      this.keep();
      void this.submit('sew.pleat', v);
    });
  }

  cancel(): void {
    this.panel = null;
    this.start = null;
    this.live = false;
    this.ctx.setSnapMark(null);
    this.ctx.say('');
    this.ctx.redraw();
  }

  busy(): boolean { return this.panel !== null; }

  leave(): void { this.cancel(); }

  draw(g: CanvasRenderingContext2D, c: Colors): void {
    if (!this.live || this.panel === null) return;
    const dx = this.to[0] - this.from[0];
    const dy = this.to[1] - this.from[1];
    const l = Math.hypot(dx, dy) || 1;
    const sign = this.side(this.panel) === 'left' ? 1 : -1;
    const nx = (-dy / l) * sign;
    const ny = (dx / l) * sign;
    const width = this.set.depth * (this.set.kind === 'box' ? 4 : 2);
    const step = width + this.set.depth * 2;
    g.save();
    g.strokeStyle = c.accent;
    for (let k = 0; k < this.set.count; k++) {
      const off = k * step;
      const a = this.ctx.toScreen([this.from[0] + nx * off, this.from[1] + ny * off]);
      const b = this.ctx.toScreen([this.to[0] + nx * off, this.to[1] + ny * off]);
      g.globalAlpha = k === 0 ? 1 : 0.5;
      g.lineWidth = k === 0 ? 2 : 1.4;
      g.setLineDash(k === 0 ? [] : [6, 4]);
      g.beginPath();
      g.moveTo(a[0], a[1]);
      g.lineTo(b[0], b[1]);
      g.stroke();
    }
    g.restore();
    const e = this.ctx.toScreen(this.to);
    worldLabel(g, `${this.set.count} × ${this.set.kind === 'box' ? 'Box' : 'Knife'} · ${fmtLen(this.set.depth, this.ctx.unit())}`, e[0] + 12, e[1] - 10, c);
  }
}

export class GatherTool extends SewBase {
  id = 'gather';
  label = 'Gather';
  key = 'Shift+G';
  icon = 'tGather';
  private target: { id: number; start: Ptr; current: number } | null = null;
  private ratio = 1;
  private live = false;
  private at: Vec = [0, 0];
  private hover: number | null = null;

  down(p: Ptr): void {
    const hit = seamAt(this.ctx.sew(), p.w, this.ctx.tol(8));
    if (!hit) return;
    const id = hit.seam.seam.id;
    this.target = { id, start: p, current: hit.seam.seam.ratio };
    this.ratio = Math.max(1, hit.seam.seam.ratio);
    this.live = false;
    this.selectSeam(id);
    this.ctx.say('Drag to set the ratio · Enter for exact values');
  }

  move(p: Ptr): void {
    const t = this.target;
    if (!t) {
      const hit = seamAt(this.ctx.sew(), p.w, this.ctx.tol(8));
      const next = hit ? hit.seam.seam.id : null;
      this.ctx.cursor(hit ? 'ew-resize' : 'default');
      if (next !== this.hover) { this.hover = next; this.ctx.redraw(); }
      return;
    }
    if (!this.live && !moved(t.start, p)) return;
    this.live = true;
    const px = Math.hypot(p.sx - t.start.sx, p.sy - t.start.sy);
    this.ratio = Math.min(20, Math.max(1, Math.round((1 + px / 100) * 20) / 20));
    this.at = p.w;
    this.ctx.redraw();
  }

  up(_p: Ptr): void {
    const t = this.target;
    const live = this.live;
    this.target = null;
    this.live = false;
    this.ctx.say('');
    if (!t) return;
    if (live) void this.submit('sew.gather', { seam: t.id, ratio: this.ratio });
    else { this.target = t; this.exact(); }
    this.ctx.redraw();
  }

  exact(): void {
    const id = this.target?.id ?? this.ctx.sel.seam;
    const s = this.ctx.doc.sew.seams.find((x) => x.id === id);
    this.cancel();
    if (!s) { this.ctx.say('Click a seam'); return; }
    void this.ctx.exact('sew.gather', { seam: s.id, ratio: s.ratio > 1 ? s.ratio : 1.5 }).then((v) => { if (v) void this.submit('sew.gather', v); });
  }

  cancel(): void {
    this.target = null;
    this.live = false;
    this.ctx.say('');
    this.ctx.redraw();
  }

  busy(): boolean { return this.target !== null; }

  leave(): void { this.cancel(); }

  draw(g: CanvasRenderingContext2D, c: Colors): void {
    const scene = this.ctx.sew();
    const id = this.target?.id ?? this.hover;
    const sg = id === null ? undefined : scene.seams.find((s) => s.seam.id === id);
    if (sg) {
      for (const ch of [sg.a, sg.b]) {
        g.save();
        g.beginPath();
        ch.pts.forEach((pt, i) => {
          const s = this.ctx.toScreen(pt);
          if (i === 0) g.moveTo(s[0], s[1]);
          else g.lineTo(s[0], s[1]);
        });
        g.lineWidth = 6;
        g.globalAlpha = 0.3;
        g.strokeStyle = c.accent;
        g.stroke();
        g.restore();
      }
    }
    if (this.live) {
      const s = this.ctx.toScreen(this.at);
      worldLabel(g, `${this.ratio.toFixed(2)} : 1`, s[0] + 12, s[1] - 10, c);
    }
  }
}
