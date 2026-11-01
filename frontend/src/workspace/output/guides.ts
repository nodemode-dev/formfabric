import { setClass } from '../../ui/dom';
import type { DocStore } from '../doc';
import { place, pointInPoly, unplace } from '../geom';
import type { PatternView } from '../pattern-view/view';
import type { OutputStore } from './store';
import type { Guide, Vec } from './types';

export class GuideLayer {
  private readonly canvas: HTMLCanvasElement;
  private readonly g: CanvasRenderingContext2D;
  private on = false;
  private w = 0;
  private h = 0;
  private dpr = 1;
  private sig = '';
  private draft: Guide | null = null;
  private cursor: Vec | null = null;
  private frameId = 0;
  private colors: { line: string; draft: string; dot: string } | null = null;
  private readonly changed: (on: boolean) => void;

  constructor(private readonly pane: HTMLElement, private readonly view: PatternView, private readonly doc: DocStore, private readonly store: OutputStore, changed: (on: boolean) => void) {
    this.changed = changed;
    this.canvas = document.createElement('canvas');
    this.canvas.className = 'ou-guides';
    this.canvas.hidden = true;
    pane.appendChild(this.canvas);
    this.g = this.canvas.getContext('2d') as CanvasRenderingContext2D;
    new ResizeObserver(() => this.resize()).observe(pane);
    pane.addEventListener('pointerdown', (e) => this.down(e), true);
    pane.addEventListener('pointermove', (e) => this.move(e), true);
    pane.addEventListener('dblclick', (e) => { if (this.on) { e.stopPropagation(); this.finish(); } }, true);
    window.addEventListener('keydown', (e) => this.key(e), true);
    store.onChange(() => { this.sig = ''; });
    const dirty = (): void => { this.colors = null; this.sig = ''; };
    new MutationObserver(dirty).observe(document.documentElement, { attributes: true });
  }

  get active(): boolean { return this.on; }

  set(on: boolean): void {
    if (on === this.on) return;
    this.on = on;
    this.canvas.hidden = !on;
    setClass(this.pane, 'ou-guiding', on);
    if (!on) { this.draft = null; this.cursor = null; cancelAnimationFrame(this.frameId); this.frameId = 0; }
    else { this.resize(); this.sig = ''; this.loop(); }
    this.changed(on);
  }

  private resize(): void {
    const w = this.pane.clientWidth;
    const h = this.pane.clientHeight;
    const dpr = window.devicePixelRatio || 1;
    if (w === this.w && h === this.h && dpr === this.dpr) return;
    this.w = w; this.h = h; this.dpr = dpr;
    this.canvas.width = Math.max(1, Math.round(w * dpr));
    this.canvas.height = Math.max(1, Math.round(h * dpr));
    this.canvas.style.width = `${w}px`;
    this.canvas.style.height = `${h}px`;
    this.sig = '';
  }

  private loop(): void {
    if (!this.on) return;
    const c = this.view.cam;
    const sig = `${c.x}|${c.y}|${c.s}|${this.w}|${this.h}|${this.draft?.points.length ?? -1}|${this.cursor?.[0] ?? ''}|${this.cursor?.[1] ?? ''}`;
    if (sig !== this.sig) { this.sig = sig; this.paint(); }
    this.frameId = requestAnimationFrame(() => this.loop());
  }

  private palette(): { line: string; draft: string; dot: string } {
    if (this.colors) return this.colors;
    const cs = getComputedStyle(document.documentElement);
    const v = (n: string, d: string): string => cs.getPropertyValue(n).trim() || d;
    this.colors = { line: v('--accent-color', '#f97316'), draft: v('--text-primary', '#eee'), dot: v('--canvas-bg', '#1c1b1d') };
    return this.colors;
  }

  private worldOf(p: Guide): Vec[] {
    const panel = this.doc.panel(p.panel);
    return panel ? p.points.map((q) => place(panel, q)) : [];
  }

  private paint(): void {
    const g = this.g;
    const c = this.palette();
    g.setTransform(this.dpr, 0, 0, this.dpr, 0, 0);
    g.clearRect(0, 0, this.w, this.h);
    const draw = (pts: Vec[], color: string, extra: Vec | null): void => {
      if (!pts.length) return;
      g.strokeStyle = color;
      g.lineWidth = 2;
      g.lineJoin = 'round';
      g.beginPath();
      pts.forEach((p, i) => { const [x, y] = this.view.toScreen(p); if (i === 0) g.moveTo(x, y); else g.lineTo(x, y); });
      if (extra) { const [x, y] = this.view.toScreen(extra); g.setLineDash([5, 4]); g.lineTo(x, y); }
      g.stroke();
      g.setLineDash([]);
      g.fillStyle = color;
      for (const p of pts) { const [x, y] = this.view.toScreen(p); g.beginPath(); g.arc(x, y, 3.5, 0, Math.PI * 2); g.fill(); }
    };
    for (const gd of this.store.retopo.guides) draw(this.worldOf(gd), c.line, null);
    if (this.draft) draw(this.worldOf(this.draft), c.draft, this.cursor);
  }

  private local(e: PointerEvent): Vec {
    const r = this.pane.getBoundingClientRect();
    return this.view.toWorld(e.clientX - r.left, e.clientY - r.top);
  }

  private down(e: PointerEvent): void {
    if (!this.on || e.button !== 0) return;
    if (!(e.target instanceof HTMLCanvasElement) || e.target === this.canvas) return;
    const w = this.local(e);
    e.stopPropagation();
    e.preventDefault();
    if (!this.draft) {
      const geo = [...this.doc.geos()].reverse().find((gg) => gg.poly.length > 2 && pointInPoly(w, gg.poly));
      if (!geo) return;
      this.draft = { panel: geo.panel.id, points: [unplace(geo.panel, w)] };
    } else {
      const panel = this.doc.panel(this.draft.panel);
      if (!panel) return;
      this.draft.points.push(unplace(panel, w));
    }
    this.sig = '';
  }

  private move(e: PointerEvent): void {
    if (!this.on || !this.draft) return;
    this.cursor = this.local(e);
  }

  private key(e: KeyboardEvent): void {
    if (!this.on) return;
    if (e.key === 'Escape') {
      if (this.draft) { this.draft = null; this.sig = ''; } else this.set(false);
      e.preventDefault();
      e.stopImmediatePropagation();
    } else if (e.key === 'Enter' && this.draft) {
      this.finish();
      e.preventDefault();
      e.stopImmediatePropagation();
    } else if ((e.key === 'Backspace' || e.key === 'Delete') && this.draft) {
      this.draft.points.pop();
      if (!this.draft.points.length) this.draft = null;
      this.sig = '';
      e.preventDefault();
      e.stopImmediatePropagation();
    }
  }

  private finish(): void {
    const d = this.draft;
    this.draft = null;
    this.sig = '';
    if (!d || d.points.length < 2) return;
    void this.submit([...this.store.retopo.guides, d]);
  }

  async submit(guides: Guide[]): Promise<void> {
    const r = await this.store.run('retopo.set', { guides });
    if (r !== undefined) await this.store.loadRetopo();
  }
}
