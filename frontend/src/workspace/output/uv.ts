import type { ShellDeps } from '../../shell/deps';
import { el, q, setClass, setHidden, setText } from '../../ui/dom';
import { onLanguage, t } from '../../ui/i18n';
import { icon } from '../../ui/icons';
import { pointInPoly } from '../geom';
import type { OutputStore } from './store';
import type { UvIslandInfo, Vec } from './types';
import { button, numRow, titled } from './widgets';

type Mode = 'move' | 'rotate' | 'scale';

interface Gesture {
  mode: Mode;
  island: UvIslandInfo;
  from: Vec;
  centre: Vec;
  dx: number;
  dy: number;
  turn: number;
  grow: number;
}

interface Colors { bg: string; tile: string; line: string; grid: string; accent: string; dim: string; text: string; muted: string; off: string }

const MODES: Array<[Mode, string, string, string]> = [
  ['move', 'moveTool', 'G', 'Move'],
  ['rotate', 'rotateTool', 'R', 'Rotate'],
  ['scale', 'scaleTool', 'S', 'Scale'],
];
const MIN_S = 8;
const MAX_S = 20000;
const OPEN_KEY = 'formfabric.output.uv';

const HTML =
  '<div class="ou-uv-bar" data-k="bar"><div class="ff-seg" data-k="modes" role="group"></div>' +
  '<button class="cv-icon-btn" type="button" data-k="frame"></button><span class="ou-uv-gap"></span>' +
  '<span class="ou-uv-info" data-k="info"></span><button class="cv-icon-btn" type="button" data-k="close"></button></div>' +
  '<div class="ou-uv-lay" data-k="lay"></div>' +
  '<div class="ou-uv-stage" data-k="stage"><canvas class="ou-uv-canvas" data-k="canvas" tabindex="0"></canvas><p class="ou-uv-empty" data-k="empty" hidden></p></div>';

const tileOf = (index: number, cols: number): Vec => [index % cols, Math.floor(index / cols)];

function centreOf(loops: Vec[][], off: Vec): Vec {
  let x0 = Infinity, y0 = Infinity, x1 = -Infinity, y1 = -Infinity;
  for (const l of loops) for (const p of l) { x0 = Math.min(x0, p[0]); y0 = Math.min(y0, p[1]); x1 = Math.max(x1, p[0]); y1 = Math.max(y1, p[1]); }
  if (!Number.isFinite(x0)) return [off[0] + 0.5, off[1] + 0.5];
  return [(x0 + x1) / 2 + off[0], (y0 + y1) / 2 + off[1]];
}

export class UvEditor {
  private readonly root: HTMLElement;
  private readonly canvas: HTMLCanvasElement;
  private readonly g: CanvasRenderingContext2D;
  private readonly info: HTMLElement;
  private readonly empty: HTMLElement;
  private readonly modeButtons = new Map<Mode, HTMLButtonElement>();
  private readonly padding = numRow('Padding', '', '0.005', 0, 0.1);
  private readonly tiles = numRow('Tiles', '', '1', 1, 16);
  private readonly layout = button('Layout', 'primary');
  private cam = { x: 0.5, y: 0.5, s: 220 };
  private w = 0;
  private h = 0;
  private dpr = 1;
  private queued = false;
  private open = false;
  private mode: Mode = 'move';
  private selected: number | null = null;
  private gesture: Gesture | null = null;
  private pan: { x: number; y: number } | null = null;
  private space = false;
  private framed = false;
  private colors: Colors | null = null;
  private busy = false;

  constructor(pane: HTMLElement, private readonly deps: ShellDeps, private readonly store: OutputStore, private readonly toggle: HTMLButtonElement, private readonly sync: (open: boolean) => void) {
    this.root = el('<section class="ou-uv" hidden></section>');
    this.root.innerHTML = HTML;
    pane.appendChild(this.root);
    this.canvas = q<HTMLCanvasElement>(this.root, '[data-k="canvas"]');
    this.g = this.canvas.getContext('2d') as CanvasRenderingContext2D;
    this.info = q(this.root, '[data-k="info"]');
    this.empty = q(this.root, '[data-k="empty"]');

    const modes = q(this.root, '[data-k="modes"]');
    for (const [id, ico, key, text] of MODES) {
      const b = el<HTMLButtonElement>(`<button class="ff-seg-btn" type="button">${icon(ico, 16)}</button>`);
      b.addEventListener('click', (e) => { this.setMode(id); if (e.detail > 0) b.blur(); });
      b.dataset.label = text;
      b.dataset.key = key;
      modes.appendChild(b);
      this.modeButtons.set(id, b);
    }
    const retitle = (): void => {
      for (const b of this.modeButtons.values()) { const s = `${t(b.dataset.label ?? '')} (${b.dataset.key})`; if (b.title !== s) b.title = s; }
    };
    retitle();
    onLanguage(retitle);
    const frame = q<HTMLButtonElement>(this.root, '[data-k="frame"]');
    frame.innerHTML = icon('frameAll', 16);
    titled(frame, 'Frame');
    frame.addEventListener('click', (e) => { this.frame(); if (e.detail > 0) frame.blur(); });
    const close = q<HTMLButtonElement>(this.root, '[data-k="close"]');
    close.innerHTML = icon('close', 16);
    titled(close, 'Close');
    close.addEventListener('click', () => this.setOpen(false));

    const lay = q(this.root, '[data-k="lay"]');
    lay.append(this.padding.el, this.tiles.el, this.layout);
    this.padding.el.classList.add('ou-inline');
    this.tiles.el.classList.add('ou-inline');
    this.padding.set(0.01);
    this.tiles.set(1);
    this.setMode('move');
    this.layout.addEventListener('click', () => { void this.runLayout(); });

    const ro = new ResizeObserver(() => this.resize());
    ro.observe(q(this.root, '[data-k="stage"]'));
    const dirty = (): void => { this.colors = null; this.redraw(); };
    new MutationObserver(dirty).observe(document.documentElement, { attributes: true });
    document.addEventListener('nm:ui-settings-changed', dirty);
    this.canvas.addEventListener('pointerdown', (e) => this.onDown(e));
    this.canvas.addEventListener('pointermove', (e) => this.onMove(e));
    this.canvas.addEventListener('pointerup', (e) => this.onUp(e));
    this.canvas.addEventListener('pointercancel', () => this.cancel());
    this.canvas.addEventListener('wheel', (e) => this.onWheel(e), { passive: false });
    this.canvas.addEventListener('contextmenu', (e) => e.preventDefault());
    window.addEventListener('keydown', (e) => this.onKey(e), true);
    window.addEventListener('keyup', (e) => { if (e.code === 'Space') this.space = false; });
    this.toggle.addEventListener('click', (e) => { this.setOpen(!this.open); if (e.detail > 0) this.toggle.blur(); });

    store.onChange(() => this.refresh());
    deps.engine.onEngineChanged((e) => { if (this.open && e.name !== 'view.units') void store.loadUv(); });
    this.paintState();
    if (localStorage.getItem(OPEN_KEY) === '1') this.setOpen(true);
  }

  get isOpen(): boolean { return this.open; }

  setOpen(on: boolean): void {
    if (on === this.open) return;
    this.open = on;
    localStorage.setItem(OPEN_KEY, on ? '1' : '0');
    setHidden(this.root, !on);
    setClass(this.toggle, 'is-active', on);
    this.toggle.setAttribute('aria-expanded', String(on));
    this.sync(on);
    if (on) {
      if (this.deps.layout.get() === '3d') this.deps.layout.set('split');
      this.resize();
      void this.store.loadUv();
      this.paintState();
    }
  }

  private setMode(m: Mode): void {
    this.mode = m;
    for (const [id, b] of this.modeButtons) setClass(b, 'is-on', id === m);
  }

  private paintState(): void {
    const have = this.store.has('uv.get');
    const canSet = this.store.has('uv.island.set');
    const canLay = this.store.has('uv.layout');
    if (this.layout.disabled !== (!canLay || this.busy)) this.layout.disabled = !canLay || this.busy;
    this.padding.disable(!canLay);
    this.tiles.disable(!canLay);
    for (const b of this.modeButtons.values()) if (b.disabled !== !canSet) b.disabled = !canSet;
    this.layout.title = canLay ? '' : t('Not available in this engine build');
    const n = this.store.uv.islands.length;
    let msg = '';
    if (!this.store.known) msg = t('Waiting for the engine...');
    else if (!have) msg = t('UV commands are not available in this engine build.');
    else if (n === 0) msg = t('No islands yet. Mesh the panels, then press Layout.');
    setText(this.empty, msg);
    setHidden(this.empty, msg === '');
  }

  private refresh(): void {
    const uv = this.store.uv;
    if (!this.padding.input.matches(':focus')) this.padding.set(uv.padding);
    if (this.selected !== null && !uv.islands.some((i) => i.panel === this.selected)) this.selected = null;
    this.paintState();
    this.paintInfo();
    if (!this.framed && uv.islands.length) { this.framed = true; this.frame(); }
    this.redraw();
  }

  private paintInfo(): void {
    const i = this.store.uv.islands.find((x) => x.panel === this.selected);
    if (!i) { setText(this.info, `${this.store.uv.islands.length} ${t('islands')}`); return; }
    setText(this.info, `${i.name}   U ${i.offset[0].toFixed(3)}  V ${i.offset[1].toFixed(3)}   ${Math.round(i.rotation)}°   ${(i.scale * 1000).toPrecision(3)} ${t('UV per m')}   ${t('Tile')} ${1001 + i.tile}`);
  }

  private async runLayout(): Promise<void> {
    if (this.busy) return;
    this.busy = true;
    this.paintState();
    const padding = this.padding.get();
    const tiles = Math.max(1, Math.round(this.tiles.get()));
    const r = await this.store.attempt('uv.layout', { padding, tiles });
    this.busy = false;
    if (!r.ok) this.deps.toast(r.message, 'error');
    await this.store.loadUv();
    this.paintState();
  }

  private resize(): void {
    const stage = q(this.root, '[data-k="stage"]');
    const w = Math.round(stage.clientWidth);
    const h = Math.round(stage.clientHeight);
    const dpr = window.devicePixelRatio || 1;
    if (w === this.w && h === this.h && dpr === this.dpr) return;
    this.w = w;
    this.h = h;
    this.dpr = dpr;
    this.canvas.width = Math.max(1, Math.round(w * dpr));
    this.canvas.height = Math.max(1, Math.round(h * dpr));
    this.canvas.style.width = `${w}px`;
    this.canvas.style.height = `${h}px`;
    if (this.open && this.store.uv.islands.length && !this.framed) { this.framed = true; this.frame(); }
    this.redraw();
  }

  frame(): void {
    const uv = this.store.uv;
    if (!this.w || !this.h) return;
    const cols = uv.cols;
    const rows = Math.max(1, Math.ceil(uv.tiles / cols));
    const s = Math.min((this.w - 80) / cols, (this.h - 80) / rows);
    this.cam = { x: cols / 2, y: rows / 2, s: Math.max(MIN_S, Math.min(MAX_S, s)) };
    this.redraw();
  }

  private toWorld(sx: number, sy: number): Vec {
    return [this.cam.x + (sx - this.w / 2) / this.cam.s, this.cam.y - (sy - this.h / 2) / this.cam.s];
  }

  private toScreen(x: number, y: number): Vec {
    return [this.w / 2 + (x - this.cam.x) * this.cam.s, this.h / 2 - (y - this.cam.y) * this.cam.s];
  }

  private world(i: UvIslandInfo): Vec[][] {
    const o = tileOf(i.tile, this.store.uv.cols);
    return i.loops.map((l) => l.map((p) => [p[0] + o[0], p[1] + o[1]] as Vec));
  }

  private hit(w: Vec): UvIslandInfo | null {
    const list = this.store.uv.islands;
    for (let k = list.length - 1; k >= 0; k--) {
      for (const l of this.world(list[k])) if (l.length > 2 && pointInPoly(w, l)) return list[k];
    }
    return null;
  }

  private ptr(e: PointerEvent): { sx: number; sy: number; w: Vec } {
    const r = this.canvas.getBoundingClientRect();
    const sx = e.clientX - r.left;
    const sy = e.clientY - r.top;
    return { sx, sy, w: this.toWorld(sx, sy) };
  }

  private onDown(e: PointerEvent): void {
    this.canvas.focus();
    this.canvas.setPointerCapture(e.pointerId);
    if (e.button === 1 || e.button === 2 || (e.button === 0 && this.space)) {
      e.preventDefault();
      this.pan = { x: e.clientX, y: e.clientY };
      return;
    }
    if (e.button !== 0) return;
    const p = this.ptr(e);
    const hit = this.hit(p.w);
    this.selected = hit ? hit.panel : null;
    this.paintInfo();
    if (hit && this.store.has('uv.island.set')) {
      const centre = centreOf(this.world(hit), [0, 0]);
      this.gesture = { mode: this.mode, island: hit, from: p.w, centre, dx: 0, dy: 0, turn: 0, grow: 1 };
    }
    this.redraw();
  }

  private onMove(e: PointerEvent): void {
    if (this.pan) {
      this.cam.x -= (e.clientX - this.pan.x) / this.cam.s;
      this.cam.y += (e.clientY - this.pan.y) / this.cam.s;
      this.pan = { x: e.clientX, y: e.clientY };
      this.redraw();
      return;
    }
    const gst = this.gesture;
    if (!gst) return;
    const p = this.ptr(e).w;
    if (gst.mode === 'move') {
      gst.dx = p[0] - gst.from[0];
      gst.dy = p[1] - gst.from[1];
    } else if (gst.mode === 'rotate') {
      const a0 = Math.atan2(gst.from[1] - gst.centre[1], gst.from[0] - gst.centre[0]);
      const a1 = Math.atan2(p[1] - gst.centre[1], p[0] - gst.centre[0]);
      let deg = ((a1 - a0) * 180) / Math.PI;
      if (e.shiftKey) deg = Math.round(deg / 15) * 15;
      gst.turn = deg;
    } else {
      const d0 = Math.hypot(gst.from[0] - gst.centre[0], gst.from[1] - gst.centre[1]) || 1e-6;
      const d1 = Math.hypot(p[0] - gst.centre[0], p[1] - gst.centre[1]);
      gst.grow = Math.min(20, Math.max(0.05, d1 / d0));
    }
    this.redraw();
  }

  private onUp(e: PointerEvent): void {
    if (this.canvas.hasPointerCapture(e.pointerId)) this.canvas.releasePointerCapture(e.pointerId);
    if (this.pan) { this.pan = null; return; }
    const gst = this.gesture;
    this.gesture = null;
    if (!gst) return;
    const moved = gst.mode === 'move' ? Math.hypot(gst.dx, gst.dy) > 1e-5 : gst.mode === 'rotate' ? Math.abs(gst.turn) > 1e-3 : Math.abs(gst.grow - 1) > 1e-4;
    if (!moved) { this.redraw(); return; }
    void this.commit(gst);
  }

  private cancel(): void {
    this.gesture = null;
    this.pan = null;
    this.redraw();
  }

  private async commit(gst: Gesture): Promise<void> {
    const i = gst.island;
    const uv = this.store.uv;
    let offset: Vec = [i.offset[0], i.offset[1]];
    let tile = i.tile;
    let rotation = i.rotation;
    let scale = i.scale;
    if (gst.mode === 'move') {
      offset = [offset[0] + gst.dx, offset[1] + gst.dy];
      const rows = Math.max(1, Math.ceil(uv.tiles / uv.cols));
      const c = centreOf(this.world(i), [gst.dx, gst.dy]);
      const col = Math.floor(c[0]);
      const row = Math.floor(c[1]);
      if (col >= 0 && col < uv.cols && row >= 0 && row < rows && col + row * uv.cols < uv.tiles) {
        const next = col + row * uv.cols;
        if (next !== tile) {
          const a = tileOf(tile, uv.cols);
          offset = [offset[0] + a[0] - col, offset[1] + a[1] - row];
          tile = next;
        }
      }
    } else if (gst.mode === 'rotate') rotation += gst.turn;
    else scale *= gst.grow;
    const r = await this.store.attempt('uv.island.set', { panel: i.panel, offset, rotation, scale, tile });
    if (!r.ok) this.deps.toast(r.message, 'error');
    await this.store.loadUv();
  }

  private onWheel(e: WheelEvent): void {
    e.preventDefault();
    const p = this.ptr(e as unknown as PointerEvent);
    const before = p.w;
    const factor = Math.exp(-e.deltaY * (e.ctrlKey ? 0.01 : 0.0015));
    const s = Math.max(MIN_S, Math.min(MAX_S, this.cam.s * factor));
    this.cam.s = s;
    this.cam.x = before[0] - (p.sx - this.w / 2) / s;
    this.cam.y = before[1] + (p.sy - this.h / 2) / s;
    this.redraw();
  }

  private onKey(e: KeyboardEvent): void {
    if (!this.open) return;
    if (e.code === 'Space' && this.canvas === document.activeElement) { this.space = true; e.preventDefault(); e.stopImmediatePropagation(); return; }
    if (this.canvas !== document.activeElement || e.ctrlKey || e.metaKey || e.altKey) return;
    const k = e.key.toLowerCase();
    if (k === 'escape' && this.gesture) { this.cancel(); e.preventDefault(); e.stopImmediatePropagation(); return; }
    const mode = k === 'g' ? 'move' : k === 'r' ? 'rotate' : k === 's' ? 'scale' : null;
    if (mode) this.setMode(mode);
    else if (k === 'f') this.frame();
    else return;
    e.preventDefault();
    e.stopImmediatePropagation();
  }

  private palette(): Colors {
    if (this.colors) return this.colors;
    const cs = getComputedStyle(document.documentElement);
    const v = (n: string, d: string): string => cs.getPropertyValue(n).trim() || d;
    this.colors = {
      bg: v('--canvas-bg', '#1c1b1d'),
      tile: v('--bg-primary', '#111'),
      line: v('--border-light', '#34333a'),
      grid: v('--border-color', '#2a292e'),
      accent: v('--accent-color', '#f97316'),
      dim: v('--accent-color-dim', '#f9731633'),
      text: v('--text-primary', '#eee'),
      muted: v('--text-muted', '#717072'),
      off: v('--uv-island-off', 'transparent'),
    };
    return this.colors;
  }

  redraw(): void {
    if (this.queued || !this.open) return;
    this.queued = true;
    requestAnimationFrame(() => { this.queued = false; this.paint(); });
  }

  private paint(): void {
    const g = this.g;
    const c = this.palette();
    const uv = this.store.uv;
    g.setTransform(this.dpr, 0, 0, this.dpr, 0, 0);
    g.fillStyle = c.bg;
    g.fillRect(0, 0, this.w, this.h);
    for (let k = 0; k < uv.tiles; k++) {
      const o = tileOf(k, uv.cols);
      const [x0, y0] = this.toScreen(o[0], o[1] + 1);
      const side = this.cam.s;
      g.fillStyle = c.tile;
      g.fillRect(x0, y0, side, side);
      g.strokeStyle = c.grid;
      g.lineWidth = 1;
      g.beginPath();
      const steps = side > 360 ? 10 : side > 120 ? 4 : 1;
      for (let n = 1; n < steps; n++) {
        const d = (side * n) / steps;
        g.moveTo(Math.round(x0 + d) + 0.5, y0);
        g.lineTo(Math.round(x0 + d) + 0.5, y0 + side);
        g.moveTo(x0, Math.round(y0 + d) + 0.5);
        g.lineTo(x0 + side, Math.round(y0 + d) + 0.5);
      }
      g.stroke();
      g.strokeStyle = c.line;
      g.strokeRect(Math.round(x0) + 0.5, Math.round(y0) + 0.5, side, side);
      g.fillStyle = c.muted;
      g.font = '11px sans-serif';
      g.fillText(String(1001 + k), x0 + 6, y0 + 15);
    }
    const live = this.gesture;
    for (const island of uv.islands) {
      const on = island.panel === this.selected;
      let loops = this.world(island);
      if (live && live.island === island) loops = this.preview(loops, live);
      g.beginPath();
      for (const l of loops) {
        l.forEach((p, n) => { const [sx, sy] = this.toScreen(p[0], p[1]); if (n === 0) g.moveTo(sx, sy); else g.lineTo(sx, sy); });
        g.closePath();
      }
      g.fillStyle = on ? c.dim : c.off;
      g.fill();
      g.strokeStyle = on ? c.accent : c.muted;
      g.lineWidth = on ? 2 : 1;
      g.stroke();
      if (this.cam.s > 90) {
        const ce = centreOf(loops, [0, 0]);
        const [tx, ty] = this.toScreen(ce[0], ce[1]);
        g.fillStyle = on ? c.accent : c.text;
        g.font = '11px sans-serif';
        g.textAlign = 'center';
        g.fillText(island.name, tx, ty + 4);
        g.textAlign = 'start';
      }
    }
  }

  private preview(loops: Vec[][], gst: Gesture): Vec[][] {
    const [cx, cy] = gst.centre;
    const a = (gst.turn * Math.PI) / 180;
    const co = Math.cos(a);
    const si = Math.sin(a);
    return loops.map((l) => l.map((p) => {
      if (gst.mode === 'move') return [p[0] + gst.dx, p[1] + gst.dy] as Vec;
      const x = p[0] - cx;
      const y = p[1] - cy;
      if (gst.mode === 'rotate') return [cx + x * co - y * si, cy + x * si + y * co] as Vec;
      return [cx + x * gst.grow, cy + y * gst.grow] as Vec;
    }));
  }
}
