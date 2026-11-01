import { call, onEngineChanged, onEngineLink } from '../../engine/service';
import type { Garment } from '../types';

export interface DocLike {
  garment: Garment;
  onChange(cb: () => void): () => void;
  run<T>(name: string, args: Record<string, unknown>): Promise<T | undefined>;
  refresh(): Promise<void>;
}

export class StandaloneDoc implements DocLike {
  garment: Garment = { units: 'mm', panels: [] };
  private readonly listeners = new Set<() => void>();
  private inflight: Promise<void> | null = null;
  private again = false;

  constructor() {
    onEngineChanged((e) => { if (e.name !== 'view.units') void this.refresh(); });
    onEngineLink((state) => { if (state === 'up') void this.refresh(); });
  }

  onChange(cb: () => void): () => void {
    this.listeners.add(cb);
    return () => { this.listeners.delete(cb); };
  }

  refresh(): Promise<void> {
    if (this.inflight) { this.again = true; return this.inflight; }
    this.inflight = this.pull().finally(() => { this.inflight = null; });
    return this.inflight;
  }

  private async pull(): Promise<void> {
    do {
      this.again = false;
      try { this.garment = await call<Garment>('pattern.get', {}); } catch { return; }
      for (const cb of this.listeners) cb();
    } while (this.again);
  }

  async run<T>(name: string, args: Record<string, unknown>): Promise<T | undefined> {
    try {
      const r = await call<T>(name, args);
      await this.refresh();
      return r;
    } catch {
      await this.refresh();
      return undefined;
    }
  }
}
