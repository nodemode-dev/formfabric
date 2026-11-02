import { textFieldFocused } from '../../ui/native-menu';
import type { OutputStore } from '../output/store';
import type { View3D } from './index';

type Pt = [number, number];

export interface TopoPanel {
  panel: number;
  name: string;
  points: Pt[];
  faces: number[][];
  lines: number[][];
  remeshed: string | null;
}

export interface TopoGhost {
  panel: number;
  pos: Pt;
  from: { panel: number; point: number };
  seam: number;
  fraction: number;
  partner_fraction: number;
  occupied: boolean;
}

export interface TopoState {
  panels: TopoPanel[];
  open: { panel: number; points: number[] } | null;
  ghosts: TopoGhost[];
}

export const EMPTY_TOPO: TopoState = { panels: [], open: null, ghosts: [] };

export interface TopoHooks {
  view: () => View3D | null;
  notify: (message: string, kind: 'info' | 'error') => void;
  onEngine: (cb: (name: string) => void) => void;
  changed: () => void;
}

interface Loc { tri: number; w: [number, number, number] }
interface Cand { panel: number; pos: Pt; kind: 'point' | 'ghost'; index: number; x: number; y: number }

const SNAP_PX = 12;
const CLICK_SLOP = 4;
const DOUBLE_MS = 350;
const STYLE_ID = 'ff3-topo-style';
const CSS = '.ff3-topo{position:absolute;left:0;top:0;pointer-events:none;z-index:var(--z-raised)}';
const tmp = new Float64Array(3);
const tmp2 = new Float64Array(3);

export class TopologyTool {
  state: TopoState = EMPTY_TOPO;
  picked: Array<{ panel: number; index: number }> = [];
  private on = false;
  private overlay: HTMLCanvasElement | null = null;
  private ctx: CanvasRenderingContext2D | null = null;
  private attached: HTMLCanvasElement | null = null;
  private frameId = 0;
  private sig = '';
  private version = 0;
  private cursor: Pt | null = null;
  private down: { x: number; y: number; id: number } | null = null;
  private last: { x: number; y: number; t: number } | null = null;
  private queue: Promise<void> = Promise.resolve();
  private chain: Promise<void> = Promise.resolve();
  private waiting = 0;
  private flight: Promise<void> | null = null;
  private again = false;
  private readonly flats = new Map<number, number[][]>();
  private readonly pending = new Map<number, Promise<number[][] | null>>();
  private readonly locs = new Map<string, Loc | null>();
  private colors: { accent: string; ink: string; bg: string } | null = null;
  private readonly listeners = new Set<() => void>();

  constructor(private readonly store: OutputStore, private readonly hooks: TopoHooks) {
    hooks.onEngine((name) => {
      if (name === 'view.units') return;
      this.flats.clear();
      this.locs.clear();
      if (this.on || this.listeners.size) void this.load();
    });
    new MutationObserver(() => { this.colors = null; this.sig = ''; }).observe(document.documentElement, { attributes: true });
  }

  get active(): boolean { return this.on; }

  onChange(cb: () => void): void { this.listeners.add(cb); }

  private emit(): void {
    for (const cb of this.listeners) cb();
    this.hooks.changed();
  }

  set(on: boolean): void {
    if (on === this.on) return;
    const view = this.hooks.view();
    if (on && !view) { this.hooks.notify('The 3D View is not available', 'error'); return; }
    this.on = on;
    if (on && view) this.attach(view);
    else this.detach();
    if (on) { void this.load(); this.loop(); }
    else { cancelAnimationFrame(this.frameId); this.frameId = 0; this.cursor = null; this.paint(true); }
    this.emit();
  }

  private attach(view: View3D): void {
    const { pane, canvas } = view.toolbox();
    if (!document.getElementById(STYLE_ID)) {
      const style = document.createElement('style');
      style.id = STYLE_ID;
      style.textContent = CSS;
      document.head.appendChild(style);
    }
    if (!this.overlay) {
      this.overlay = document.createElement('canvas');
      this.overlay.className = 'ff3-topo';
      this.ctx = this.overlay.getContext('2d');
    }
    if (this.overlay.parentElement !== pane) pane.appendChild(this.overlay);
    if (this.attached !== canvas) {
      this.detach();
      canvas.addEventListener('pointerdown', this.onDown, true);
      canvas.addEventListener('pointermove', this.onMove, true);
      canvas.addEventListener('pointerup', this.onUp, true);
      canvas.addEventListener('dblclick', this.onDbl, true);
      this.attached = canvas;
    }
    window.addEventListener('keydown', this.onKey, true);
    canvas.style.cursor = 'crosshair';
  }

  private detach(): void {
    window.removeEventListener('keydown', this.onKey, true);
    const c = this.attached;
    if (!c) return;
    c.removeEventListener('pointerdown', this.onDown, true);
    c.removeEventListener('pointermove', this.onMove, true);
    c.removeEventListener('pointerup', this.onUp, true);
    c.removeEventListener('dblclick', this.onDbl, true);
    c.style.cursor = '';
    this.attached = null;
  }

  load(): Promise<void> {
    if (!this.store.has('topology.list')) return Promise.resolve();
    this.again = true;
    if (this.flight) return this.flight;
    this.flight = (async () => {
      while (this.again) {
        this.again = false;
        const r = await this.store.attempt<Partial<TopoState>>('topology.list', {});
        if (r.ok) {
          this.state = { panels: r.result.panels ?? [], open: r.result.open ?? null, ghosts: r.result.ghosts ?? [] };
          this.version++;
          this.locs.clear();
          this.picked = this.picked.filter((p) => this.state.panels.some((t) => t.panel === p.panel && p.index < t.points.length));
          this.emit();
        }
      }
    })().finally(() => { this.flight = null; });
    return this.flight;
  }

  act(name: string, args: object = {}): Promise<boolean> {
    const run = async (): Promise<boolean> => {
      const r = await this.store.attempt<unknown>(name, args);
      if (!r.ok) this.hooks.notify(r.message, 'error');
      this.picked = [];
      await this.load();
      return r.ok;
    };
    const next = this.queue.then(run, run);
    this.queue = next.then(() => undefined, () => undefined);
    return next;
  }

  private rect(): DOMRect | null { return this.attached?.getBoundingClientRect() ?? null; }

  private fetchFlat(panel: number): Promise<number[][] | null> {
    const have = this.flats.get(panel);
    if (have) return Promise.resolve(have);
    const flying = this.pending.get(panel);
    if (flying) return flying;
    const p = this.store.attempt<{ positions?: number[][] }>('mesh.get', { panel }).then((r) => {
      this.pending.delete(panel);
      if (!r.ok || !r.result.positions) return null;
      this.flats.set(panel, r.result.positions);
      this.locs.clear();
      this.sig = '';
      return r.result.positions;
    });
    this.pending.set(panel, p);
    return p;
  }

  private flat(panel: number): number[][] | null {
    const have = this.flats.get(panel);
    if (!have) void this.fetchFlat(panel);
    return have ?? null;
  }

  private locate(view: View3D, panel: number, p: Pt): Loc | null {
    const key = `${panel}|${p[0]}|${p[1]}`;
    const hit = this.locs.get(key);
    if (hit !== undefined) return hit;
    const flat = this.flat(panel);
    const buf = view.toolbox().scene.get(panel);
    if (!flat || !buf || !buf.ready) return null;
    let best: Loc | null = null;
    let miss = Infinity;
    const idx = buf.idx;
    for (let t = 0; t < idx.length; t += 3) {
      const a = flat[idx[t]];
      const b = flat[idx[t + 1]];
      const c = flat[idx[t + 2]];
      if (!a || !b || !c) continue;
      const det = (b[1] - c[1]) * (a[0] - c[0]) + (c[0] - b[0]) * (a[1] - c[1]);
      if (Math.abs(det) < 1e-18) continue;
      const l0 = ((b[1] - c[1]) * (p[0] - c[0]) + (c[0] - b[0]) * (p[1] - c[1])) / det;
      const l1 = ((c[1] - a[1]) * (p[0] - c[0]) + (a[0] - c[0]) * (p[1] - c[1])) / det;
      const l2 = 1 - l0 - l1;
      const m = Math.max(-l0, -l1, -l2, 0);
      if (m < miss) {
        miss = m;
        best = { tri: t / 3, w: [l0, l1, l2] };
        if (m <= 1e-9) break;
      }
    }
    this.locs.set(key, best);
    return best;
  }

  private screen(view: View3D, panel: number, p: Pt, out: Float64Array): boolean {
    const loc = this.locate(view, panel, p);
    const { scene, frame } = view.toolbox();
    const buf = scene.get(panel);
    if (!loc || !buf) return false;
    let x = 0, y = 0, z = 0;
    for (let k = 0; k < 3; k++) {
      const v = buf.idx[loc.tri * 3 + k] * 3;
      const w = Math.max(0, loc.w[k]);
      x += buf.pos[v] * w;
      y += buf.pos[v + 1] * w;
      z += buf.pos[v + 2] * w;
    }
    const s = Math.max(1e-12, loc.w.reduce((a, b) => a + Math.max(0, b), 0));
    return frame.project(x / s, y / s, z / s, out);
  }

  private candidates(view: View3D, only: number | null): Cand[] {
    const out: Cand[] = [];
    for (const t of this.state.panels) {
      if (only !== null && t.panel !== only) continue;
      t.points.forEach((p, index) => {
        if (this.screen(view, t.panel, p, tmp)) out.push({ panel: t.panel, pos: p, kind: 'point', index, x: tmp[0], y: tmp[1] });
      });
    }
    this.state.ghosts.forEach((g, index) => {
      if (only !== null && g.panel !== only) return;
      if (this.screen(view, g.panel, g.pos, tmp)) out.push({ panel: g.panel, pos: g.pos, kind: 'ghost', index, x: tmp[0], y: tmp[1] });
    });
    return out;
  }

  private nearest(view: View3D, only: number | null, x: number, y: number, kinds: 'all' | 'point' = 'all'): Cand | null {
    let best: Cand | null = null;
    let d = SNAP_PX;
    for (const c of this.candidates(view, only)) {
      if (kinds === 'point' && c.kind !== 'point') continue;
      const dist = Math.hypot(c.x - x, c.y - y);
      if (dist <= d) { d = dist; best = c; }
    }
    return best;
  }

  private readonly onDown = (e: PointerEvent): void => {
    if (e.button !== 0) { this.down = null; return; }
    this.down = { x: e.clientX, y: e.clientY, id: e.pointerId };
  };

  private readonly onMove = (e: PointerEvent): void => {
    const r = this.rect();
    if (!r) return;
    this.cursor = [e.clientX - r.left, e.clientY - r.top];
  };

  private readonly onDbl = (e: MouseEvent): void => {
    e.stopImmediatePropagation();
    e.preventDefault();
  };

  private readonly onUp = (e: PointerEvent): void => {
    const d = this.down;
    this.down = null;
    const r = this.rect();
    if (!d || !r || e.button !== 0 || Math.hypot(e.clientX - d.x, e.clientY - d.y) > CLICK_SLOP) return;
    e.stopImmediatePropagation();
    const x = e.clientX - r.left;
    const y = e.clientY - r.top;
    const now = performance.now();
    const prev = this.last;
    this.last = { x, y, t: now };
    if (prev && now - prev.t < DOUBLE_MS && Math.hypot(prev.x - x, prev.y - y) < SNAP_PX && (this.state.open || this.waiting > 0)) {
      this.last = null;
      this.enqueue(() => this.finish());
      return;
    }
    this.enqueue(() => this.click(x, y, e.shiftKey));
  };

  private enqueue(f: () => Promise<unknown>): void {
    this.waiting++;
    const run = (): Promise<unknown> => f().finally(() => { this.waiting--; });
    this.chain = this.chain.then(run, run).then(() => undefined, () => undefined);
  }

  private mmPerPx(view: View3D, buf: { idx: Uint32Array; pos: Float32Array }, tri: number, flat: number[][]): number {
    const { frame } = view.toolbox();
    let low = Infinity;
    for (let k = 0; k < 3; k++) {
      const i = buf.idx[tri * 3 + k];
      const j = buf.idx[tri * 3 + ((k + 1) % 3)];
      const a = flat[i];
      const b = flat[j];
      if (!a || !b) continue;
      if (!frame.project(buf.pos[i * 3], buf.pos[i * 3 + 1], buf.pos[i * 3 + 2], tmp) || !frame.project(buf.pos[j * 3], buf.pos[j * 3 + 1], buf.pos[j * 3 + 2], tmp2)) continue;
      const px = Math.hypot(tmp[0] - tmp2[0], tmp[1] - tmp2[1]);
      if (px < 1) continue;
      low = Math.min(low, Math.hypot(a[0] - b[0], a[1] - b[1]) / px);
    }
    return Number.isFinite(low) ? low : 1;
  }

  private async click(x: number, y: number, shift: boolean): Promise<void> {
    const view = this.hooks.view();
    if (!view) return;
    await this.queue;
    if (shift) {
      const c = this.nearest(view, null, x, y, 'point');
      if (c) this.pick(c.panel, c.index);
      return;
    }
    const hit = view.pickAt(x, y);
    if (!hit) return;
    const open = this.state.open;
    if (open && open.panel !== hit.panel) {
      this.hooks.notify('Finish or cancel the open shape before drawing on another panel', 'info');
      return;
    }
    const buf = view.toolbox().scene.get(hit.panel);
    const flat = await this.fetchFlat(hit.panel);
    if (!buf || !flat) return;
    const at = (k: number): number[] => flat[buf.idx[hit.tri * 3 + k]];
    const a = at(0), b = at(1), c = at(2);
    if (!a || !b || !c) return;
    const w = 1 - hit.u - hit.v;
    const pos: Pt = [w * a[0] + hit.u * b[0] + hit.v * c[0], w * a[1] + hit.u * b[1] + hit.v * c[1]];
    const snap = SNAP_PX * this.mmPerPx(view, buf, hit.tri, flat);
    const near = this.nearest(view, hit.panel, x, y);
    if (near && near.kind === 'point' && open && open.points[0] === near.index && open.points.length >= 3) {
      await this.act('topology.close');
      return;
    }
    if (near) await this.act('topology.add_point', { panel: hit.panel, pos: near.pos, snap: 0.05 });
    else await this.act('topology.add_point', { panel: hit.panel, pos, snap });
  }

  private pick(panel: number, index: number): void {
    const at = this.picked.findIndex((p) => p.panel === panel && p.index === index);
    if (at >= 0) this.picked.splice(at, 1);
    else {
      if (this.picked.length && this.picked[0].panel !== panel) this.picked = [];
      this.picked.push({ panel, index });
      if (this.picked.length > 2) this.picked.shift();
    }
    this.sig = '';
    this.emit();
  }

  async finish(): Promise<void> { await this.act('topology.finish'); }

  private readonly onKey = (e: KeyboardEvent): void => {
    if (!this.on || textFieldFocused() || e.ctrlKey || e.metaKey || e.altKey) return;
    if (e.key === 'Enter') {
      if (this.state.open) void this.finish();
    } else if (e.key === 'Delete' || e.key === 'Backspace') {
      if (this.state.open) void this.act('topology.delete_last');
    } else if (e.key === 'Escape') {
      if (this.state.open) void this.act('topology.cancel');
      else this.set(false);
    } else return;
    e.preventDefault();
    e.stopImmediatePropagation();
  };

  private loop(): void {
    if (!this.on) return;
    this.paint(false);
    this.frameId = requestAnimationFrame(() => this.loop());
  }

  private palette(): { accent: string; ink: string; bg: string } {
    if (this.colors) return this.colors;
    const cs = getComputedStyle(document.documentElement);
    const v = (n: string, d: string): string => cs.getPropertyValue(n).trim() || d;
    this.colors = { accent: v('--accent-color', '#f97316'), ink: v('--text-primary', '#eee'), bg: v('--canvas-bg', '#1c1b1d') };
    return this.colors;
  }

  private paint(clear: boolean): void {
    const view = this.hooks.view();
    const ov = this.overlay;
    const ctx = this.ctx;
    if (!view || !ov || !ctx) return;
    const { frame, scene } = view.toolbox();
    const dpr = window.devicePixelRatio || 1;
    const sig = `${frame.vp.join(',')}|${frame.width}|${frame.height}|${dpr}|${this.version}|${scene.version}|${this.cursor?.join(',') ?? ''}|${this.picked.length}|${this.flats.size}|${clear}`;
    if (sig === this.sig) return;
    this.sig = sig;
    const bw = Math.max(1, Math.round(frame.width * dpr));
    const bh = Math.max(1, Math.round(frame.height * dpr));
    if (ov.width !== bw) ov.width = bw;
    if (ov.height !== bh) ov.height = bh;
    const cw = `${frame.width}px`;
    const ch = `${frame.height}px`;
    if (ov.style.width !== cw) ov.style.width = cw;
    if (ov.style.height !== ch) ov.style.height = ch;
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.clearRect(0, 0, frame.width, frame.height);
    if (clear) return;
    const pal = this.palette();
    const open = this.state.open;
    const at = (panel: number, p: Pt): [number, number] | null => (this.screen(view, panel, p, tmp) ? [tmp[0], tmp[1]] : null);
    ctx.lineWidth = 1.5;
    ctx.lineJoin = 'round';
    for (const t of this.state.panels) {
      const pts = t.points.map((p) => at(t.panel, p));
      ctx.strokeStyle = pal.accent;
      ctx.fillStyle = pal.accent;
      for (const f of t.faces) {
        const ps = f.map((i) => pts[i]);
        if (ps.some((p) => !p)) continue;
        ctx.beginPath();
        ps.forEach((p, i) => { if (p) { if (i) ctx.lineTo(p[0], p[1]); else ctx.moveTo(p[0], p[1]); } });
        ctx.closePath();
        ctx.globalAlpha = 0.16;
        ctx.fill();
        ctx.globalAlpha = 1;
        ctx.stroke();
      }
      for (const l of t.lines) {
        ctx.beginPath();
        let started = false;
        for (const i of l) {
          const p = pts[i];
          if (!p) continue;
          if (started) ctx.lineTo(p[0], p[1]); else { ctx.moveTo(p[0], p[1]); started = true; }
        }
        ctx.stroke();
      }
      if (open && open.panel === t.panel) {
        ctx.strokeStyle = pal.ink;
        ctx.beginPath();
        let started = false;
        for (const i of open.points) {
          const p = pts[i];
          if (!p) continue;
          if (started) ctx.lineTo(p[0], p[1]); else { ctx.moveTo(p[0], p[1]); started = true; }
        }
        if (this.cursor && started) ctx.lineTo(this.cursor[0], this.cursor[1]);
        ctx.setLineDash([5, 4]);
        ctx.stroke();
        ctx.setLineDash([]);
      }
      pts.forEach((p, i) => {
        if (!p) return;
        const chosen = this.picked.some((k) => k.panel === t.panel && k.index === i);
        ctx.beginPath();
        ctx.arc(p[0], p[1], chosen ? 5.5 : 3.5, 0, Math.PI * 2);
        ctx.fillStyle = chosen ? pal.ink : pal.accent;
        ctx.fill();
        ctx.strokeStyle = pal.bg;
        ctx.stroke();
      });
    }
    ctx.strokeStyle = pal.accent;
    for (const g of this.state.ghosts) {
      const p = at(g.panel, g.pos);
      if (!p) continue;
      ctx.globalAlpha = g.occupied ? 0.45 : 1;
      ctx.beginPath();
      ctx.moveTo(p[0], p[1] - 6);
      ctx.lineTo(p[0] + 6, p[1]);
      ctx.lineTo(p[0], p[1] + 6);
      ctx.lineTo(p[0] - 6, p[1]);
      ctx.closePath();
      ctx.setLineDash([3, 2]);
      ctx.stroke();
      ctx.setLineDash([]);
      ctx.globalAlpha = 1;
    }
    if (this.cursor) {
      const near = this.nearest(view, open ? open.panel : null, this.cursor[0], this.cursor[1]);
      if (near) {
        ctx.beginPath();
        ctx.arc(near.x, near.y, 9, 0, Math.PI * 2);
        ctx.strokeStyle = pal.ink;
        ctx.stroke();
      }
    }
  }
}
