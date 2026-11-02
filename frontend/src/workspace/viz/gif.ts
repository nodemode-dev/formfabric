const R = 6;
const G = 7;
const B = 6;
const BAYER = [0, 8, 2, 10, 12, 4, 14, 6, 3, 11, 1, 9, 15, 7, 13, 5];

function palette(): Uint8Array {
  const p = new Uint8Array(768);
  let n = 1;
  for (let r = 0; r < R; r++) for (let g = 0; g < G; g++) for (let b = 0; b < B; b++) {
    p[n * 3] = Math.round((r * 255) / (R - 1));
    p[n * 3 + 1] = Math.round((g * 255) / (G - 1));
    p[n * 3 + 2] = Math.round((b * 255) / (B - 1));
    n++;
  }
  return p;
}

function indices(data: Uint8ClampedArray, w: number, h: number, alpha: boolean): Uint8Array {
  const out = new Uint8Array(w * h);
  for (let y = 0; y < h; y++) {
    for (let x = 0; x < w; x++) {
      const i = y * w + x;
      const k = i * 4;
      if (alpha && data[k + 3] < 128) { out[i] = 0; continue; }
      const d = (BAYER[(y & 3) * 4 + (x & 3)] / 16 - 0.5) * 0.9;
      const r = Math.max(0, Math.min(R - 1, Math.round((data[k] / 255) * (R - 1) + d)));
      const g = Math.max(0, Math.min(G - 1, Math.round((data[k + 1] / 255) * (G - 1) + d)));
      const b = Math.max(0, Math.min(B - 1, Math.round((data[k + 2] / 255) * (B - 1) + d)));
      out[i] = 1 + (r * G + g) * B + b;
    }
  }
  return out;
}

function lzw(px: Uint8Array, minBits: number): number[] {
  const clear = 1 << minBits;
  const eoi = clear + 1;
  const out: number[] = [];
  let size = minBits + 1;
  let next = eoi + 1;
  let cur = 0;
  let bits = 0;
  let table = new Map<number, number>();
  const emit = (code: number): void => {
    cur |= code << bits;
    bits += size;
    while (bits >= 8) { out.push(cur & 255); cur >>>= 8; bits -= 8; }
  };
  emit(clear);
  let prefix = px[0];
  for (let i = 1; i < px.length; i++) {
    const k = px[i];
    const key = (prefix << 8) | k;
    const hit = table.get(key);
    if (hit !== undefined) { prefix = hit; continue; }
    emit(prefix);
    if (next === 4096) {
      emit(clear);
      next = eoi + 1;
      size = minBits + 1;
      table = new Map();
    } else {
      if (next >= 1 << size) size++;
      table.set(key, next++);
    }
    prefix = k;
  }
  emit(prefix);
  emit(eoi);
  if (bits > 0) out.push(cur & 255);
  return out;
}

class Sink {
  private buf = new Uint8Array(1 << 20);
  private n = 0;

  push(...values: number[]): void {
    for (const v of values) this.put(v);
  }

  put(v: number): void {
    if (this.n === this.buf.length) {
      const grown = new Uint8Array(this.buf.length * 2);
      grown.set(this.buf);
      this.buf = grown;
    }
    this.buf[this.n++] = v & 255;
  }

  done(): Uint8Array<ArrayBuffer> {
    return this.buf.slice(0, this.n);
  }
}

export function encodeGif(frames: ImageData[], w: number, h: number, delayCs: number, alpha: boolean): Uint8Array<ArrayBuffer> {
  const out = new Sink();
  const u16 = (v: number): void => out.push(v, v >> 8);
  for (const c of 'GIF89a') out.put(c.charCodeAt(0));
  u16(w);
  u16(h);
  out.push(0xf7, 0, 0);
  for (const v of palette()) out.put(v);
  out.push(0x21, 0xff, 0x0b);
  for (const c of 'NETSCAPE2.0') out.put(c.charCodeAt(0));
  out.push(3, 1, 0, 0, 0);
  for (const f of frames) {
    out.push(0x21, 0xf9, 4, (alpha ? 2 << 2 : 1 << 2) | (alpha ? 1 : 0), delayCs, delayCs >> 8, 0, 0);
    out.put(0x2c);
    u16(0);
    u16(0);
    u16(w);
    u16(h);
    out.push(0, 8);
    const data = lzw(indices(f.data, w, h, alpha), 8);
    for (let i = 0; i < data.length; i += 255) {
      const n = Math.min(255, data.length - i);
      out.put(n);
      for (let j = 0; j < n; j++) out.put(data[i + j]);
    }
    out.put(0);
  }
  out.put(0x3b);
  return out.done();
}
