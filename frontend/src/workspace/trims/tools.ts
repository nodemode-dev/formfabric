import { dist, place, type PanelGeo } from '../geom';
import type { Colors, Ptr, ViewCtx } from '../pattern-view/ctx';
import { panelAt, worldLabel } from '../pattern-view/render';
import {
  chainOf, lineSegs, nearestParam, pickLine, pointAt, runsToEnds, spansBetween, Tracker,
  type LinePick, type LineRef,
} from '../pattern-view/sew-geom';
import { spanArg, type SeamEnd } from '../pattern-view/sew-types';
import { BaseTool, moved } from '../pattern-view/tools-common';
import { fmtLen } from '../state';
import type { Vec } from '../types';
import { canPickImage, pickImage } from '../fabric/files';
import './icons';
import { removeTrim } from './actions';
import { drawHardware } from './draw';
import { pickTrim, type HwGeo } from './scene';
import { trimState } from './state';
import { findPrint, trimDoc, type FinishKind, type StyleKind, type TrimItem, type TStyle } from './types';

function accepts(kind: StyleKind, style: TStyle): boolean {
  return kind === 'topstitch' ? style.kind === 'topstitch' || style.kind === 'puckering' : style.kind === kind;
}

export async function styleFor(ctx: ViewCtx, kind: StyleKind): Promise<number | null> {
  const doc = trimDoc(ctx.doc.garment);
  const wanted = trimState.style[kind];
  const found = doc.styles.find((s) => s.id === wanted && accepts(kind, s)) ?? doc.styles.find((s) => accepts(kind, s));
  if (found) return found.id;
  const made = await ctx.doc.run<{ id?: number }>('style.create', { kind });
  if (made && typeof made.id === 'number') { trimState.setStyle(kind, made.id); return made.id; }
  return null;
}

async function submit(ctx: ViewCtx, command: string, args: Record<string, unknown>, kind: TrimItem['kind']): Promise<void> {
  const r = await ctx.doc.run<{ id?: number }>(command, args);
  if (r && typeof r.id === 'number') trimState.select({ kind, id: r.id });
  ctx.redraw();
}

async function ask(ctx: ViewCtx, command: string, initial: Record<string, unknown>, kind: TrimItem['kind']): Promise<void> {
  const v = await ctx.exact(command, initial);
  if (v) await submit(ctx, command, v, kind);
}

function hwAt(ctx: ViewCtx, p: Ptr, only?: 'button' | 'buttonhole'): HwGeo | null {
  const scene = ctx.trims();
  const hit = pickTrim(scene, p.w, ctx.tol(9), ['hardware']);
  const hw = hit ? scene.byHardware.get(hit.id) : undefined;
  if (!hw) return null;
  return only && hw.style?.kind !== only ? null : hw;
}

export class HardwareTool extends BaseTool {
  id: string;
  label: string;
  key: string;
  icon: string;
  private ghost: { at: Vec; panel: number } | null = null;

  constructor(ctx: ViewCtx, private readonly kind: 'button' | 'buttonhole') {
    super(ctx);
    this.id = kind;
    this.label = kind === 'button' ? 'Button' : 'Buttonhole';
    this.key = kind === 'button' ? 'Shift+B' : 'Shift+H';
    this.icon = kind === 'button' ? 'tButton' : 'tButtonhole';
  }

  private spot(p: Ptr): { at: Vec; panel: number } | null {
    const at = this.ctx.snap(p.w).pt;
    const panel = panelAt(this.ctx.doc.geos(), at);
    return panel === null ? null : { at, panel };
  }

  move(p: Ptr): void {
    const next = this.spot(p);
    const same = (!next && !this.ghost) || (next && this.ghost && next.panel === this.ghost.panel && dist(next.at, this.ghost.at) < 1e-6);
    this.ghost = next;
    this.ctx.cursor(next ? 'crosshair' : 'default');
    if (!same) this.ctx.redraw();
  }

  down(p: Ptr): void {
    const s = this.spot(p);
    if (!s) return;
    void (async () => {
      const style = await styleFor(this.ctx, this.kind);
      if (style === null) return;
      await submit(this.ctx, `${this.kind}.add`, { style, panel: s.panel, point: s.at, rotation: trimState.opt.rotation }, 'hardware');
    })();
  }

  exact(): void {
    void (async () => {
      const style = await styleFor(this.ctx, this.kind);
      await ask(this.ctx, `${this.kind}.add`, style === null ? {} : { style }, 'hardware');
    })();
  }

  leave(): void { this.ghost = null; this.ctx.setSnapMark(null); }

  draw(g: CanvasRenderingContext2D, c: Colors): void {
    const gh = this.ghost;
    if (!gh) return;
    const panel = this.ctx.doc.panel(gh.panel);
    if (!panel) return;
    const doc = trimDoc(this.ctx.doc.garment);
    const style = doc.styles.find((s) => s.id === trimState.style[this.kind] && s.kind === this.kind) ?? doc.styles.find((s) => s.kind === this.kind);
    const r = (trimState.opt.rotation * Math.PI) / 180;
    const w = style ? Number(style.params.width ?? 12) : 12;
    const len = style ? Number(style.params.length ?? 14) : 14;
    const hw: HwGeo = {
      hw: { id: -1, style: style?.id ?? 0, anchor: { panel: gh.panel, point: gh.at }, rotation: trimState.opt.rotation, fastened_to: null },
      style: style ?? { id: 0, kind: this.kind, name: '', params: {} },
      at: gh.at, angle: r, width: this.kind === 'button' ? w : 3, length: this.kind === 'button' ? w : len, panel,
    };
    g.save();
    g.globalAlpha = 0.6;
    drawHardware(g, hw, (x, y) => this.ctx.toScreen([x, y]), this.ctx.cam.s, c, null);
    g.restore();
  }
}

export class FastenTool extends BaseTool {
  id = 'fasten';
  label = 'Fasten';
  key = 'Shift+F';
  icon = 'tFasten';
  private first: HwGeo | null = null;
  private hover: HwGeo | null = null;

  private hint(): void {
    this.ctx.say(this.first ? 'Click the matching buttonhole or button' : 'Click a button or a buttonhole · Alt-click a button releases it');
  }

  move(p: Ptr): void {
    const h = hwAt(this.ctx, p);
    const same = h?.hw.id === this.hover?.hw.id;
    this.hover = h;
    this.ctx.cursor(h ? 'pointer' : 'default');
    if (!same) this.ctx.redraw();
  }

  down(p: Ptr): void {
    const h = hwAt(this.ctx, p);
    if (!h) return;
    const kind = h.style?.kind;
    if (kind !== 'button' && kind !== 'buttonhole') return;
    if (p.alt) {
      if (kind === 'button') void this.ctx.doc.run('button.fasten', { button: h.hw.id, unfasten: true });
      return;
    }
    if (!this.first) {
      this.first = h;
      trimState.select({ kind: 'hardware', id: h.hw.id });
      this.hint();
      this.ctx.redraw();
      return;
    }
    const a = this.first;
    if (a.style?.kind === kind) {
      this.first = h;
      trimState.select({ kind: 'hardware', id: h.hw.id });
      this.ctx.redraw();
      return;
    }
    const button = kind === 'button' ? h : a;
    const hole = kind === 'button' ? a : h;
    this.first = null;
    this.ctx.say('');
    void this.ctx.doc.run('button.fasten', { button: button.hw.id, buttonhole: hole.hw.id }).then(() => this.ctx.redraw());
  }

  keydown(e: KeyboardEvent): boolean {
    if (e.key === 'Escape' && this.first) { this.cancel(); return true; }
    return false;
  }

  exact(): void {
    void this.ctx.exact('button.fasten', {}).then((v) => { if (v) void this.ctx.doc.run('button.fasten', v); });
  }

  cancel(): void { this.first = null; this.ctx.say(''); this.ctx.redraw(); }
  busy(): boolean { return this.first !== null; }
  leave(): void { this.first = null; this.hover = null; this.ctx.say(''); }

  draw(g: CanvasRenderingContext2D, c: Colors): void {
    for (const h of [this.first, this.hover]) {
      if (!h) continue;
      const s = this.ctx.toScreen(h.at);
      g.save();
      g.strokeStyle = c.accent;
      g.lineWidth = 2;
      g.globalAlpha = h === this.first ? 1 : 0.6;
      g.beginPath();
      g.arc(s[0], s[1], Math.max(9, h.length * this.ctx.cam.s * 0.5 + 5), 0, Math.PI * 2);
      g.stroke();
      g.restore();
    }
    const a = this.first;
    if (a && this.hover && this.hover !== a) {
      const s = this.ctx.toScreen(a.at);
      const e = this.ctx.toScreen(this.hover.at);
      g.save();
      g.strokeStyle = c.accent;
      g.setLineDash([5, 4]);
      g.beginPath();
      g.moveTo(s[0], s[1]);
      g.lineTo(e[0], e[1]);
      g.stroke();
      g.restore();
    }
  }
}

interface Move { item: TrimItem; start: Ptr; grab: Vec; at: Vec; live: boolean }

export class TrimSelectTool extends BaseTool {
  id = 'trimselect';
  label = 'Trim Select';
  key = 'Shift+A';
  icon = 'tTrimSelect';
  private move_: Move | null = null;
  private hover: TrimItem | null = null;

  private pick(p: Ptr): TrimItem | null {
    return pickTrim(this.ctx.trims(), p.w, this.ctx.tol(8));
  }

  down(p: Ptr): void {
    const item = this.pick(p);
    trimState.select(item);
    this.ctx.redraw();
    if (!item) { this.move_ = null; return; }
    const scene = this.ctx.trims();
    let center: Vec | null = null;
    if (item.kind === 'hardware') center = scene.byHardware.get(item.id)?.at ?? null;
    else if (item.kind === 'print') {
      const found = findPrint(this.ctx.doc.garment, item.id);
      const pg = found ? scene.prints.get(found.panel.id)?.find((x) => x.print.id === item.id) : undefined;
      if (pg) center = place(pg.geo.panel, pg.print.position);
    }
    this.move_ = center ? { item, start: p, grab: [center[0] - p.w[0], center[1] - p.w[1]], at: center, live: false } : null;
  }

  move(p: Ptr): void {
    const m = this.move_;
    if (!m) {
      const h = this.pick(p);
      const same = h?.kind === this.hover?.kind && h?.id === this.hover?.id;
      this.hover = h;
      this.ctx.cursor(h ? 'pointer' : 'default');
      if (!same) this.ctx.redraw();
      return;
    }
    if (!m.live && !moved(m.start, p)) return;
    m.live = true;
    const snapped = this.ctx.snap([p.w[0] + m.grab[0], p.w[1] + m.grab[1]]).pt;
    m.at = snapped;
    this.ctx.cursor('grabbing');
    this.ctx.redraw();
  }

  up(_p: Ptr): void {
    const m = this.move_;
    this.move_ = null;
    this.ctx.setSnapMark(null);
    this.ctx.cursor('default');
    if (!m || !m.live) return;
    const panel = panelAt(this.ctx.doc.geos(), m.at);
    if (m.item.kind === 'hardware') {
      const args: Record<string, unknown> = { hardware: m.item.id, point: m.at };
      if (panel !== null) args.panel = panel;
      void this.ctx.doc.run('hardware.move', args).then(() => this.ctx.redraw());
    } else if (m.item.kind === 'print') {
      void this.ctx.doc.run('print.set', { print: m.item.id, position: m.at }).then(() => this.ctx.redraw());
    }
  }

  keydown(e: KeyboardEvent): boolean {
    const item = trimState.item;
    if ((e.key === 'Delete' || e.key === 'Backspace') && item) { void removeItem(this.ctx, item); return true; }
    if (e.key === 'Escape' && item) { trimState.select(null); this.ctx.redraw(); return true; }
    return false;
  }

  cancel(): void { this.move_ = null; this.ctx.redraw(); }
  busy(): boolean { return this.move_?.live === true; }
  leave(): void { this.move_ = null; this.hover = null; }

  draw(g: CanvasRenderingContext2D, c: Colors): void {
    const h = this.hover;
    const scene = this.ctx.trims();
    if (h && h.kind === 'hardware' && !this.move_?.live) {
      const hw = scene.byHardware.get(h.id);
      if (hw) {
        const s = this.ctx.toScreen(hw.at);
        g.save();
        g.strokeStyle = c.accent;
        g.globalAlpha = 0.6;
        g.lineWidth = 2;
        g.beginPath();
        g.arc(s[0], s[1], Math.max(9, hw.length * this.ctx.cam.s * 0.5 + 4), 0, Math.PI * 2);
        g.stroke();
        g.restore();
      }
    }
    const m = this.move_;
    if (m && m.live) {
      const s = this.ctx.toScreen(m.at);
      g.save();
      g.strokeStyle = c.accent;
      g.lineWidth = 1.5;
      g.setLineDash([4, 3]);
      g.beginPath();
      g.arc(s[0], s[1], 12, 0, Math.PI * 2);
      g.stroke();
      g.restore();
    }
  }
}

async function removeItem(ctx: ViewCtx, item: TrimItem): Promise<void> {
  await removeTrim(ctx.doc, item);
  ctx.redraw();
}

interface SpanDrag { ref: LineRef; n: number; closed: boolean; p0: number; tr: Tracker; start: Ptr; pick: LinePick; ends: SeamEnd[]; live: boolean }

function fullEnd(panel: number, inner: number | null, segment: number): SeamEnd {
  return { panel, inner, segment, t0: 0, t1: 1 };
}

export function reverseEnds(ends: SeamEnd[]): SeamEnd[] {
  return ends.slice().reverse().map((e) => ({ ...e, t0: e.t1, t1: e.t0 }));
}

abstract class SpanTool extends BaseTool {
  protected sides: SeamEnd[][] = [[], []];
  private phase = 0;
  private drag: SpanDrag | null = null;
  private hover: LinePick | null = null;

  constructor(ctx: ViewCtx, private readonly phases: 1 | 2) { super(ctx); }

  protected abstract hintFor(phase: number): string;
  protected abstract finish(sides: SeamEnd[][]): void;
  private geos(): Map<number, PanelGeo> {
    return new Map(this.ctx.doc.geos().map((g) => [g.panel.id, g]));
  }

  private say(): void { this.ctx.say(this.hintFor(this.phase)); }

  down(p: Ptr): void {
    const pick = pickLine(this.ctx.doc.geos(), p.w, this.ctx.tol(8));
    if (!pick) return;
    const geo = this.geos().get(pick.panel);
    const n = lineSegs(geo, pick.inner).length;
    if (!geo || n === 0) return;
    const closed = pick.inner === null || (geo.inner.find((i) => i.id === pick.inner)?.closed ?? false);
    const p0 = pick.segment + pick.t;
    this.drag = { ref: { panel: pick.panel, inner: pick.inner }, n, closed, p0, tr: new Tracker(p0, n, closed), start: p, pick, ends: [], live: false };
    this.say();
  }

  move(p: Ptr): void {
    const d = this.drag;
    if (!d) {
      const hit = pickLine(this.ctx.doc.geos(), p.w, this.ctx.tol(8));
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
    const ends = d.live ? d.ends : [fullEnd(d.pick.panel, d.pick.inner, d.pick.segment)];
    if (!ends.length) { this.ctx.redraw(); return; }
    this.sides[this.phase].push(...ends);
    if (p.shift) { this.say(); this.ctx.redraw(); return; }
    this.advance();
  }

  private advance(): void {
    if (!this.sides[this.phase].length) return;
    if (this.phase + 1 < this.phases) {
      this.phase++;
      this.say();
      this.ctx.redraw();
      return;
    }
    const done = this.sides.map((s) => s.slice());
    this.reset();
    this.finish(done);
  }

  protected reset(): void {
    this.sides = [[], []];
    this.phase = 0;
    this.drag = null;
    this.hover = null;
    this.ctx.setSnapMark(null);
    this.ctx.say('');
    this.ctx.redraw();
  }

  keydown(e: KeyboardEvent): boolean {
    if (e.key === 'Enter') { if (this.busy()) this.advance(); else this.exact(); return true; }
    if (e.key === 'Escape') { this.reset(); return true; }
    if ((e.key === 'Delete' || e.key === 'Backspace') && this.sides[this.phase].length) {
      this.sides[this.phase].pop();
      this.ctx.redraw();
      return true;
    }
    return false;
  }

  exact(): void {
    const init: Record<string, unknown> = {};
    const s = this.sides[0];
    if (s.length) init.spans = s.map(spanArg);
    this.reset();
    void this.initial().then((extra) => this.askExact({ ...init, ...extra }));
  }

  protected abstract initial(): Promise<Record<string, unknown>>;
  protected abstract askExact(init: Record<string, unknown>): Promise<void>;

  cancel(): void { this.reset(); }
  busy(): boolean { return this.drag !== null || this.sides[0].length > 0 || this.sides[1].length > 0; }
  leave(): void { this.reset(); }

  protected stroke(g: CanvasRenderingContext2D, ends: SeamEnd[], color: string, width: number, alpha = 0.95): void {
    const ch = chainOf(this.geos(), ends);
    if (ch.pts.length < 2) return;
    g.save();
    g.beginPath();
    ch.pts.forEach((pt, i) => {
      const s = this.ctx.toScreen(pt);
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

  draw(g: CanvasRenderingContext2D, c: Colors): void {
    if (this.hover && !this.drag) this.stroke(g, [fullEnd(this.hover.panel, this.hover.inner, this.hover.segment)], c.accent, 3, 0.45);
    const names = ['A', 'B'];
    for (let i = 0; i < this.phases; i++) {
      const ends = this.sides[i];
      if (!ends.length) continue;
      this.stroke(g, ends, i === 0 ? c.accent : c.text, 4);
      const ch = chainOf(this.geos(), ends);
      if (ch.pts.length > 1 && this.phases > 1) {
        const s = this.ctx.toScreen(pointAt(ch, 0.5));
        worldLabel(g, names[i], s[0] + 10, s[1] - 8, c);
      }
    }
    const d = this.drag;
    if (d && d.live && d.ends.length) {
      this.stroke(g, d.ends, c.accent, 5, 0.9);
      const ch = chainOf(this.geos(), d.ends);
      if (ch.pts.length > 1) {
        const e = this.ctx.toScreen(ch.pts[ch.pts.length - 1]);
        worldLabel(g, fmtLen(ch.total, this.ctx.unit()), e[0] + 12, e[1] - 10, c);
      }
    }
  }
}

export class ZipperTool extends SpanTool {
  id = 'zipper';
  label = 'Zipper';
  key = 'Shift+Z';
  icon = 'tZipper';
  constructor(ctx: ViewCtx) { super(ctx, 2); }

  protected hintFor(phase: number): string {
    return phase === 0 ? 'Drag along the first side in the zipping direction · Shift adds spans · Enter continues' : 'Drag along the other side · Shift adds spans · Enter finishes';
  }

  protected finish(sides: SeamEnd[][]): void {
    const a = sides[0];
    let b = sides[1];
    const geos = new Map(this.ctx.doc.geos().map((g) => [g.panel.id, g]));
    const ca = chainOf(geos, a);
    const cb = chainOf(geos, b);
    if (ca.pts.length && cb.pts.length) {
      const head = ca.pts[0];
      if (dist(cb.pts[0], head) > dist(cb.pts[cb.pts.length - 1], head)) b = reverseEnds(b);
    }
    void styleFor(this.ctx, 'zipper').then((style) => {
      if (style === null) return;
      void submit(this.ctx, 'zipper.add', { style, a: a.map(spanArg), b: b.map(spanArg), closed: 1 }, 'zipper');
    });
  }

  protected async initial(): Promise<Record<string, unknown>> {
    const style = await styleFor(this.ctx, 'zipper');
    return style === null ? {} : { style };
  }

  protected async askExact(init: Record<string, unknown>): Promise<void> { await ask(this.ctx, 'zipper.add', init, 'zipper'); }
}

export class TopstitchTool extends SpanTool {
  id = 'topstitch';
  label = 'Topstitch';
  key = 'Shift+S';
  icon = 'tTopstitch';
  constructor(ctx: ViewCtx) { super(ctx, 1); }

  protected hintFor(): string {
    return 'Drag along an edge or click a segment · Shift keeps adding spans · Enter adds the stitch';
  }

  protected finish(sides: SeamEnd[][]): void {
    void styleFor(this.ctx, 'topstitch').then((style) => {
      if (style === null) return;
      const args: Record<string, unknown> = { style, spans: sides[0].map(spanArg) };
      if (trimState.opt.stitchOffset !== null) args.offset = trimState.opt.stitchOffset;
      void submit(this.ctx, 'stitch.add', args, 'stitch');
    });
  }

  protected async initial(): Promise<Record<string, unknown>> {
    const style = await styleFor(this.ctx, 'topstitch');
    return style === null ? {} : { style };
  }

  protected async askExact(init: Record<string, unknown>): Promise<void> { await ask(this.ctx, 'stitch.add', init, 'stitch'); }
}

export class FinishTool extends SpanTool {
  id = 'finish';
  label = 'Edge Finish';
  key = 'Shift+E';
  icon = 'tFinish';
  constructor(ctx: ViewCtx) { super(ctx, 1); }

  protected hintFor(): string {
    return 'Drag along an edge or click a segment · Shift keeps adding spans · Enter adds the finish';
  }

  protected finish(sides: SeamEnd[][]): void {
    const o = trimState.opt;
    const args: Record<string, unknown> = { kind: o.finishKind as FinishKind, spans: sides[0].map(spanArg) };
    if (o.finishWidth !== null) args.width = o.finishWidth;
    if (o.finishKind === 'elastic' && o.finishRatio !== null) args.ratio = o.finishRatio;
    if (o.finishStrength !== null) args[o.finishKind === 'skive' ? 'percent' : 'strength'] = o.finishStrength;
    void submit(this.ctx, 'finish.add', args, 'finish');
  }

  protected async initial(): Promise<Record<string, unknown>> { return { kind: trimState.opt.finishKind }; }
  protected async askExact(init: Record<string, unknown>): Promise<void> { await ask(this.ctx, 'finish.add', init, 'finish'); }
}

export class PrintTool extends BaseTool {
  id = 'print';
  label = 'Print';
  key = 'Shift+I';
  icon = 'tPrint';
  private ghost: { at: Vec; panel: number } | null = null;

  private spot(p: Ptr): { at: Vec; panel: number } | null {
    const at = this.ctx.snap(p.w).pt;
    const panel = panelAt(this.ctx.doc.geos(), at);
    return panel === null ? null : { at, panel };
  }

  move(p: Ptr): void {
    const next = this.spot(p);
    const same = (!next && !this.ghost) || (next && this.ghost && next.panel === this.ghost.panel && dist(next.at, this.ghost.at) < 1e-6);
    this.ghost = next;
    this.ctx.cursor(next ? 'crosshair' : 'default');
    if (!same) this.ctx.redraw();
  }

  down(p: Ptr): void {
    const s = this.spot(p);
    if (s) void addPrint(this.ctx, s.panel, s.at);
  }

  exact(): void {
    const id = this.ctx.sel.list().find((x) => this.ctx.doc.panel(x)) ?? this.ghost?.panel;
    const o = trimState.opt;
    void ask(this.ctx, 'print.add', { ...(id ? { panel: id } : {}), ...(o.printImage ? { image: o.printImage } : {}), size: [o.printWidth, o.printHeight], side: o.printSide }, 'print');
  }

  leave(): void { this.ghost = null; this.ctx.setSnapMark(null); }

  draw(g: CanvasRenderingContext2D, c: Colors): void {
    const gh = this.ghost;
    if (!gh) return;
    const o = trimState.opt;
    const s = this.ctx.toScreen(gh.at);
    const w = o.printWidth * this.ctx.cam.s;
    const h = o.printHeight * this.ctx.cam.s;
    g.save();
    g.strokeStyle = c.accent;
    g.globalAlpha = 0.7;
    g.lineWidth = 1.4;
    g.setLineDash([5, 4]);
    g.strokeRect(s[0] - w / 2, s[1] - h / 2, w, h);
    g.restore();
  }
}

export async function addPrint(ctx: ViewCtx, panel: number, at: Vec | null): Promise<void> {
  const o = trimState.opt;
  const p = ctx.doc.panel(panel);
  if (!p) return;
  const geo = ctx.doc.geos().find((x) => x.panel.id === panel);
  const center: Vec = at ?? (geo ? geo.center : p.position);
  let image = o.printImage;
  if (!image && canPickImage()) image = (await pickImage()) ?? '';
  const args: Record<string, unknown> = { panel, position: center, size: [o.printWidth, o.printHeight], side: o.printSide };
  if (!image) {
    const v = await ctx.exact('print.add', args);
    if (!v) return;
    if (typeof v.image === 'string') o.printImage = v.image;
    await submit(ctx, 'print.add', v, 'print');
    return;
  }
  o.printImage = image;
  args.image = image;
  await submit(ctx, 'print.add', args, 'print');
}
