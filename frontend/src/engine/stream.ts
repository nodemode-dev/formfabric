const MAGIC = 0x53424646;
const VERSION = 1;
const HEADER_LEN = 44;
const KIND_TOPOLOGY = 1;
const KIND_POSITIONS = 2;

export interface TopologyFrame {
  kind: 'topology';
  frame: number;
  object: number;
  vertexCount: number;
  indices: Uint32Array;
  panelIds: Uint16Array;
}

export interface PositionsFrame {
  kind: 'positions';
  frame: number;
  object: number;
  vertexCount: number;
  positions: Float32Array;
}

export interface ScalarsFrame {
  kind: 'scalars';
  frame: number;
  object: number;
  vertexCount: number;
  values: Float32Array;
  min: number;
  max: number;
  map: number;
}

export type StreamFrame = TopologyFrame | PositionsFrame | ScalarsFrame;

export type KindDecoder = (buffer: ArrayBuffer, view: DataView, head: { frame: number; object: number; vertexCount: number }) => StreamFrame;

const decoders = new Map<number, KindDecoder>();

export function registerKind(kind: number, decoder: KindDecoder): void {
  decoders.set(kind, decoder);
}
export type StreamCallback = (frame: StreamFrame) => void;
export type StreamState = 'idle' | 'connecting' | 'open' | 'closed';

interface StreamInfo { port: number; token?: string }

export function decodeFrame(buffer: ArrayBuffer): StreamFrame {
  if (buffer.byteLength < HEADER_LEN) throw new Error('stream frame shorter than its header');
  const view = new DataView(buffer);
  if (view.getUint32(0, true) !== MAGIC) throw new Error('stream frame magic is not FFBS');
  const version = view.getUint16(4, true);
  if (version !== VERSION) throw new Error(`unsupported stream frame version ${version}`);
  const kind = view.getUint16(6, true);
  const frame = view.getUint32(8, true);
  const object = view.getUint32(12, true);
  const vertexCount = view.getUint32(16, true);
  if (kind === KIND_POSITIONS) {
    if (buffer.byteLength !== HEADER_LEN + vertexCount * 6) throw new Error('positions frame length mismatch');
    const ox = view.getFloat32(20, true);
    const oy = view.getFloat32(24, true);
    const oz = view.getFloat32(28, true);
    const sx = view.getFloat32(32, true) / 65535;
    const sy = view.getFloat32(36, true) / 65535;
    const sz = view.getFloat32(40, true) / 65535;
    const positions = new Float32Array(vertexCount * 3);
    let at = HEADER_LEN;
    for (let i = 0; i < vertexCount; i++) {
      positions[i * 3] = ox + view.getUint16(at, true) * sx;
      positions[i * 3 + 1] = oy + view.getUint16(at + 2, true) * sy;
      positions[i * 3 + 2] = oz + view.getUint16(at + 4, true) * sz;
      at += 6;
    }
    return { kind: 'positions', frame, object, vertexCount, positions };
  }
  if (kind === KIND_TOPOLOGY) {
    if (buffer.byteLength < HEADER_LEN + 4) throw new Error('topology frame too short');
    const triangles = view.getUint32(HEADER_LEN, true);
    if (buffer.byteLength !== HEADER_LEN + 4 + triangles * 14) throw new Error('topology frame length mismatch');
    const indices = new Uint32Array(triangles * 3);
    let at = HEADER_LEN + 4;
    for (let i = 0; i < indices.length; i++) {
      const index = view.getUint32(at, true);
      if (index >= vertexCount) throw new Error('topology index out of range');
      indices[i] = index;
      at += 4;
    }
    const panelIds = new Uint16Array(triangles);
    for (let t = 0; t < triangles; t++) {
      panelIds[t] = view.getUint16(at, true);
      at += 2;
    }
    return { kind: 'topology', frame, object, vertexCount, indices, panelIds };
  }
  const extra = decoders.get(kind);
  if (extra) return extra(buffer, view, { frame, object, vertexCount });
  throw new Error(`unknown stream frame kind ${kind}`);
}

function engineBase(): string {
  const injected = (globalThis as unknown as { __FFB_ENGINE__?: string }).__FFB_ENGINE__;
  return injected ? injected.replace(/\/$/, '') : '';
}

export class EngineStream {
  private socket: WebSocket | null = null;
  private listeners = new Map<number, Set<StreamCallback>>();
  private topologies = new Map<number, TopologyFrame>();
  private closed = false;
  private stateListeners = new Set<(state: StreamState) => void>();
  private retryTimer = 0;
  private tries = 0;
  state: StreamState = 'idle';

  async connect(): Promise<void> {
    if (this.socket) return;
    this.closed = false;
    this.setState('connecting');
    const response = await fetch(`${engineBase()}/api/engine/stream`);
    if (!response.ok) {
      this.setState('closed');
      throw new Error(`stream info request failed with ${response.status}`);
    }
    const body = (await response.json()) as StreamInfo | { ok: true; result: StreamInfo };
    const info = 'result' in body ? body.result : body;
    if (!info.port || !info.token) {
      this.setState('closed');
      throw new Error('the engine stream server is not running');
    }
    if (this.closed || this.socket) return;
    const socket = new WebSocket(`ws://127.0.0.1:${info.port}/?token=${encodeURIComponent(info.token)}`);
    socket.binaryType = 'arraybuffer';
    socket.onopen = () => { this.tries = 0; this.setState('open'); };
    socket.onmessage = (event: MessageEvent) => {
      if (!(event.data instanceof ArrayBuffer)) return;
      let frame: StreamFrame;
      try { frame = decodeFrame(event.data); } catch { return; }
      if (frame.kind === 'topology') this.topologies.set(frame.object, frame);
      this.dispatch(frame);
    };
    socket.onclose = () => {
      if (this.socket === socket) this.socket = null;
      this.setState('closed');
      this.retry();
    };
    this.socket = socket;
  }

  private retry(): void {
    if (this.closed || this.retryTimer || this.listeners.size === 0 || this.tries >= 8) return;
    const wait = Math.min(8000, 500 * 2 ** this.tries++);
    this.retryTimer = window.setTimeout(() => { this.retryTimer = 0; this.connect().catch(() => this.retry()); }, wait);
  }

  subscribe(objectId: number, cb: StreamCallback): () => void {
    let set = this.listeners.get(objectId);
    if (!set) {
      set = new Set();
      this.listeners.set(objectId, set);
    }
    set.add(cb);
    const known = this.topologies.get(objectId);
    if (known) cb(known);
    return () => {
      const current = this.listeners.get(objectId);
      if (!current) return;
      current.delete(cb);
      if (current.size === 0) this.listeners.delete(objectId);
    };
  }

  onState(cb: (state: StreamState) => void): () => void {
    this.stateListeners.add(cb);
    return () => { this.stateListeners.delete(cb); };
  }

  close(): void {
    this.closed = true;
    window.clearTimeout(this.retryTimer);
    this.retryTimer = 0;
    const socket = this.socket;
    this.socket = null;
    if (socket) {
      socket.onopen = null;
      socket.onmessage = null;
      socket.onclose = null;
      socket.close(1000);
    }
    this.topologies.clear();
    this.setState('closed');
  }

  private dispatch(frame: StreamFrame): void {
    const set = this.listeners.get(frame.object);
    if (!set) return;
    for (const cb of set) cb(frame);
  }

  private setState(state: StreamState): void {
    if (this.state === state) return;
    this.state = state;
    for (const cb of this.stateListeners) cb(state);
  }
}

const shared = new EngineStream();

export function connectStream(): Promise<void> {
  return shared.connect();
}

export function subscribe(objectId: number, cb: StreamCallback): () => void {
  return shared.subscribe(objectId, cb);
}

export function closeStream(): void {
  shared.close();
}

export function onStreamState(cb: (state: StreamState) => void): () => void {
  return shared.onState(cb);
}
