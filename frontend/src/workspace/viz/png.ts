const SIGNATURE = [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];

let table: Uint32Array | null = null;

function crc(bytes: Uint8Array): number {
  if (!table) {
    table = new Uint32Array(256);
    for (let n = 0; n < 256; n++) {
      let c = n;
      for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
      table[n] = c >>> 0;
    }
  }
  let c = 0xffffffff;
  for (let i = 0; i < bytes.length; i++) c = table[(c ^ bytes[i]) & 255] ^ (c >>> 8);
  return (c ^ 0xffffffff) >>> 0;
}

export const pixelsPerMetre = (ppi: number): number => Math.round(ppi / 0.0254);

export function withPpi(png: Uint8Array, ppi: number): Uint8Array<ArrayBuffer> {
  if (png.length < 33 || SIGNATURE.some((b, i) => png[i] !== b)) return png.slice();
  const view = new DataView(png.buffer, png.byteOffset, png.byteLength);
  const ihdrEnd = 8 + 12 + view.getUint32(8);
  const chunk = new Uint8Array(21);
  const cv = new DataView(chunk.buffer);
  cv.setUint32(0, 9);
  chunk.set([0x70, 0x48, 0x59, 0x73], 4);
  const ppm = pixelsPerMetre(ppi);
  cv.setUint32(8, ppm);
  cv.setUint32(12, ppm);
  chunk[16] = 1;
  cv.setUint32(17, crc(chunk.subarray(4, 17)));
  const parts: Uint8Array[] = [png.subarray(0, ihdrEnd)];
  let at = ihdrEnd;
  while (at + 12 <= png.length) {
    const len = view.getUint32(at);
    const end = at + 12 + len;
    const type = String.fromCharCode(png[at + 4], png[at + 5], png[at + 6], png[at + 7]);
    if (type !== 'pHYs') parts.push(png.subarray(at, Math.min(end, png.length)));
    at = end;
  }
  parts.splice(1, 0, chunk);
  const out = new Uint8Array(parts.reduce((n, p) => n + p.length, 0));
  let o = 0;
  for (const p of parts) { out.set(p, o); o += p.length; }
  return out;
}
