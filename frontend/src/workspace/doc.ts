import type { WasmEngine } from '../engine/wasm';
import type { EngineApi } from '../shell/deps';
import { geoOf, type PanelGeo } from './geom';
import type { Seam, SeamInfo, SewSnapshot, Tack } from './pattern-view/sew-types';
import { EMPTY_SEW } from './pattern-view/sew-types';
import type { Garment, Panel, PanelMeasure } from './types';

export type Step = [string, Record<string, unknown>];

const EMPTY: Garment = { units: 'mm', panels: [] };

export class DocStore {
  garment: Garment = EMPTY;
  sew: SewSnapshot = EMPTY_SEW;
  wasm: WasmEngine | null = null;
  ready = false;
  private readonly listeners = new Set<() => void>();
  private inflight: Promise<void> | null = null;
  private again = false;
  private holding = 0;
  private held = false;
  private trialBase: Garment | null = null;
  private trialKey = new Map<number, string>();

  constructor(private readonly engine: EngineApi, private readonly fail: (message: string) => void) {}

  onChange(cb: () => void): () => void {
    this.listeners.add(cb);
    return () => { this.listeners.delete(cb); };
  }

  private emit(): void {
    for (const cb of this.listeners) cb();
  }

  geos(): PanelGeo[] {
    return this.garment.panels.map(geoOf);
  }

  panel(id: number): Panel | undefined {
    return this.garment.panels.find((p) => p.id === id);
  }

  setWasm(w: WasmEngine | null): void {
    this.wasm = w;
    if (!w) return;
    if (this.ready) w.load(this.garment);
    w.dispatch('pattern.snap', { point: [0, 0] });
    w.dispatch('pattern.measure', {});
  }

  refresh(): Promise<void> {
    if (this.holding > 0) { this.held = true; return Promise.resolve(); }
    if (this.inflight) { this.again = true; return this.inflight; }
    this.inflight = this.pull().finally(() => { this.inflight = null; });
    return this.inflight;
  }

  private async pullSew(g: Garment): Promise<SewSnapshot> {
    const raw = g as Garment & { seams?: Seam[]; tacks?: Tack[] };
    const seams = raw.seams ?? [];
    const tacks = raw.tacks ?? [];
    const info = new Map<number, SeamInfo>();
    const bad = new Map<number, number>();
    if (seams.length) {
      try {
        const [list, check] = await Promise.all([
          this.engine.call<{ seams: SeamInfo[] }>('sew.list', {}),
          this.engine.call<{ mismatched: Array<{ id: number; mismatch: number }> }>('sew.check', {}),
        ]);
        for (const s of list.seams) info.set(s.id, s);
        for (const m of check.mismatched) bad.set(m.id, m.mismatch);
      } catch { /* lengths stay unknown until the next refresh */ }
    }
    return { seams, tacks, info, bad };
  }

  private async pull(): Promise<void> {
    do {
      this.again = false;
      let next: Garment;
      try { next = await this.engine.call<Garment>('pattern.get', {}); } catch { return; }
      const sew = await this.pullSew(next);
      if (this.holding > 0) { this.held = true; return; }
      this.garment = next;
      this.sew = sew;
      this.ready = true;
      this.wasm?.load(next);
      this.emit();
    } while (this.again);
  }

  beginPreview(label: string): void {
    this.holding++;
    this.wasm?.dispatch('pattern.begin_drag', { label });
  }

  preview(name: string, args: Record<string, unknown>, panels: number[]): boolean {
    const w = this.wasm;
    if (!w) return false;
    const r = w.dispatch(name, args);
    if (!r.ok) return false;
    const fresh = this.garment.panels.slice();
    const ids = new Set(panels);
    for (const p of fresh) if (p.link && ids.has(p.link.source)) ids.add(p.id);
    for (const id of ids) {
      const g = w.dispatch<Garment>('pattern.get', { panel: id });
      if (!g.ok || !g.result.panels[0]) continue;
      const at = fresh.findIndex((p) => p.id === id);
      if (at >= 0) fresh[at] = g.result.panels[0];
    }
    this.garment = { ...this.garment, panels: fresh };
    this.emit();
    return true;
  }

  endPreview(): void {
    this.wasm?.dispatch('pattern.end_drag', {});
    this.holding = Math.max(0, this.holding - 1);
  }

  async settle(): Promise<void> {
    if (this.holding > 0) return;
    this.held = false;
    await this.refresh();
  }

  async cancelPreview(): Promise<void> {
    this.endPreview();
    await this.settle();
  }

  prime(panels: number[], name?: string, args?: Record<string, unknown>): void {
    if (name) this.preview(name, args ?? {}, panels);
    else {
      for (const id of panels) {
        const g = this.wasm?.dispatch<Garment>('pattern.get', { panel: id });
        if (g && g.ok && g.result.panels[0]) geoOf(g.result.panels[0]);
      }
    }
    for (const id of panels) {
      const p = this.panel(id);
      if (p) geoOf(p);
    }
    this.wasm?.dispatch('pattern.snap', { point: [0, 0], targets: ['point', 'segment', 'grid'], tolerance: 1, grid: 10 });
  }

  discardPreview(): void {
    this.endPreview();
    if (this.held && this.holding === 0) void this.settle();
  }

  beginTrial(): void {
    this.holding++;
    this.trialBase = this.garment;
    this.trialKey = new Map(this.garment.panels.map((p) => [p.id, JSON.stringify(p)]));
  }

  trial(name: string, args: Record<string, unknown>): string | null {
    const w = this.wasm;
    const base = this.trialBase;
    if (!w || !base) return 'unavailable';
    w.load(base);
    const r = w.dispatch(name, args);
    if (!r.ok) {
      if (this.garment !== base) { this.garment = base; this.emit(); }
      return r.error.message;
    }
    const g = w.dispatch<Garment>('pattern.get', {});
    if (!g.ok) return g.error.message;
    const old = new Map(base.panels.map((p) => [p.id, p]));
    const panels = g.result.panels.map((p) => {
      const o = old.get(p.id);
      return o && this.trialKey.get(p.id) === JSON.stringify(p) ? o : p;
    });
    this.garment = { ...g.result, panels };
    this.emit();
    return null;
  }

  endTrial(): void {
    this.trialBase = null;
    this.holding = Math.max(0, this.holding - 1);
  }

  async commitTrial(steps: Step[], label: string): Promise<boolean> {
    const done = this.commit(steps, label);
    this.endTrial();
    return done;
  }

  async cancelTrial(): Promise<void> {
    this.endTrial();
    await this.settle();
  }

  async commit(steps: Step[], label: string): Promise<boolean> {
    let ok = true;
    this.holding++;
    try {
      await this.engine.call('pattern.begin_drag', { label });
      try {
        for (const [name, args] of steps) await this.engine.call(name, args);
      } finally {
        await this.engine.call('pattern.end_drag', {});
      }
    } catch (err) {
      ok = false;
      this.fail(err instanceof Error ? err.message : String(err));
    } finally {
      this.holding = Math.max(0, this.holding - 1);
    }
    await this.settle();
    return ok;
  }

  async commitPreview(steps: Step[], label: string): Promise<boolean> {
    const done = this.commit(steps, label);
    this.endPreview();
    return done;
  }

  async run<T>(name: string, args: Record<string, unknown>): Promise<T | undefined> {
    try {
      const r = await this.engine.call<T>(name, args);
      await this.refresh();
      return r;
    } catch (err) {
      this.fail(err instanceof Error ? err.message : String(err));
      await this.refresh();
      return undefined;
    }
  }

  measure(id: number): PanelMeasure | null {
    const w = this.wasm;
    if (!w) return null;
    const r = w.dispatch<PanelMeasure>('pattern.measure', { panel: id });
    return r.ok ? r.result : null;
  }
}
