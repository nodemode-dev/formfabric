import { call, catalogue, onEngineChanged, onEngineLink } from '../../engine/service';
import { connectStream, subscribe, type StreamFrame } from '../../engine/stream';
import { linear, lookOf } from './look';
import type { GarmentFull, MeshData, PanelFull } from './model';
import { placeLocal } from './place';
import type { PanelBuf, Scene } from './scene';
import { outlineMesh } from './triangulate';

const FETCH_DELAY_MS = 200;
const FETCH_PARALLEL = 4;
const STREAM_RETRY_MS = [1500, 3000, 6000, 12000, 24000];
const BODY_BASE = 1_000_000;
const BODY_SLOTS = 16;
const SKIN: [number, number, number] = [0.87, 0.7, 0.6];

function flip(indices: Uint32Array): Uint32Array {
  const out = new Uint32Array(indices.length);
  for (let i = 0; i < indices.length; i += 3) {
    out[i] = indices[i];
    out[i + 1] = indices[i + 2];
    out[i + 2] = indices[i + 1];
  }
  return out;
}

export function parseMesh(reply: unknown): MeshData | null {
  let o = reply as Record<string, unknown> | null;
  if (o && !o.positions) {
    const inner = (o.mesh ?? (Array.isArray(o.meshes) ? o.meshes[0] : undefined) ?? (Array.isArray(o.panels) ? o.panels[0] : undefined)) as Record<string, unknown> | undefined;
    if (inner) o = inner;
  }
  if (!o || !o.positions || !(o.triangles ?? o.indices)) return null;
  const rawPos = o.positions as unknown;
  const rawTri = (o.triangles ?? o.indices) as unknown;
  const nested = Array.isArray(rawTri) && Array.isArray(rawTri[0]);
  const tri = Uint32Array.from(nested ? (rawTri as number[][]).flat() : (rawTri as number[]));
  if (tri.length === 0 || tri.length % 3 !== 0) return null;
  let maxIndex = 0;
  for (let i = 0; i < tri.length; i++) if (tri[i] > maxIndex) maxIndex = tri[i];
  const n = maxIndex + 1;
  let flat: number[];
  let stride = 2;
  if (Array.isArray(rawPos) && Array.isArray(rawPos[0])) {
    stride = (rawPos[0] as number[]).length;
    flat = (rawPos as number[][]).flat();
  } else {
    flat = rawPos as number[];
    stride = flat.length === n * 3 && flat.length !== n * 2 ? 3 : 2;
  }
  if (flat.length < n * stride) return null;
  const local = new Float32Array(n * 2);
  for (let i = 0; i < n; i++) {
    local[i * 2] = flat[i * stride];
    local[i * 2 + 1] = flat[i * stride + 1];
  }
  return { local, indices: tri };
}

function sameLocal(a: Float32Array | null, b: Float32Array): boolean {
  if (!a || a.length !== b.length) return false;
  for (let i = 0; i < a.length; i++) if (a[i] !== b[i]) return false;
  return true;
}

export class MeshFeed {
  meshGet = false;
  arrangePlace = false;
  private readonly subs = new Map<number, () => void>();
  private readonly want = new Set<number>();
  private queue: number[] = [];
  private inflight = 0;
  private timer = 0;
  private garment: GarmentFull = { units: 'mm', panels: [] };
  private streamTries = 0;
  private streamTimer = 0;
  private streaming = false;
  private dirty = false;
  private bodyTimer = 0;
  private bodiesWatched = false;

  constructor(private readonly scene: Scene, private readonly changed: () => void, private readonly capsChanged: () => void) {
    onEngineChanged((e) => {
      if (e.name === 'view.units') return;
      if (/^(body|file|history)\./.test(e.name)) this.scheduleBodies();
      this.markAll();
      this.schedule();
    });
    onEngineLink((state) => {
      if (state !== 'up') return;
      this.scheduleBodies();
      void this.refreshCaps();
      this.openStream();
    });
  }

  async refreshCaps(): Promise<void> {
    try {
      const c = await catalogue(true);
      const has = (id: string): boolean => c.commands.some((x) => x.id === id);
      const mg = has('mesh.get');
      const ap = has('arrange.place');
      const changed = mg !== this.meshGet || ap !== this.arrangePlace;
      this.meshGet = mg;
      this.arrangePlace = ap;
      if (changed) this.capsChanged();
      if (mg) {
        this.markAll();
        this.schedule();
      }
    } catch { return; }
  }

  private openStream(): void {
    this.watchBodies();
    if (this.streaming) return;
    this.streaming = true;
    connectStream().then(() => { this.streamTries = 0; }, () => { this.streaming = false; this.retryStream(); });
  }

  private retryStream(): void {
    if (this.streamTimer || this.streamTries >= STREAM_RETRY_MS.length) return;
    const wait = STREAM_RETRY_MS[this.streamTries++];
    this.streamTimer = window.setTimeout(() => { this.streamTimer = 0; this.openStream(); }, wait);
  }

  private markAll(): void {
    this.dirty = true;
  }

  sync(g: GarmentFull): void {
    this.garment = g;
    const ids = new Set<number>();
    for (const p of g.panels) {
      ids.add(p.id);
      const created = !this.scene.get(p.id);
      const b = this.scene.ensure(p.id);
      if (created) {
        this.watch(p.id);
        this.want.add(p.id);
      }
      b.setLook(lookOf(g, p));
      if (b.source === 'stream' || b.source === 'feed') continue;
      const shapeKey = JSON.stringify([p.outline, p.inner, p.particle_distance, p.mirror]);
      const placeKey = JSON.stringify([p.position, p.rotation, p.mirror, p.place3d ?? null]);
      if (shapeKey !== b.shapeKey || !b.local) {
        b.shapeKey = shapeKey;
        b.placeKey = placeKey;
        const m = outlineMesh(p);
        if (m) this.apply(b, p, m, 'local');
        else b.ready = false;
      } else if (placeKey !== b.placeKey) {
        b.placeKey = placeKey;
        this.replace(b, p);
      }
    }
    for (const b of [...this.scene.list]) {
      if (ids.has(b.id) || b.source === 'feed' || b.kind === 'body') continue;
      this.subs.get(b.id)?.();
      this.subs.delete(b.id);
      this.want.delete(b.id);
      this.scene.remove(b.id);
    }
    this.scene.touch();
    if (this.meshGet && this.want.size) this.schedule();
    this.changed();
  }

  private scheduleBodies(): void {
    window.clearTimeout(this.bodyTimer);
    this.bodyTimer = window.setTimeout(() => void this.syncBodies(), FETCH_DELAY_MS);
  }

  private async syncBodies(): Promise<void> {
    let list: Array<{ object?: number; id?: number }> = [];
    try {
      list = ((await call<{ bodies?: Array<{ object?: number; id?: number }> }>('body.list')).bodies) ?? [];
    } catch { return; }
    const objects = new Set<number>();
    for (const item of list) objects.add(item.object ?? (item.id !== undefined ? BODY_BASE + item.id : -1));
    for (const b of [...this.scene.list]) {
      if (b.kind !== 'body' || objects.has(b.id)) continue;
      this.scene.remove(b.id);
    }
    this.scene.touch();
    this.changed();
  }

  private watchBodies(): void {
    if (this.bodiesWatched) return;
    this.bodiesWatched = true;
    for (let i = 1; i <= BODY_SLOTS; i++) {
      const object = BODY_BASE + i;
      this.subs.set(object, subscribe(object, (f) => this.bodyFrame(object, f)));
    }
  }

  private bodyFrame(id: number, f: StreamFrame): void {
    let b = this.scene.get(id);
    if (!b) {
      if (f.kind !== 'topology' || f.indices.length === 0) return;
      b = this.scene.ensure(id);
      b.kind = 'body';
      b.source = 'stream';
      b.front.set(linear(SKIN));
      b.back.set(linear(SKIN));
      b.rough = 0.62;
      b.sheen = 0.3;
    }
    if (f.kind === 'topology') {
      if (f.indices.length === 0) {
        b.pending = null;
        b.ready = false;
        this.scene.touch();
        this.changed();
        return;
      }
      b.pending = { indices: f.indices, vcount: f.vertexCount };
      return;
    }
    if (b.pending && b.pending.vcount === f.vertexCount) {
      this.scene.setTopology(b, b.pending.indices, b.pending.vcount);
      this.scene.setUv(b, null);
      b.pending = null;
    } else if (f.vertexCount !== b.vcount) return;
    this.scene.setPositions(b, f.positions);
    this.scene.touch();
    this.changed();
  }

  private watch(id: number): void {
    this.subs.set(id, subscribe(id, (f) => this.frame(id, f)));
    this.openStream();
  }

  private frame(id: number, f: StreamFrame): void {
    const b = this.scene.get(id);
    if (!b || b.source === 'feed') return;
    if (f.kind === 'topology') {
      if (f.indices.length === 0) return;
      b.pending = { indices: f.indices, vcount: f.vertexCount };
      return;
    }
    if (b.pending && b.pending.vcount === f.vertexCount) {
      this.scene.setTopology(b, b.pending.indices, b.pending.vcount);
      this.scene.setUv(b, b.local);
      b.pending = null;
      b.source = 'stream';
    } else if (b.source !== 'stream' || f.vertexCount !== b.vcount) return;
    this.scene.setPositions(b, f.positions);
    this.changed();
  }

  private apply(b: PanelBuf, p: PanelFull, m: MeshData, source: 'local' | 'mesh'): void {
    const indices = p.mirror ? flip(m.indices) : m.indices;
    const n = m.local.length >> 1;
    this.scene.setTopology(b, indices, n);
    b.local = m.local;
    this.scene.setUv(b, m.local);
    placeLocal(p, m.local, b.pos);
    this.scene.afterPositions(b);
    b.source = source;
  }

  private replace(b: PanelBuf, p: PanelFull): void {
    if (!b.local) return;
    placeLocal(p, b.local, b.pos);
    this.scene.afterPositions(b);
  }

  private schedule(): void {
    if (!this.meshGet) return;
    window.clearTimeout(this.timer);
    this.timer = window.setTimeout(() => this.flush(), FETCH_DELAY_MS);
  }

  private flush(): void {
    if (!this.meshGet) return;
    this.queue = this.dirty ? this.scene.list.map((b) => b.id) : [...this.want];
    this.dirty = false;
    this.want.clear();
    this.pump();
  }

  private pump(): void {
    while (this.inflight < FETCH_PARALLEL && this.queue.length) {
      const id = this.queue.shift() as number;
      const b = this.scene.get(id);
      if (!b || b.source === 'stream' || b.source === 'feed') continue;
      const key = b.shapeKey;
      this.inflight++;
      call<unknown>('mesh.get', { panel: id }).then((r) => {
        const cur = this.scene.get(id);
        if (!cur || cur.source === 'stream' || cur.source === 'feed' || cur.shapeKey !== key) return;
        const p = this.garment.panels.find((x) => x.id === id);
        const m = parseMesh(r);
        if (!p || !m) return;
        if (cur.source === 'mesh' && sameLocal(cur.local, m.local)) return;
        this.apply(cur, p, m, 'mesh');
        this.scene.touch();
        this.changed();
      }, () => undefined).finally(() => {
        this.inflight--;
        this.pump();
      });
    }
  }

  feed(id: number, positions: Float32Array, indices: Uint32Array): void {
    const b = this.scene.ensure(id);
    b.source = 'feed';
    this.scene.setTopology(b, indices, positions.length / 3);
    this.scene.setUv(b, null);
    this.scene.setPositions(b, positions);
    this.scene.touch();
    this.changed();
  }

  clearFed(): void {
    for (const b of [...this.scene.list]) if (b.source === 'feed') this.scene.remove(b.id);
    this.scene.touch();
    this.changed();
  }
}
