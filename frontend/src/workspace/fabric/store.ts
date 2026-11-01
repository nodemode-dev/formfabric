import type { EngineApi } from '../../shell/deps';
import type { FabricInfo, FabricList, PresetsReply } from './types';

const EMPTY: FabricList = { fabrics: [], colorways: [], default_fabric: null, active_colorway: null };

export type Outcome<T> = { ok: true; result: T } | { ok: false; message: string };

export class FabricStore {
  list: FabricList = EMPTY;
  presets: PresetsReply | null = null;
  loaded = false;
  stale = true;
  private readonly listeners = new Set<() => void>();
  private inflight: Promise<void> | null = null;
  private again = false;

  constructor(private readonly engine: EngineApi, private readonly fail: (message: string) => void) {}

  onChange(cb: () => void): () => void {
    this.listeners.add(cb);
    return () => { this.listeners.delete(cb); };
  }

  private emit(): void {
    for (const cb of this.listeners) cb();
  }

  fabric(id: number | null): FabricInfo | undefined {
    return id === null ? undefined : this.list.fabrics.find((f) => f.id === id);
  }

  refresh(): Promise<void> {
    this.stale = false;
    if (this.inflight) { this.again = true; return this.inflight; }
    this.inflight = this.pull().finally(() => { this.inflight = null; });
    return this.inflight;
  }

  private async pull(): Promise<void> {
    do {
      this.again = false;
      try {
        const next = await this.engine.call<FabricList>('fabric.list', {});
        this.list = next;
        this.loaded = true;
        this.emit();
      } catch { return; }
    } while (this.again);
  }

  async loadPresets(): Promise<void> {
    if (this.presets) return;
    try {
      this.presets = await this.engine.call<PresetsReply>('fabric.presets', {});
      this.emit();
    } catch { return; }
  }

  async attempt<T>(name: string, args: Record<string, unknown>): Promise<Outcome<T>> {
    try {
      const result = await this.engine.call<T>(name, args);
      void this.refresh();
      return { ok: true, result };
    } catch (err) {
      void this.refresh();
      return { ok: false, message: err instanceof Error ? err.message : String(err) };
    }
  }

  async run<T>(name: string, args: Record<string, unknown>): Promise<T | undefined> {
    const r = await this.attempt<T>(name, args);
    if (r.ok) return r.result;
    this.fail(r.message);
    return undefined;
  }

  async preview<T>(name: string, args: Record<string, unknown>): Promise<Outcome<T>> {
    try {
      return { ok: true, result: await this.engine.call<T>(name, args) };
    } catch (err) {
      return { ok: false, message: err instanceof Error ? err.message : String(err) };
    }
  }
}
