import { exactInput } from '../../ui/exact-input';
import { t } from '../../ui/i18n';
import type { ShellDeps } from '../../shell/deps';
import type { DocStore } from '../doc';
import { unionBounds, type Bounds, type Hit } from '../geom';
import type { Selection } from '../state';
import type { Garment, Vec } from '../types';
import type { SewSnapshot } from './sew-types';
import type { Cam, Colors, Ptr, SnapResult, Tool, ViewCtx } from './ctx';
import { drawScene, gridStep } from './render';
import { StyleLineTool } from '../styleline/tool';
import { DrawPanelTool, EllipseTool, InnerLineTool, InnerShapeTool, RectangleTool } from './tools-create';
import { CurvatureTool, EditTool, MeasureTool, SelectTool, SplitTool } from './tools-edit';
import { GrainTool, MatchTool, NotchTool, SeamTool } from './tools-garment';
import { DartTool, InstanceTool, MirrorTool, SlashTool, SpiralTool, TraceTool, UnfoldTool } from './tools-derive';
import { buildSewScene, type SewScene } from './sew-geom';
import { EditSeamTool, FreeSeamTool, GatherTool, PleatTool, SewTool, TackTool } from './tools-sew';
import { onImageReady } from '../trims/images';
import { sceneOf, type TrimScene } from '../trims/scene';
import { trimState } from '../trims/state';
import { FastenTool, FinishTool, HardwareTool, PrintTool, TopstitchTool, TrimSelectTool, ZipperTool } from '../trims/tools';

export interface ViewHooks {
  selected(): void;
  cursor(w: Vec | null): void;
  zoom(s: number): void;
  tool(id: string): void;
}

const MIN_S = 0.02;
const MAX_S = 400;

export class PatternView implements ViewCtx {
  cam: Cam = { x: 300, y: 200, s: 0.6 };
  readonly tools: Tool[];
  private active: Tool;
  private readonly g: CanvasRenderingContext2D;
  private w = 0;
  private h = 0;
  private dpr = 1;
  private queued = false;
  private hover: Hit | null = null;
  private mark: SnapResult | null = null;
  private colorSet: Colors | null = null;
  private readonly probe: HTMLElement;
  private panning: { x: number; y: number } | null = null;
  private space = false;
  private rulersOn: boolean;
  private rect: DOMRect | null = null;
  private sewMemo: { g: Garment; s: SewSnapshot; scene: SewScene } | null = null;
  private trimMemo: { g: Garment; scene: TrimScene } | null = null;

  constructor(
    private readonly pane: HTMLElement,
    private readonly canvas: HTMLCanvasElement,
    private readonly deps: ShellDeps,
    readonly doc: DocStore,
    readonly sel: Selection,
    private readonly hooks: ViewHooks,
  ) {
    this.g = canvas.getContext('2d') as CanvasRenderingContext2D;
    this.rulersOn = localStorage.getItem('formfabric.rulers') === '1';
    this.tools = [
      new SelectTool(this), new EditTool(this), new DrawPanelTool(this), new RectangleTool(this), new EllipseTool(this),
      new InnerShapeTool(this), new InnerLineTool(this), new StyleLineTool(this), new SplitTool(this), new CurvatureTool(this), new MeasureTool(this),
      new SeamTool(this), new NotchTool(this), new GrainTool(this), new DartTool(this), new SlashTool(this), new MirrorTool(this),
      new InstanceTool(this), new UnfoldTool(this), new SpiralTool(this), new TraceTool(this), new MatchTool(this),
      new SewTool(this), new FreeSeamTool(this), new EditSeamTool(this), new TackTool(this), new PleatTool(this), new GatherTool(this),
      new HardwareTool(this, 'button'), new HardwareTool(this, 'buttonhole'), new FastenTool(this), new ZipperTool(this),
      new TopstitchTool(this), new FinishTool(this), new PrintTool(this), new TrimSelectTool(this),
    ];
    this.active = this.tools[0];
    onImageReady(() => this.redraw());
    trimState.onChange(() => this.redraw());
    this.probe = document.createElement('span');
    this.probe.style.cssText = 'position:absolute;width:0;height:0;visibility:hidden;pointer-events:none';
    pane.appendChild(this.probe);

    const ro = new ResizeObserver(() => this.resize());
    ro.observe(pane);
    const dirty = (): void => { this.colorSet = null; this.redraw(); };
    new MutationObserver(dirty).observe(document.documentElement, { attributes: true });
    new MutationObserver(dirty).observe(document.body, { attributes: true, attributeFilter: ['class'] });
    document.addEventListener('nm:ui-settings-changed', dirty);

    canvas.addEventListener('pointerdown', (e) => this.onDown(e));
    canvas.addEventListener('pointermove', (e) => this.onMove(e));
    canvas.addEventListener('pointerup', (e) => this.onUp(e));
    canvas.addEventListener('pointercancel', (e) => this.onUp(e));
    canvas.addEventListener('pointerleave', () => { if (!this.panning) this.hooks.cursor(null); });
    canvas.addEventListener('pointerenter', () => { this.rect = null; });
    window.addEventListener('resize', () => { this.rect = null; });
    canvas.addEventListener('dblclick', (e) => this.active.dbl?.(this.ptr(e)));
    canvas.addEventListener('contextmenu', (e) => { e.preventDefault(); this.active.exact(); });
    canvas.addEventListener('wheel', (e) => this.onWheel(e), { passive: false });
    this.resize();
  }

  get tool(): Tool { return this.active; }
  get rulers(): boolean { return this.rulersOn; }

  setRulers(on: boolean): void {
    this.rulersOn = on;
    localStorage.setItem('formfabric.rulers', on ? '1' : '0');
    this.redraw();
  }

  setTool(id: string): void {
    const next = this.tools.find((x) => x.id === id);
    if (!next || next === this.active) return;
    this.active.leave?.();
    this.active = next;
    this.hover = null;
    this.mark = null;
    this.cursor('default');
    this.hooks.tool(id);
    this.redraw();
  }

  unit(): string { return this.deps.prefs().units; }

  toWorld(sx: number, sy: number): Vec {
    return [this.cam.x + (sx - this.w / 2) / this.cam.s, this.cam.y - (sy - this.h / 2) / this.cam.s];
  }

  toScreen(p: Vec): Vec {
    return [this.w / 2 + (p[0] - this.cam.x) * this.cam.s, this.h / 2 - (p[1] - this.cam.y) * this.cam.s];
  }

  tol(px: number): number { return px / this.cam.s; }

  snap(w: Vec, exclude: number[] = []): SnapResult {
    const p = this.deps.prefs();
    if (!(p.snapGrid || p.snapPoints || p.snapEdges)) { this.mark = null; return { pt: w, kind: null }; }
    const step = Math.max(p.gridSize, gridStep(this.cam.s).minor);
    let out: SnapResult = { pt: w, kind: null };
    const wasm = this.doc.wasm;
    if (wasm) {
      const targets: string[] = [];
      if (p.snapPoints) targets.push('point', 'midpoint', 'inner');
      if (p.snapEdges) targets.push('segment');
      if (p.snapGrid) targets.push('grid');
      const args: Record<string, unknown> = { point: w, targets, tolerance: this.tol(8), grid: p.snapGrid ? step : 0 };
      if (exclude.length) args.exclude = exclude;
      const r = wasm.dispatch<{ snapped: boolean; point: Vec; kind: string | null }>('pattern.snap', args);
      if (r.ok && r.result.snapped) out = { pt: r.result.point, kind: r.result.kind };
    } else if (p.snapGrid) {
      out = { pt: [Math.round(w[0] / step) * step, Math.round(w[1] / step) * step], kind: 'grid' };
    }
    this.mark = out.kind ? out : null;
    return out;
  }

  setSnapMark(m: SnapResult | null): void { this.mark = m; }

  setHover(h: Hit | null): void {
    const same = (!h && !this.hover) || (h && this.hover && h.kind === this.hover.kind && h.panel === this.hover.panel && h.index === this.hover.index);
    this.hover = h;
    if (!same) this.redraw();
  }

  cursor(css: string): void {
    if (this.canvas.style.cursor !== css) this.canvas.style.cursor = css;
  }

  say(text: string): void { this.deps.status.setHint(text); }

  selected(): void { this.hooks.selected(); }

  sew(): SewScene {
    const g = this.doc.garment;
    const s = this.doc.sew;
    if (!this.sewMemo || this.sewMemo.g !== g || this.sewMemo.s !== s) this.sewMemo = { g, s, scene: buildSewScene(this.doc.geos(), s) };
    return this.sewMemo.scene;
  }

  trims(): TrimScene {
    const g = this.doc.garment;
    if (!this.trimMemo || this.trimMemo.g !== g) this.trimMemo = { g, scene: sceneOf(this.doc.geos(), g) };
    return this.trimMemo.scene;
  }

  async exact(commandId: string, initial: Record<string, unknown>): Promise<Record<string, unknown> | null> {
    const cat = await this.deps.engine.catalogue();
    const c = cat.commands.find((x) => x.id === commandId);
    if (!c) return null;
    return exactInput(t(c.label).replace(/\.\.\.$/, ''), c.inputSchema, initial);
  }

  frame(ids?: number[]): void {
    let b: Bounds | null = null;
    const want = ids ?? (this.sel.panels.size ? this.sel.list() : null);
    for (const g of this.doc.geos()) if (!want || want.includes(g.panel.id)) b = unionBounds(b, g.bounds);
    if (!b || this.w === 0) { this.cam = { x: 300, y: 200, s: 0.6 }; this.hooks.zoom(this.cam.s); this.redraw(); return; }
    const pad = 70;
    const bw = Math.max(b[2] - b[0], 10);
    const bh = Math.max(b[3] - b[1], 10);
    const s = Math.min((this.w - pad * 2) / bw, (this.h - pad * 2) / bh);
    this.cam = { x: (b[0] + b[2]) / 2, y: (b[1] + b[3]) / 2, s: Math.max(MIN_S, Math.min(MAX_S, s)) };
    this.hooks.zoom(this.cam.s);
    this.redraw();
  }

  private ptr(e: PointerEvent | MouseEvent): Ptr {
    if (e.type === 'pointerdown' || !this.rect) this.rect = this.canvas.getBoundingClientRect();
    const r = this.rect;
    const sx = e.clientX - r.left;
    const sy = e.clientY - r.top;
    return { sx, sy, w: this.toWorld(sx, sy), shift: e.shiftKey, alt: e.altKey, mod: e.ctrlKey || e.metaKey, button: e.button };
  }

  private onDown(e: PointerEvent): void {
    if (!e.isPrimary) return;
    const focused = document.activeElement;
    if (focused instanceof HTMLElement && focused !== document.body) focused.blur();
    this.canvas.setPointerCapture(e.pointerId);
    if (e.button === 1 || (e.button === 0 && this.space)) {
      e.preventDefault();
      this.panning = { x: e.clientX, y: e.clientY };
      this.cursor('grabbing');
      return;
    }
    if (e.button !== 0) return;
    this.active.down(this.ptr(e));
  }

  private onMove(e: PointerEvent): void {
    if (!e.isPrimary) return;
    if (this.panning) {
      this.cam.x -= (e.clientX - this.panning.x) / this.cam.s;
      this.cam.y += (e.clientY - this.panning.y) / this.cam.s;
      this.panning = { x: e.clientX, y: e.clientY };
      this.redraw();
      return;
    }
    const p = this.ptr(e);
    this.hooks.cursor(p.w);
    this.active.move(p);
  }

  private onUp(e: PointerEvent): void {
    if (!e.isPrimary) return;
    if (this.canvas.hasPointerCapture(e.pointerId)) this.canvas.releasePointerCapture(e.pointerId);
    if (this.panning) {
      this.panning = null;
      this.cursor(this.space ? 'grab' : 'default');
      return;
    }
    if (e.button === 0) this.active.up(this.ptr(e));
  }

  private onWheel(e: WheelEvent): void {
    e.preventDefault();
    const r = this.canvas.getBoundingClientRect();
    const sx = e.clientX - r.left;
    const sy = e.clientY - r.top;
    const before = this.toWorld(sx, sy);
    const factor = Math.exp(-e.deltaY * (e.ctrlKey ? 0.01 : 0.0015));
    const s = Math.max(MIN_S, Math.min(MAX_S, this.cam.s * factor));
    this.cam.s = s;
    this.cam.x = before[0] - (sx - this.w / 2) / s;
    this.cam.y = before[1] + (sy - this.h / 2) / s;
    this.hooks.zoom(s);
    this.redraw();
  }

  setSpace(on: boolean): void {
    if (this.space === on) return;
    this.space = on;
    if (!this.panning) this.cursor(on ? 'grab' : 'default');
  }

  keydown(e: KeyboardEvent): boolean {
    const mod = e.ctrlKey || e.metaKey;
    if (mod && !e.altKey && e.key.toLowerCase() === 'z' && this.active.busy()) { this.active.cancel(); return true; }
    if (this.active.keydown?.(e)) return true;
    if (mod || e.altKey) return false;
    const k = e.key.toUpperCase();
    if (k === 'F' && !e.shiftKey) { this.frame(); return true; }
    const combo = e.shiftKey ? `SHIFT+${k}` : k;
    const tool = this.tools.find((x) => x.key.toUpperCase() === combo);
    if (tool) { this.setTool(tool.id); return true; }
    if ((e.key === 'Delete' || e.key === 'Backspace') && this.sel.panels.size) {
      void this.doc.run('pattern.delete', { panels: this.sel.list() });
      return true;
    }
    return false;
  }

  private resolve(name: string): string {
    this.probe.style.color = `var(${name})`;
    return getComputedStyle(this.probe).color;
  }

  private colors(): Colors {
    this.colorSet ??= {
      bg: this.resolve('--canvas-bg'),
      grid: this.resolve('--border-color'),
      text: this.resolve('--text-primary'),
      muted: this.resolve('--text-muted'),
      accent: this.resolve('--accent-color'),
      line: this.resolve('--text-secondary'),
      warn: this.resolve('--warn'),
    };
    return this.colorSet;
  }

  private resizeTimer = 0;

  private resize(): void {
    const r = this.pane.getBoundingClientRect();
    const w = Math.max(0, Math.round(r.width));
    const h = Math.max(0, Math.round(r.height));
    this.rect = null;
    if (this.w > 0 && this.h > 0 && w > 0 && h > 0 && (w !== this.w || h !== this.h)) {
      this.canvas.style.width = `${w}px`;
      this.canvas.style.height = `${h}px`;
      window.clearTimeout(this.resizeTimer);
      this.resizeTimer = window.setTimeout(() => this.afterResize(), 140);
      return;
    }
    this.settle();
  }

  private afterResize(): void {
    if (document.documentElement.hasAttribute('data-ft-dock-drag')) {
      this.resizeTimer = window.setTimeout(() => this.afterResize(), 140);
      return;
    }
    this.settle();
  }

  private settle(): void {
    window.clearTimeout(this.resizeTimer);
    const r = this.pane.getBoundingClientRect();
    const w = Math.max(0, Math.round(r.width));
    const h = Math.max(0, Math.round(r.height));
    const dpr = window.devicePixelRatio || 1;
    this.rect = null;
    if (w === this.w && h === this.h && dpr === this.dpr) {
      this.canvas.style.width = `${w}px`;
      this.canvas.style.height = `${h}px`;
      return;
    }
    this.w = w;
    this.h = h;
    this.dpr = dpr;
    this.canvas.width = Math.round(w * dpr);
    this.canvas.height = Math.round(h * dpr);
    this.canvas.style.width = `${w}px`;
    this.canvas.style.height = `${h}px`;
    this.paint();
  }

  redraw(): void {
    if (this.queued) return;
    this.queued = true;
    requestAnimationFrame(() => { this.queued = false; this.paint(); });
  }

  private paint(): void {
    if (this.w === 0 || this.h === 0) return;
    const dpr = window.devicePixelRatio || 1;
    if (dpr !== this.dpr) { this.dpr = dpr; this.canvas.width = Math.round(this.w * dpr); this.canvas.height = Math.round(this.h * dpr); }
    const g = this.g;
    g.setTransform(this.dpr, 0, 0, this.dpr, 0, 0);
    const colors = this.colors();
    drawScene(g, {
      w: this.w, h: this.h, cam: this.cam, geos: this.doc.geos(), sel: this.sel, hover: this.hover,
      showPoints: this.active.id === 'edit', rulers: this.rulersOn, unit: this.unit(), colors, sew: this.sew(), trims: this.trims(),
    }, this.active);
    const m = this.mark;
    if (m && m.kind) {
      const s = this.toScreen(m.pt);
      g.save();
      g.strokeStyle = colors.accent;
      g.lineWidth = 1.5;
      g.beginPath();
      if (m.kind === 'grid') { g.moveTo(s[0] - 5, s[1]); g.lineTo(s[0] + 5, s[1]); g.moveTo(s[0], s[1] - 5); g.lineTo(s[0], s[1] + 5); }
      else if (m.kind === 'midpoint') { g.moveTo(s[0], s[1] - 6); g.lineTo(s[0] + 6, s[1] + 5); g.lineTo(s[0] - 6, s[1] + 5); g.closePath(); }
      else if (m.kind === 'segment') { g.moveTo(s[0] - 5, s[1] - 5); g.lineTo(s[0] + 5, s[1] + 5); g.moveTo(s[0] + 5, s[1] - 5); g.lineTo(s[0] - 5, s[1] + 5); }
      else g.rect(s[0] - 5, s[1] - 5, 10, 10);
      g.stroke();
      g.restore();
    }
  }
}
