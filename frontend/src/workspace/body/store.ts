import type { EngineApi } from '../../shell/deps';
import type { BodyInfo, BodyList, JointInfo, JointsReply, MeasureReply, PlaceItem } from './types';

const EMPTY: BodyList = { bodies: [], library: [], poses: [] };

export type Outcome<T> = { ok: true; result: T } | { ok: false; message: string };

const asRecord = (v: unknown): Record<string, unknown> | null => (v && typeof v === 'object' && !Array.isArray(v) ? (v as Record<string, unknown>) : null);

export function placeItems(raw: unknown): PlaceItem[] {
  const root = asRecord(raw);
  const list = Array.isArray(raw) ? raw : Array.isArray(root?.points) ? root.points : [];
  const out: PlaceItem[] = [];
  list.forEach((item, i) => {
    if (typeof item === 'string') { out.push({ key: item, name: item }); return; }
    const r = asRecord(item);
    if (!r) return;
    const id = r.id ?? r.name ?? i;
    const name = typeof r.name === 'string' ? r.name : String(id);
    out.push({ key: String(id), name, offset: typeof r.offset === 'number' ? r.offset : undefined });
  });
  return out;
}

export class BodyStore {
  list: BodyList = EMPTY;
  selId: number | null = null;
  measures: Record<string, number> = {};
  joints: JointInfo[] = [];
  points: PlaceItem[] = [];
  commands = new Set<string>();
  schemas = new Map<string, Record<string, unknown>>();
  loaded = false;
  stale = true;
  readonly originals = new Map<number, Record<string, number>>();
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

  has(command: string): boolean {
    return this.commands.has(command);
  }

  props(command: string): Record<string, Record<string, unknown>> {
    const p = this.schemas.get(command)?.properties;
    return p && typeof p === 'object' ? (p as Record<string, Record<string, unknown>>) : {};
  }

  body(id: number | null): BodyInfo | undefined {
    return id === null ? undefined : this.list.bodies.find((b) => b.id === id);
  }

  selected(): BodyInfo | undefined {
    return this.body(this.selId) ?? this.list.bodies[0];
  }

  select(id: number | null): void {
    this.selId = id;
    void this.refresh();
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
        if (this.commands.size === 0) {
          const cat = await this.engine.catalogue();
          this.commands = new Set(cat.commands.map((c) => c.id));
          this.schemas = new Map(cat.commands.map((c) => [c.id, c.inputSchema as unknown as Record<string, unknown>]));
        }
        const next = await this.engine.call<BodyList>('body.list', {});
        const ids = new Set(next.bodies.map((b) => b.id));
        for (const id of [...this.originals.keys()]) if (!ids.has(id)) this.originals.delete(id);
        for (const b of next.bodies) if (!this.originals.has(b.id) && Object.keys(b.measurements).length > 0) this.originals.set(b.id, { ...b.measurements });
        this.list = next;
        const sel = this.selected();
        if (sel && this.selId !== sel.id) this.selId = sel.id;
        if (!sel) {
          this.selId = null;
          this.measures = {};
          this.joints = [];
          this.points = [];
        } else {
          const [m, j, p] = await Promise.all([
            this.engine.call<MeasureReply>('body.measure', { id: sel.id }).catch(() => null),
            this.engine.call<JointsReply>('body.joints', { id: sel.id }).catch(() => null),
            this.has('place.points') ? this.engine.call<unknown>('place.points', { body: sel.id }).catch(() => null) : Promise.resolve(null),
          ]);
          if (m) {
            this.measures = m.measurements;
            if (!this.originals.get(sel.id) || Object.keys(this.originals.get(sel.id) ?? {}).length === 0) this.originals.set(sel.id, { ...m.measurements });
          }
          if (j) this.joints = j.joints;
          this.points = p ? placeItems(p) : [];
        }
        this.loaded = true;
        this.emit();
      } catch { return; }
    } while (this.again);
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

  async read<T>(name: string, args: Record<string, unknown>): Promise<T | undefined> {
    try {
      return await this.engine.call<T>(name, args);
    } catch (err) {
      this.fail(err instanceof Error ? err.message : String(err));
      return undefined;
    }
  }
}
