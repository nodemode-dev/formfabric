import type { PatternView } from '../pattern-view/view';
import { lineSegs, pickLine } from '../pattern-view/sew-geom';
import { segLength } from '../geom';
import type { DocStore } from '../doc';

export interface Span { panel: number; segment: number; t0: number; t1: number }
export interface PortDraft { name: string; spans: Span[] }

const same = (a: Span, b: Span): boolean => a.panel === b.panel && a.segment === b.segment;

export class PortPicker {
  active = false;
  current: Span[] = [];
  ports: PortDraft[] = [];
  onChange: () => void = () => undefined;
  private readonly canvas: HTMLCanvasElement;
  private frame = 0;
  private readonly down: (e: PointerEvent) => void;
  private readonly keys: (e: KeyboardEvent) => void;

  constructor(private readonly pane: HTMLElement, private readonly view: PatternView, private readonly doc: DocStore) {
    this.canvas = document.createElement('canvas');
    this.canvas.className = 'ff-lib-pick';
    this.canvas.hidden = true;
    pane.appendChild(this.canvas);
    new ResizeObserver(() => this.size()).observe(pane);
    this.down = (e) => this.pointer(e);
    this.keys = (e) => { if (e.key === 'Escape' && this.active) { e.preventDefault(); e.stopPropagation(); this.stop(); } };
  }

  private size(): void {
    const dpr = window.devicePixelRatio || 1;
    const w = Math.round(this.pane.clientWidth * dpr), h = Math.round(this.pane.clientHeight * dpr);
    if (this.canvas.width !== w) this.canvas.width = w;
    if (this.canvas.height !== h) this.canvas.height = h;
  }

  start(): void {
    if (this.active) return;
    this.active = true;
    this.seen = '';
    this.canvas.hidden = false;
    this.pane.classList.add('ff-lib-picking');
    this.pane.addEventListener('pointerdown', this.down, true);
    window.addEventListener('keydown', this.keys, true);
    this.size();
    this.loop();
    this.onChange();
  }

  stop(): void {
    if (!this.active) return;
    this.active = false;
    this.canvas.hidden = true;
    this.pane.classList.remove('ff-lib-picking');
    this.pane.removeEventListener('pointerdown', this.down, true);
    window.removeEventListener('keydown', this.keys, true);
    cancelAnimationFrame(this.frame);
    this.onChange();
  }

  reset(): void {
    this.current = [];
    this.ports = [];
    this.onChange();
  }

  commit(name: string): boolean {
    if (!this.current.length) return false;
    this.ports.push({ name, spans: this.current });
    this.current = [];
    this.onChange();
    return true;
  }

  removePort(name: string): void {
    this.ports = this.ports.filter((p) => p.name !== name);
    this.onChange();
  }

  lengthOf(spans: Span[]): number {
    let sum = 0;
    for (const s of spans) {
      const seg = lineSegs(this.doc.geos().find((g) => g.panel.id === s.panel), null)[s.segment];
      if (seg) sum += segLength(seg) * (s.t1 - s.t0);
    }
    return sum;
  }

  private pointer(e: PointerEvent): void {
    if (e.button !== 0) return;
    const canvas = this.pane.querySelector<HTMLCanvasElement>('canvas.ff-canvas');
    if (!canvas) return;
    const r = canvas.getBoundingClientRect();
    const w = this.view.toWorld(e.clientX - r.left, e.clientY - r.top);
    const hit = pickLine(this.doc.geos(), w, this.view.tol(10));
    e.preventDefault();
    e.stopPropagation();
    if (!hit || hit.inner !== null) return;
    const span: Span = { panel: hit.panel, segment: hit.segment, t0: 0, t1: 1 };
    const at = this.current.findIndex((s) => same(s, span));
    if (at >= 0) this.current.splice(at, 1); else this.current.push(span);
    this.onChange();
  }

  private seen = '';

  private loop = (): void => {
    if (!this.active) return;
    const c = this.view.cam;
    const sig = `${c.x}|${c.y}|${c.s}|${this.canvas.width}|${this.canvas.height}|${this.current.length}|${this.ports.length}|${this.doc.garment.panels.length}`;
    if (sig !== this.seen) { this.seen = sig; this.paint(); }
    this.frame = requestAnimationFrame(this.loop);
  };

  private paint(): void {
    const ctx = this.canvas.getContext('2d');
    if (!ctx) return;
    const dpr = window.devicePixelRatio || 1;
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);
    const css = getComputedStyle(document.documentElement);
    const accent = css.getPropertyValue('--accent-color').trim() || '#f97316';
    const dim = css.getPropertyValue('--text-secondary').trim() || '#a4a4a4';
    const geos = this.doc.geos();
    const line = (s: Span, color: string, width: number): void => {
      const seg = lineSegs(geos.find((g) => g.panel.id === s.panel), null)[s.segment];
      if (!seg) return;
      const a = this.view.toScreen(seg.a), b = this.view.toScreen(seg.b);
      ctx.strokeStyle = color;
      ctx.lineWidth = width;
      ctx.beginPath();
      ctx.moveTo(a[0], a[1]);
      if (seg.curved) {
        const c1 = this.view.toScreen(seg.c1), c2 = this.view.toScreen(seg.c2);
        ctx.bezierCurveTo(c1[0], c1[1], c2[0], c2[1], b[0], b[1]);
      } else ctx.lineTo(b[0], b[1]);
      ctx.stroke();
    };
    for (const p of this.ports) for (const s of p.spans) line(s, dim, 4);
    for (const s of this.current) line(s, accent, 5);
  }
}
