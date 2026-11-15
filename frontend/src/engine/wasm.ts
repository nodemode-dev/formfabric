export type WasmReply<T> = { ok: true; result: T } | { ok: false; error: { code: string; message: string } };

interface Exports {
  memory: WebAssembly.Memory;
  ffb_alloc(len: number): number;
  ffb_free(ptr: number, len: number): void;
  ffb_dispatch(ptr: number, len: number): number;
  ffb_load(ptr: number, len: number): number;
}

const encoder = new TextEncoder();
const decoder = new TextDecoder();

export class WasmEngine {
  constructor(private readonly x: Exports) {}

  stopped = false;

  private send(fn: (ptr: number, len: number) => number, text: string): unknown {
    const gone = { ok: false, error: { code: 'internal', message: 'the preview engine stopped' } };
    if (this.stopped) return gone;
    try {
      const bytes = encoder.encode(text);
      const ptr = this.x.ffb_alloc(bytes.length);
      new Uint8Array(this.x.memory.buffer, ptr, bytes.length).set(bytes);
      const out = fn(ptr, bytes.length);
      this.x.ffb_free(ptr, bytes.length);
      const len = new DataView(this.x.memory.buffer).getUint32(out, true);
      const json = decoder.decode(new Uint8Array(this.x.memory.buffer, out + 4, len));
      this.x.ffb_free(out, 4 + len);
      return JSON.parse(json);
    } catch {
      this.stopped = true;
      return gone;
    }
  }

  load(document: unknown): WasmReply<{ panels: number }> {
    return this.send((p, n) => this.x.ffb_load(p, n), JSON.stringify(document)) as WasmReply<{ panels: number }>;
  }

  dispatch<T>(name: string, args: object = {}): WasmReply<T> {
    return this.send((p, n) => this.x.ffb_dispatch(p, n), JSON.stringify({ name, args })) as WasmReply<T>;
  }
}

let pending: Promise<WasmEngine | null> | null = null;

export function loadWasm(url = '/engine/ffb_wasm.wasm'): Promise<WasmEngine | null> {
  pending ??= (async () => {
    try {
      const response = await fetch(url);
      if (!response.ok) return null;
      const module = await WebAssembly.compile(await response.arrayBuffer());
      const instance = await WebAssembly.instantiate(module, {});
      return new WasmEngine(instance.exports as unknown as Exports);
    } catch {
      return null;
    }
  })();
  return pending;
}
