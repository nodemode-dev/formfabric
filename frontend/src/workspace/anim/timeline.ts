import type { AnimStore } from './store';
import { rowsOf, type Row } from './types';

export const RULER_H = 24;
export const ROW_H = 22;
const GRAB = 6;
const KEY_R = 5;
const STEPS = [1, 2, 5, 10, 20, 50, 100, 200, 500, 1000, 2000, 5000];

export interface Hooks {
  scrub(frame: number): void;
  range(start: number, end: number): void;
  motion(id: number, patch: { start?: number; lead_in?: number }): void;
  moveKey(track: number, from: number, to: number): void;
  keyMenu(track: number, frame: number, x: number, y: number): void;
  pickRow(row: Row | null): void;
  pickKey(track: number | null, frame: number): void;
  rowScroll(px: number): void;
}

type Hit =
  | { t: 'ruler' }
  | { t: 'start' }
  | { t: 'end' }
  | { t: 'key'; track: number; frame: number }
  | { t: 'lead'; id: number }
  | { t: 'motion'; id: number }
  | { t: 'cache'; id: number }
  | { t: 'row'; row: Row | null };

interface Drag {
  hit: Hit;
  x0: number;
  f0: number;
  moved: boolean;
  a: number;
  b: number;
}

interface Palette { bg: string; band: string; line: string; muted: string; text: string; accent: string; dim: string }

export class Timeline {
  ppf = 6;
  scroll = -4;
  rowY = 0;
  selTrack: number | null = null;
  selFrame = -1;
  selRow: Row | null = null;
  private readonly ctx: CanvasRenderingContext2D;
  private w = 0;
  private h = 0;
  private dpr = 1;
  private raf = 0;
  private pal: Palette | null = null;
  private drag: Drag | null = null;
  private pvMotion: { id: number; start: number; lead_in: number } | null = null;
  private pvKey: { track: number; from: number; to: number } | null = null;
  private pvRange: { start: number; end: number } | null = null;
  private fitted = false;
  private cursor = '';

  constructor(private readonly canvas: HTMLCanvasElement, private readonly store: AnimStore, private readonly hooks: Hooks) {
    this.ctx = canvas.getContext('2d') as CanvasRenderingContext2D;
    new ResizeObserver(() => this.resize()).observe(canvas.parentElement ?? canvas);
    new MutationObserver(() => { this.pal = null; this.invalidate(); }).observe(document.documentElement, { attributes: true });
    canvas.addEventListener('pointerdown', (e) => this.down(e));
    canvas.addEventListener('pointermove', (e) => this.move(e));
    canvas.addEventListener('pointerup', (e) => this.up(e));
    canvas.addEventListener('pointercancel', () => this.cancel());
    canvas.addEventListener('wheel', (e) => this.wheel(e), { passive: false });
    canvas.addEventListener('dblclick', (e) => { if (this.pos(e)[1] < RULER_H) this.fit(); });
    canvas.addEventListener('contextmenu', (e) => this.context(e));
  }

  rows(): Row[] {
    return rowsOf(this.store.anim);
  }

  private resize(): void {
    const host = this.canvas.parentElement;
    if (!host) return;
    const r = host.getBoundingClientRect();
    const w = Math.floor(r.width);
    const h = Math.floor(r.height);
    if (w < 2 || h < 2) return;
    const dpr = window.devicePixelRatio || 1;
    if (w === this.w && h === this.h && dpr === this.dpr) return;
    this.w = w;
    this.h = h;
    this.dpr = dpr;
    this.canvas.width = Math.round(w * dpr);
    this.canvas.height = Math.round(h * dpr);
    this.canvas.style.width = `${w}px`;
    this.canvas.style.height = `${h}px`;
    if (!this.fitted) this.fit(); else this.invalidate();
  }

  fit(): void {
    if (this.w < 10) return;
    this.fitted = true;
    const a = this.store.anim;
    const last = Math.max(a.end, ...a.caches.map((c) => c.start + c.frames), ...a.motions.map((m) => m.start + m.frames), a.start + 1);
    const first = Math.min(a.start, ...a.motions.map((m) => m.start - m.lead_in), 0);
    this.ppf = Math.max(0.2, Math.min(60, (this.w - 28) / Math.max(1, last - first)));
    this.scroll = first - 10 / this.ppf;
    this.invalidate();
  }

  invalidate(): void {
    if (this.raf) return;
    this.raf = requestAnimationFrame(() => { this.raf = 0; this.draw(); });
  }

  private palette(): Palette {
    if (this.pal) return this.pal;
    const s = getComputedStyle(this.canvas);
    const v = (n: string): string => s.getPropertyValue(n).trim() || 'currentColor';
    this.pal = { bg: v('--bg-primary'), band: v('--bg-secondary'), line: v('--border-color'), muted: v('--text-muted'), text: v('--text-primary'), accent: v('--accent-color'), dim: v('--accent-color-dim') };
    return this.pal;
  }

  private x(f: number): number {
    return (f - this.scroll) * this.ppf;
  }

  private frameAt(px: number): number {
    return Math.round(px / this.ppf + this.scroll);
  }

  private maxRowY(): number {
    return Math.max(0, this.rows().length * ROW_H - (this.h - RULER_H));
  }

  setRowY(px: number): void {
    const v = Math.max(0, Math.min(this.maxRowY(), px));
    if (v === this.rowY) return;
    this.rowY = v;
    this.hooks.rowScroll(v);
    this.invalidate();
  }

  private draw(): void {
    const { ctx, w, h } = this;
    if (w < 2) return;
    const p = this.palette();
    const a = this.store.anim;
    const cur = this.store.current;
    ctx.setTransform(this.dpr, 0, 0, this.dpr, 0, 0);
    ctx.clearRect(0, 0, w, h);
    ctx.fillStyle = p.bg;
    ctx.fillRect(0, 0, w, h);
    ctx.font = '10px ui-sans-serif, system-ui, sans-serif';
    ctx.textBaseline = 'middle';

    const rows = this.rows();
    const range = this.pvRange ?? { start: a.start, end: a.end };
    ctx.save();
    ctx.beginPath();
    ctx.rect(0, RULER_H, w, h - RULER_H);
    ctx.clip();
    rows.forEach((row, i) => {
      const y = RULER_H + i * ROW_H - this.rowY;
      if (y + ROW_H < RULER_H || y > h) return;
      if (i % 2 === 0) { ctx.fillStyle = p.band; ctx.globalAlpha = 0.5; ctx.fillRect(0, y, w, ROW_H); ctx.globalAlpha = 1; }
      if (this.selRow && this.selRow.kind === row.kind && this.selRow.id === row.id) { ctx.fillStyle = p.dim; ctx.fillRect(0, y, w, ROW_H); }
      if (row.kind === 'cache') this.drawCache(row.id, y, p);
      else if (row.kind === 'motion') this.drawMotion(row.id, y, p);
      else this.drawTrack(row.id, y, p);
    });
    ctx.fillStyle = p.line;
    const x0 = this.x(range.start);
    const x1 = this.x(range.end);
    ctx.globalAlpha = 0.28;
    ctx.fillStyle = p.bg;
    ctx.fillRect(0, RULER_H, Math.max(0, Math.min(w, x0)), h - RULER_H);
    ctx.fillRect(Math.max(0, x1), RULER_H, Math.max(0, w - x1), h - RULER_H);
    ctx.globalAlpha = 1;
    ctx.restore();

    this.drawRuler(p, range);

    const px = this.x(cur);
    if (px >= -2 && px <= w + 2) {
      ctx.strokeStyle = p.accent;
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.moveTo(Math.round(px) + 0.5, 0);
      ctx.lineTo(Math.round(px) + 0.5, h);
      ctx.stroke();
      ctx.fillStyle = p.accent;
      ctx.beginPath();
      ctx.moveTo(px - 5, 0);
      ctx.lineTo(px + 5, 0);
      ctx.lineTo(px, 8);
      ctx.closePath();
      ctx.fill();
    }
  }

  private drawRuler(p: Palette, range: { start: number; end: number }): void {
    const { ctx, w } = this;
    ctx.fillStyle = p.band;
    ctx.fillRect(0, 0, w, RULER_H);
    ctx.strokeStyle = p.line;
    ctx.beginPath();
    ctx.moveTo(0, RULER_H - 0.5);
    ctx.lineTo(w, RULER_H - 0.5);
    ctx.stroke();
    const step = STEPS.find((s) => s * this.ppf >= 54) ?? STEPS[STEPS.length - 1];
    const minor = step === 1 ? 0 : step === 2 || step === 5 ? 1 : step / 5;
    const f0 = Math.floor(this.scroll / step) * step;
    const f1 = this.scroll + w / this.ppf;
    ctx.fillStyle = p.muted;
    ctx.strokeStyle = p.muted;
    ctx.textAlign = 'left';
    for (let f = f0; f <= f1; f += step) {
      const x = Math.round(this.x(f)) + 0.5;
      ctx.beginPath();
      ctx.moveTo(x, RULER_H - 9);
      ctx.lineTo(x, RULER_H);
      ctx.stroke();
      ctx.fillText(String(f), x + 3, RULER_H - 13);
      if (minor > 0 && minor * this.ppf >= 5) {
        for (let m = f + minor; m < f + step; m += minor) {
          const mx = Math.round(this.x(m)) + 0.5;
          ctx.beginPath();
          ctx.moveTo(mx, RULER_H - 4);
          ctx.lineTo(mx, RULER_H);
          ctx.stroke();
        }
      }
    }
    ctx.fillStyle = p.accent;
    const xs = this.x(range.start);
    const xe = this.x(range.end);
    ctx.fillRect(xs, RULER_H - 3, 2, 3);
    ctx.fillRect(xe - 2, RULER_H - 3, 2, 3);
    ctx.beginPath();
    ctx.moveTo(xs, RULER_H - 11);
    ctx.lineTo(xs + 7, RULER_H - 3);
    ctx.lineTo(xs, RULER_H - 3);
    ctx.closePath();
    ctx.fill();
    ctx.beginPath();
    ctx.moveTo(xe, RULER_H - 11);
    ctx.lineTo(xe - 7, RULER_H - 3);
    ctx.lineTo(xe, RULER_H - 3);
    ctx.closePath();
    ctx.fill();
  }

  private drawCache(id: number, y: number, p: Palette): void {
    const c = this.store.anim.caches.find((k) => k.id === id);
    if (!c) return;
    const { ctx } = this;
    const x = this.x(c.start);
    const wd = Math.max(2, c.frames * this.ppf);
    ctx.globalAlpha = c.visible ? 1 : 0.4;
    ctx.fillStyle = id === this.store.activeId ? p.accent : p.muted;
    ctx.fillRect(x, y + 4, wd, ROW_H - 8);
    ctx.globalAlpha = 1;
    if (wd > 40) { ctx.fillStyle = p.bg; ctx.textAlign = 'left'; ctx.fillText(c.name, x + 5, y + ROW_H / 2, wd - 8); }
  }

  private drawMotion(id: number, y: number, p: Palette): void {
    const m0 = this.store.anim.motions.find((k) => k.id === id);
    if (!m0) return;
    const m = this.pvMotion && this.pvMotion.id === id ? { ...m0, start: this.pvMotion.start, lead_in: this.pvMotion.lead_in } : m0;
    const { ctx } = this;
    const x = this.x(m.start);
    const wd = Math.max(2, m.frames * this.ppf);
    const lead = m.lead_in * this.ppf;
    if (lead > 0) {
      ctx.globalAlpha = 0.45;
      ctx.fillStyle = p.accent;
      ctx.fillRect(x - lead, y + 4, lead, ROW_H - 8);
      ctx.globalAlpha = 1;
    }
    ctx.fillStyle = p.accent;
    ctx.fillRect(x, y + 4, wd, ROW_H - 8);
    ctx.fillStyle = p.bg;
    ctx.fillRect(x - lead, y + 4, 2, ROW_H - 8);
    if (wd > 40) { ctx.textAlign = 'left'; ctx.fillText(m.name, x + 5, y + ROW_H / 2, wd - 8); }
  }

  private drawTrack(id: number, y: number, p: Palette): void {
    const tr = this.store.anim.tracks.find((k) => k.id === id);
    if (!tr) return;
    const { ctx } = this;
    const cy = y + ROW_H / 2;
    for (const k of tr.keys) {
      let f = k.frame;
      if (this.pvKey && this.pvKey.track === id && this.pvKey.from === k.frame) f = this.pvKey.to;
      const x = this.x(f);
      if (x < -KEY_R || x > this.w + KEY_R) continue;
      const sel = this.selTrack === id && this.selFrame === k.frame;
      ctx.fillStyle = sel ? p.text : p.accent;
      ctx.strokeStyle = sel ? p.accent : p.bg;
      ctx.lineWidth = 1.5;
      ctx.beginPath();
      if (k.interp === 'step') ctx.rect(x - KEY_R + 1, cy - KEY_R + 1, 2 * KEY_R - 2, 2 * KEY_R - 2);
      else if (k.interp === 'smooth') ctx.arc(x, cy, KEY_R, 0, Math.PI * 2);
      else { ctx.moveTo(x, cy - KEY_R - 1); ctx.lineTo(x + KEY_R + 1, cy); ctx.lineTo(x, cy + KEY_R + 1); ctx.lineTo(x - KEY_R - 1, cy); ctx.closePath(); }
      ctx.fill();
      ctx.stroke();
    }
    ctx.lineWidth = 1;
  }

  private pos(e: MouseEvent): [number, number] {
    const r = this.canvas.getBoundingClientRect();
    return [e.clientX - r.left, e.clientY - r.top];
  }

  private hit(px: number, py: number): Hit {
    const a = this.store.anim;
    const range = this.pvRange ?? { start: a.start, end: a.end };
    if (py < RULER_H) {
      if (py > RULER_H - 14 && Math.abs(px - this.x(range.start)) <= GRAB + 2) return { t: 'start' };
      if (py > RULER_H - 14 && Math.abs(px - this.x(range.end)) <= GRAB + 2) return { t: 'end' };
      return { t: 'ruler' };
    }
    const rows = this.rows();
    const i = Math.floor((py - RULER_H + this.rowY) / ROW_H);
    const row = rows[i] ?? null;
    if (!row) return { t: 'row', row: null };
    if (row.kind === 'track') {
      const tr = a.tracks.find((k) => k.id === row.id);
      let best: Hit | null = null;
      let bd = KEY_R + 4;
      for (const k of tr?.keys ?? []) {
        const d = Math.abs(px - this.x(k.frame));
        if (d <= bd) { bd = d; best = { t: 'key', track: row.id, frame: k.frame }; }
      }
      return best ?? { t: 'row', row };
    }
    if (row.kind === 'motion') {
      const m = a.motions.find((k) => k.id === row.id);
      if (m) {
        const left = this.x(m.start - m.lead_in);
        const right = this.x(m.start + m.frames);
        if (px >= left - GRAB && px <= left + GRAB) return { t: 'lead', id: m.id };
        if (px > left && px <= right) return { t: 'motion', id: m.id };
      }
      return { t: 'row', row };
    }
    const c = a.caches.find((k) => k.id === row.id);
    if (c && px >= this.x(c.start) && px <= this.x(c.start + c.frames)) return { t: 'cache', id: c.id };
    return { t: 'row', row };
  }

  private setCursor(c: string): void {
    if (c !== this.cursor) { this.cursor = c; this.canvas.style.cursor = c; }
  }

  private down(e: PointerEvent): void {
    if (e.button !== 0) return;
    const [px, py] = this.pos(e);
    const hit = this.hit(px, py);
    this.drag = { hit, x0: px, f0: this.frameAt(px), moved: false, a: 0, b: 0 };
    this.canvas.setPointerCapture(e.pointerId);
    this.store.interacting = true;
    if (hit.t === 'ruler') { this.hooks.scrub(Math.max(0, this.frameAt(px))); }
    else if (hit.t === 'key') { this.selTrack = hit.track; this.selFrame = hit.frame; this.selRow = { kind: 'track', id: hit.track }; this.hooks.pickKey(hit.track, hit.frame); this.hooks.pickRow(this.selRow); }
    else if (hit.t === 'motion' || hit.t === 'lead') { this.selRow = { kind: 'motion', id: hit.id }; this.hooks.pickRow(this.selRow); }
    else if (hit.t === 'cache') { this.selRow = { kind: 'cache', id: hit.id }; this.hooks.pickRow(this.selRow); }
    else if (hit.t === 'row') { this.selRow = hit.row; this.selTrack = null; this.selFrame = -1; this.hooks.pickKey(null, -1); this.hooks.pickRow(hit.row); }
    if (hit.t === 'start') this.drag.a = this.store.anim.start;
    this.invalidate();
  }

  private move(e: PointerEvent): void {
    const [px, py] = this.pos(e);
    const d = this.drag;
    if (!d) {
      const h = this.hit(px, py);
      this.setCursor(h.t === 'start' || h.t === 'end' || h.t === 'lead' ? 'ew-resize' : h.t === 'motion' || h.t === 'key' ? 'grab' : h.t === 'ruler' ? 'col-resize' : '');
      return;
    }
    if (Math.abs(px - d.x0) > 2) d.moved = true;
    const f = Math.max(0, this.frameAt(px));
    const a = this.store.anim;
    switch (d.hit.t) {
      case 'ruler': this.hooks.scrub(f); break;
      case 'start': this.pvRange = { start: Math.min(f, (this.pvRange?.end ?? a.end) - 1), end: this.pvRange?.end ?? a.end }; break;
      case 'end': this.pvRange = { start: this.pvRange?.start ?? a.start, end: Math.max(f, (this.pvRange?.start ?? a.start) + 1) }; break;
      case 'motion': {
        const m = a.motions.find((k) => k.id === (d.hit as { id: number }).id);
        if (m && d.moved) this.pvMotion = { id: m.id, start: Math.max(m.lead_in, m.start + f - d.f0), lead_in: m.lead_in };
        break;
      }
      case 'lead': {
        const m = a.motions.find((k) => k.id === (d.hit as { id: number }).id);
        if (m) this.pvMotion = { id: m.id, start: m.start, lead_in: Math.max(0, m.start - f) };
        break;
      }
      case 'key': {
        if (d.moved) this.pvKey = { track: d.hit.track, from: d.hit.frame, to: Math.max(0, d.hit.frame + f - d.f0) };
        break;
      }
      default: break;
    }
    this.invalidate();
  }

  private up(e: PointerEvent): void {
    const d = this.drag;
    this.drag = null;
    this.store.interacting = false;
    if (this.canvas.hasPointerCapture(e.pointerId)) this.canvas.releasePointerCapture(e.pointerId);
    if (!d) return;
    const range = this.pvRange;
    const pm = this.pvMotion;
    const pk = this.pvKey;
    this.pvRange = null;
    this.pvMotion = null;
    this.pvKey = null;
    if ((d.hit.t === 'start' || d.hit.t === 'end') && range) this.hooks.range(range.start, range.end);
    if (pm) this.hooks.motion(pm.id, d.hit.t === 'lead' ? { lead_in: pm.lead_in } : { start: pm.start });
    if (pk && pk.to !== pk.from) {
      this.selFrame = pk.to;
      this.hooks.pickKey(pk.track, pk.to);
      this.hooks.moveKey(pk.track, pk.from, pk.to);
    }
    this.invalidate();
  }

  private cancel(): void {
    this.drag = null;
    this.pvRange = null;
    this.pvMotion = null;
    this.pvKey = null;
    this.store.interacting = false;
    this.invalidate();
  }

  private context(e: MouseEvent): void {
    const [px, py] = this.pos(e);
    const h = this.hit(px, py);
    if (h.t !== 'key') return;
    e.preventDefault();
    this.selTrack = h.track;
    this.selFrame = h.frame;
    this.hooks.pickKey(h.track, h.frame);
    this.invalidate();
    this.hooks.keyMenu(h.track, h.frame, e.clientX, e.clientY);
  }

  private wheel(e: WheelEvent): void {
    e.preventDefault();
    const [px] = this.pos(e);
    if (e.ctrlKey || e.metaKey) {
      const anchor = px / this.ppf + this.scroll;
      this.ppf = Math.max(0.2, Math.min(80, this.ppf * Math.exp(-e.deltaY * 0.0025)));
      this.scroll = anchor - px / this.ppf;
    } else if (e.shiftKey || Math.abs(e.deltaX) > Math.abs(e.deltaY)) {
      this.scroll += (e.shiftKey ? e.deltaY : e.deltaX) / this.ppf;
    } else {
      this.setRowY(this.rowY + e.deltaY);
    }
    this.invalidate();
  }

  showFrame(frame: number): void {
    const f = this.x(frame);
    if (f < 0 || f > this.w) { this.scroll = frame - (this.w * 0.5) / this.ppf; }
    this.invalidate();
  }
}
