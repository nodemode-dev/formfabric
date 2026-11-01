import type { EngineApi } from '../../shell/deps';
import { EMPTY_RETOPO, EMPTY_UV, type AnimRange, type FormatInfo, type Guide, type JobInfo, type Metric, type OptKind, type OptSpec, type RetopoState, type UvIslandInfo, type UvState, type Vec } from './types';

type Raw = Record<string, unknown>;

const isObj = (v: unknown): v is Raw => typeof v === 'object' && v !== null && !Array.isArray(v);
const num = (v: unknown, d: number): number => (typeof v === 'number' && Number.isFinite(v) ? v : d);
const str = (v: unknown, d = ''): string => (typeof v === 'string' ? v : d);
const vec = (v: unknown, d: Vec = [0, 0]): Vec => (Array.isArray(v) && v.length >= 2 && typeof v[0] === 'number' && typeof v[1] === 'number' ? [v[0], v[1]] : d);

const ACRONYM = new Set(['uv', 'ao', 'fps', 'lod', 'usd', 'obj', 'fbx', 'gltf', 'glb', 'abc', 'mdd', 'pc2', 'hdri']);
const UNIT = new Set(['mm', 'cm', 'm', 'in', 'ft']);

export const humanize = (key: string): string => {
  const words = key.replace(/[_.]+/g, ' ').trim().split(' ').filter(Boolean);
  if (!words.length) return key;
  const out = words.map((w) => (ACRONYM.has(w.toLowerCase()) ? w.toUpperCase() : w));
  const first = out[0];
  if (!ACRONYM.has(first.toLowerCase()) && !UNIT.has(first.toLowerCase())) out[0] = first[0].toUpperCase() + first.slice(1);
  return out.join(' ');
};

function choicesOf(raw: unknown): Array<[string, string]> {
  if (!Array.isArray(raw)) return [];
  const out: Array<[string, string]> = [];
  for (const c of raw) {
    if (typeof c === 'string' || typeof c === 'number') out.push([String(c), humanize(String(c))]);
    else if (isObj(c)) {
      const v = c.value ?? c.id ?? c.key;
      if (typeof v === 'string' || typeof v === 'number') out.push([String(v), str(c.label, humanize(String(v)))]);
    }
  }
  return out;
}

function kindOf(key: string, spec: Raw, choices: unknown[]): OptKind {
  const type = Array.isArray(spec.type) ? str(spec.type.find((x) => x !== 'null')) : str(spec.type);
  if (choices.length) return 'enum';
  if (type === 'boolean') return 'boolean';
  if (type === 'integer') return 'integer';
  if (type === 'number') return 'number';
  if (type === 'array' || key === 'range') return 'pair';
  return 'string';
}

function optOf(key: string, raw: unknown): OptSpec {
  const spec = isObj(raw) ? raw : {};
  const list = (spec.enum ?? spec.choices ?? spec.values) as unknown;
  const choices = choicesOf(list);
  const kind = kindOf(key, spec, choices);
  const lo = spec.minimum ?? spec.min;
  const hi = spec.maximum ?? spec.max;
  return {
    key,
    kind,
    label: str(spec.label ?? spec.title, humanize(key)),
    hint: str(spec.description),
    choices,
    min: typeof lo === 'number' ? lo : null,
    max: typeof hi === 'number' ? hi : null,
    def: spec.default ?? spec.def ?? (kind === 'boolean' ? false : kind === 'number' || kind === 'integer' ? 0 : kind === 'pair' ? [0, 0] : kind === 'enum' ? choices[0]?.[0] : ''),
  };
}

function optsOf(raw: unknown): OptSpec[] {
  if (Array.isArray(raw)) {
    const out: OptSpec[] = [];
    for (const item of raw) {
      if (!isObj(item)) continue;
      const key = str(item.key ?? item.id ?? item.name);
      if (key) out.push(optOf(key, item));
    }
    return out;
  }
  if (!isObj(raw)) return [];
  const props = isObj(raw.properties) ? raw.properties : raw;
  return Object.entries(props).filter(([k]) => k !== 'type' && k !== 'required' && k !== 'description').map(([k, v]) => optOf(k, v));
}

function formatOf(raw: unknown, hint?: string): FormatInfo | null {
  if (typeof raw === 'string') return { id: raw, label: raw.toUpperCase(), ext: raw, opts: [] };
  if (!isObj(raw)) return null;
  const id = str(raw.id ?? raw.format ?? raw.name ?? hint);
  if (!id) return null;
  return {
    id,
    label: str(raw.label ?? raw.title, id.toUpperCase()),
    ext: str(raw.ext ?? raw.extension, id).replace(/^\./, ''),
    opts: optsOf(raw.options ?? raw.option_schema ?? raw.schema ?? raw.inputSchema),
  };
}

export function parseFormats(raw: unknown): FormatInfo[] {
  const list = isObj(raw) ? (raw.formats ?? raw.items ?? raw) : raw;
  if (Array.isArray(list)) return list.map((f) => formatOf(f)).filter((f): f is FormatInfo => !!f);
  if (isObj(list)) return Object.entries(list).map(([k, v]) => formatOf(v, k)).filter((f): f is FormatInfo => !!f);
  return [];
}

const LOOP_KEYS = ['outline', 'polygon', 'loops', 'uv', 'points'];

function loopsOf(raw: Raw): Vec[][] {
  for (const k of LOOP_KEYS) {
    const v = raw[k];
    if (!Array.isArray(v) || !v.length) continue;
    if (Array.isArray(v[0]) && typeof v[0][0] === 'number') return [v.map((p) => vec(p))];
    if (Array.isArray(v[0]) && Array.isArray(v[0][0])) return v.map((l) => (l as unknown[]).map((p) => vec(p)));
    if (isObj(v[0]) && Array.isArray(v[0].points)) return v.map((l) => ((l as Raw).points as unknown[]).map((p) => vec(p)));
  }
  const b = raw.bbox ?? raw.bounds;
  if (Array.isArray(b) && b.length >= 4 && b.every((x) => typeof x === 'number')) {
    const [x0, y0, x1, y1] = b as number[];
    return [[[x0, y0], [x1, y0], [x1, y1], [x0, y1]]];
  }
  return [];
}

export function parseUv(raw: unknown): UvState {
  if (!isObj(raw)) return EMPTY_UV;
  const list = Array.isArray(raw.islands) ? raw.islands : [];
  const islands: UvIslandInfo[] = [];
  for (const it of list) {
    if (!isObj(it)) continue;
    islands.push({
      panel: num(it.panel, 0),
      name: str(it.name, `#${num(it.panel, 0)}`),
      offset: vec(it.offset),
      rotation: num(it.rotation, 0),
      scale: num(it.scale, 1),
      tile: num(it.tile, 0),
      loops: loopsOf(it),
    });
  }
  const t = raw.tiles;
  let tiles = 1;
  let cols = 0;
  if (typeof t === 'number') tiles = Math.max(1, Math.round(t));
  else if (Array.isArray(t) && t.length >= 2) { cols = Math.max(1, Math.round(num(t[0], 1))); tiles = cols * Math.max(1, Math.round(num(t[1], 1))); }
  else if (isObj(t)) { cols = Math.max(1, Math.round(num(t.u ?? t.x, 1))); tiles = cols * Math.max(1, Math.round(num(t.v ?? t.y, 1))); }
  const used = islands.reduce((m, i) => Math.max(m, i.tile + 1), 1);
  tiles = Math.max(tiles, used);
  return { islands, tiles, cols: cols || Math.min(tiles, 10), padding: num(raw.padding ?? raw.uv_padding, 0.01) };
}

function metricsOf(raw: unknown): Metric[] {
  const out: Metric[] = [];
  const add = (key: string, v: unknown): void => {
    if (typeof v === 'number') out.push({ key, label: humanize(key), value: Number.isInteger(v) ? String(v) : String(Number(v.toFixed(3))) });
    else if (typeof v === 'string' || typeof v === 'boolean') out.push({ key, label: humanize(key), value: String(v) });
  };
  if (Array.isArray(raw)) {
    for (const m of raw) if (isObj(m)) add(str(m.key ?? m.name ?? m.id), m.value);
  } else if (isObj(raw)) for (const [k, v] of Object.entries(raw)) add(k, v);
  return out.filter((m) => m.key);
}

function guidesOf(raw: unknown): Guide[] {
  if (!Array.isArray(raw)) return [];
  const out: Guide[] = [];
  for (const g of raw) {
    if (!isObj(g) || !Array.isArray(g.points)) continue;
    out.push({ panel: num(g.panel, 0), points: g.points.map((p) => vec(p)) });
  }
  return out;
}

export function parseRetopo(raw: unknown): RetopoState {
  if (!isObj(raw)) return EMPTY_RETOPO;
  const inner = isObj(raw.retopo) ? raw.retopo : raw;
  return {
    mesh: str(raw.mesh, 'triangle'),
    density: num(raw.density ?? raw.quad_density, 0.5),
    guides: guidesOf(inner.guides ?? raw.guides),
    metrics: metricsOf(raw.metrics ?? raw.quality ?? raw.stats ?? inner.metrics),
  };
}

export function parseJob(raw: unknown): JobInfo {
  if (!isObj(raw)) return { running: false, progress: null, label: '' };
  const p = raw.progress ?? raw.fraction ?? raw.percent;
  let progress: number | null = null;
  if (typeof p === 'number') progress = p > 1 ? Math.min(1, p / 100) : Math.max(0, p);
  const running = raw.running === true || raw.active === true || raw.busy === true || raw.state === 'running';
  return { running, progress, label: str(raw.label ?? raw.message ?? raw.stage ?? raw.name) };
}

export function parseRange(raw: unknown): AnimRange | null {
  if (!isObj(raw)) return null;
  const caches = Array.isArray(raw.caches) ? raw.caches.filter(isObj) : [];
  const pick = caches.find((c) => c.id === raw.active_cache) ?? caches.find((c) => c.visible !== false) ?? null;
  if (pick) {
    const start = num(pick.start, 0);
    const frames = num(pick.frames, 0);
    if (frames > 0) return { start, end: start + frames - 1, fromCache: true };
  }
  if (typeof raw.start === 'number' && typeof raw.end === 'number') return { start: raw.start, end: raw.end, fromCache: false };
  return null;
}

export class OutputStore {
  private caps: Set<string> | null = null;
  formats: FormatInfo[] = [];
  uv: UvState = EMPTY_UV;
  retopo: RetopoState = EMPTY_RETOPO;
  range: AnimRange | null = null;
  private readonly listeners = new Set<() => void>();
  private uvFlight: Promise<void> | null = null;
  private uvAgain = false;
  private retopoFlight: Promise<void> | null = null;
  private retopoAgain = false;

  constructor(private readonly engine: EngineApi, private readonly fail: (message: string) => void) {}

  onChange(cb: () => void): () => void {
    this.listeners.add(cb);
    return () => { this.listeners.delete(cb); };
  }

  private emit(): void {
    for (const cb of this.listeners) cb();
  }

  has(id: string): boolean {
    return this.caps?.has(id) ?? false;
  }

  get known(): boolean {
    return this.caps !== null;
  }

  async loadCaps(): Promise<boolean> {
    try {
      const cat = await this.engine.catalogue();
      this.caps = new Set(cat.commands.map((c) => c.id));
      this.emit();
      return true;
    } catch { return false; }
  }

  call<T>(name: string, args: object = {}): Promise<T> {
    return this.engine.call<T>(name, args);
  }

  async attempt<T>(name: string, args: object = {}): Promise<{ ok: true; result: T } | { ok: false; message: string }> {
    try { return { ok: true, result: await this.engine.call<T>(name, args) }; } catch (err) { return { ok: false, message: err instanceof Error ? err.message : String(err) }; }
  }

  async run<T>(name: string, args: object = {}): Promise<T | undefined> {
    const r = await this.attempt<T>(name, args);
    if (r.ok) return r.result;
    this.fail(r.message);
    return undefined;
  }

  async loadFormats(): Promise<void> {
    if (!this.has('export.formats')) return;
    const r = await this.attempt<unknown>('export.formats', {});
    if (!r.ok) return;
    const next = parseFormats(r.result);
    if (JSON.stringify(next) === JSON.stringify(this.formats)) return;
    this.formats = next;
    this.emit();
  }

  async loadRange(): Promise<void> {
    if (!this.has('anim.get')) { this.range = null; return; }
    const r = await this.attempt<unknown>('anim.get', {});
    this.range = r.ok ? parseRange(r.result) : null;
  }

  loadUv(): Promise<void> {
    if (!this.has('uv.get')) return Promise.resolve();
    this.uvAgain = true;
    if (this.uvFlight) return this.uvFlight;
    this.uvFlight = (async () => {
      while (this.uvAgain) {
        this.uvAgain = false;
        const r = await this.attempt<unknown>('uv.get', {});
        if (r.ok) { this.uv = parseUv(r.result); this.emit(); }
      }
    })().finally(() => { this.uvFlight = null; });
    return this.uvFlight;
  }

  loadRetopo(): Promise<void> {
    if (!this.has('retopo.get')) return Promise.resolve();
    this.retopoAgain = true;
    if (this.retopoFlight) return this.retopoFlight;
    this.retopoFlight = (async () => {
      while (this.retopoAgain) {
        this.retopoAgain = false;
        const r = await this.attempt<unknown>('retopo.get', {});
        if (r.ok) { this.retopo = parseRetopo(r.result); this.emit(); }
      }
    })().finally(() => { this.retopoFlight = null; });
    return this.retopoFlight;
  }

  async job(): Promise<JobInfo | null> {
    if (!this.has('job.state')) return null;
    const r = await this.attempt<unknown>('job.state', {});
    return r.ok ? parseJob(r.result) : null;
  }
}
