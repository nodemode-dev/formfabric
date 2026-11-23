import { describe, it, expect } from 'vitest';
import { bodyRows, parseAnim, parseJob, parseValue, rowsOf, showValue } from '../../../src/workspace/anim/types';
import { decodeScalars, NO_DATA } from '../../../src/workspace/view3d/scalars';
import { pixelsPerMetre, withPpi } from '../../../src/workspace/viz/png';
import { cssGradient, rampBytes } from '../../../src/workspace/viz/ramps';

function crc32(bytes: Uint8Array): number {
  let c = 0xffffffff;
  for (const b of bytes) {
    c ^= b;
    for (let k = 0; k < 8; k++) c = c & 1 ? (c >>> 1) ^ 0xedb88320 : c >>> 1;
  }
  return (c ^ 0xffffffff) >>> 0;
}

function chunk(type: string, data: number[]): number[] {
  const body = [...type].map((ch) => ch.charCodeAt(0)).concat(data);
  const out = new Uint8Array(8 + data.length + 4);
  const v = new DataView(out.buffer);
  v.setUint32(0, data.length);
  out.set(body, 4);
  v.setUint32(8 + data.length, crc32(new Uint8Array(body)));
  return Array.from(out);
}

function readChunks(png: Uint8Array): Array<{ type: string; data: Uint8Array; crcOk: boolean }> {
  const v = new DataView(png.buffer, png.byteOffset, png.byteLength);
  const out: Array<{ type: string; data: Uint8Array; crcOk: boolean }> = [];
  let at = 8;
  while (at + 12 <= png.length) {
    const len = v.getUint32(at);
    const type = String.fromCharCode(...png.subarray(at + 4, at + 8));
    out.push({ type, data: png.subarray(at + 8, at + 8 + len), crcOk: crc32(png.subarray(at + 4, at + 8 + len)) === v.getUint32(at + 8 + len) });
    at += 12 + len;
  }
  return out;
}

const SIG = [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];
const IHDR = chunk('IHDR', [0, 0, 0, 1, 0, 0, 0, 1, 8, 6, 0, 0, 0]);
const IDAT = chunk('IDAT', [1, 2, 3, 4, 5]);
const IEND = chunk('IEND', []);

function scalarFrame(min: number, max: number, map: number, component: number, q: number[], extra = 0): { buffer: ArrayBuffer; view: DataView } {
  const buffer = new ArrayBuffer(44 + q.length * 2 + extra);
  const view = new DataView(buffer);
  view.setFloat32(20, min, true);
  view.setFloat32(24, max, true);
  view.setFloat32(32, map, true);
  view.setFloat32(36, component, true);
  q.forEach((v, i) => view.setUint16(44 + i * 2, v, true));
  return { buffer, view };
}

describe('colour ramps', () => {
  it('a ramp starts and ends on the published end colours of viridis', () => {
    const b = rampBytes('viridis', 256);
    expect(b.length).toBe(256 * 4);
    expect(Array.from(b.subarray(0, 4))).toEqual([0x44, 0x01, 0x54, 255]);
    expect(Array.from(b.subarray(255 * 4))).toEqual([0xfd, 0xe7, 0x25, 255]);
  });

  it('every ramp texel is fully opaque', () => {
    const b = rampBytes('inferno', 64);
    for (let i = 0; i < 64; i++) expect(b[i * 4 + 3]).toBe(255);
  });

  it('the diverging ramp runs from blue at the low end to red at the high end', () => {
    const b = rampBytes('coolwarm', 32);
    expect(b[2]).toBeGreaterThan(b[0]);
    expect(b[31 * 4]).toBeGreaterThan(b[31 * 4 + 2]);
  });

  it('a texel halfway between two stops is the average of those stops', () => {
    const coarse = rampBytes('viridis', 10);
    const fine = rampBytes('viridis', 19);
    for (const c of [0, 2]) expect(fine[4 + c]).toBe((coarse[c] + coarse[4 + c]) / 2);
  });

  it('a finer ramp passes through every colour of a coarser one', () => {
    const coarse = rampBytes('inferno', 10);
    const fine = rampBytes('inferno', 19);
    for (let i = 0; i < 10; i++) expect(Array.from(fine.subarray(i * 8, i * 8 + 4))).toEqual(Array.from(coarse.subarray(i * 4, i * 4 + 4)));
  });

  it('sequential ramps get steadily brighter from low to high', () => {
    for (const name of ['viridis', 'inferno'] as const) {
      const b = rampBytes(name, 16);
      let last = -1;
      for (let i = 0; i < 16; i++) {
        const luma = 0.2126 * b[i * 4] + 0.7152 * b[i * 4 + 1] + 0.0722 * b[i * 4 + 2];
        expect(luma).toBeGreaterThan(last);
        last = luma;
      }
    }
  });

  it('the legend gradient runs left to right from 0% to 100% with evenly spaced stops', () => {
    const g = cssGradient('coolwarm');
    expect(g.startsWith('linear-gradient(to right, #3b4cc0 0%, ')).toBe(true);
    expect(g.endsWith('#b40426 100%)')).toBe(true);
    const pct = [...g.matchAll(/ (\d+)%/g)].map((m) => Number(m[1]));
    expect(pct).toEqual([0, 14, 29, 43, 57, 71, 86, 100]);
  });

  it('the legend gradient and the ramp texture agree on the end colours', () => {
    const g = cssGradient('inferno');
    const b = rampBytes('inferno', 8);
    const hex = (o: number): string => '#' + [b[o], b[o + 1], b[o + 2]].map((v) => v.toString(16).padStart(2, '0')).join('');
    expect(g).toContain(`${hex(0)} 0%`);
    expect(g).toContain(`${hex(28)} 100%`);
  });
});

describe('scalar map frames', () => {
  it('quantised values decode linearly between the frame minimum and maximum', () => {
    const { buffer, view } = scalarFrame(-2, 6, 1, 0, [0, 65534, 32767]);
    const f = decodeScalars(buffer, view, { frame: 4, object: 9, vertexCount: 3 });
    expect(f.values[0]).toBeCloseTo(-2, 6);
    expect(f.values[1]).toBeCloseTo(6, 6);
    expect(f.values[2]).toBeCloseTo(2, 5);
    expect([f.min, f.max, f.map, f.frame, f.object]).toEqual([-2, 6, 1, 4, 9]);
  });

  it('the reserved value 65535 decodes as no data', () => {
    const { buffer, view } = scalarFrame(0, 1, 2, 0, [65535, 0]);
    const f = decodeScalars(buffer, view, { frame: 0, object: 0, vertexCount: 2 });
    expect(f.values[0]).toBe(Math.fround(NO_DATA));
    expect(f.values[1]).toBe(0);
  });

  it('a frame whose length does not match its vertex count is refused', () => {
    const { buffer, view } = scalarFrame(0, 1, 1, 0, [1, 2], 2);
    expect(() => decodeScalars(buffer, view, { frame: 0, object: 0, vertexCount: 2 })).toThrow('scalars frame length mismatch');
  });

  it('a frame with minimum above maximum or a non-finite bound is refused', () => {
    const a = scalarFrame(5, 1, 1, 0, [1]);
    expect(() => decodeScalars(a.buffer, a.view, { frame: 0, object: 0, vertexCount: 1 })).toThrow('scalars frame range is invalid');
    const b = scalarFrame(0, Infinity, 1, 0, [1]);
    expect(() => decodeScalars(b.buffer, b.view, { frame: 0, object: 0, vertexCount: 1 })).toThrow('scalars frame range is invalid');
  });

  it('a frame with a map index outside 0 to 7 or a fractional one is refused', () => {
    for (const map of [8, -1, 1.5]) {
      const a = scalarFrame(0, 1, map, 0, [1]);
      expect(() => decodeScalars(a.buffer, a.view, { frame: 0, object: 0, vertexCount: 1 })).toThrow('out of range');
    }
  });
});

describe('PNG resolution tagging', () => {
  it('pixels per metre follow from 0.0254 metres per inch', () => {
    expect(pixelsPerMetre(300)).toBe(11811);
    expect(pixelsPerMetre(72)).toBe(2835);
    expect(pixelsPerMetre(254)).toBe(10000);
  });

  it('a pHYs chunk with a valid checksum is inserted right after the header', () => {
    const png = new Uint8Array([...SIG, ...IHDR, ...IDAT, ...IEND]);
    const out = withPpi(png, 300);
    expect(Array.from(out.subarray(0, 8))).toEqual(SIG);
    const chunks = readChunks(out);
    expect(chunks.map((c) => c.type)).toEqual(['IHDR', 'pHYs', 'IDAT', 'IEND']);
    expect(chunks.every((c) => c.crcOk)).toBe(true);
    const p = chunks[1].data;
    const v = new DataView(p.buffer, p.byteOffset, p.byteLength);
    expect(p.length).toBe(9);
    expect(v.getUint32(0)).toBe(11811);
    expect(v.getUint32(4)).toBe(11811);
    expect(p[8]).toBe(1);
  });

  it('image data is carried through untouched', () => {
    const out = withPpi(new Uint8Array([...SIG, ...IHDR, ...IDAT, ...IEND]), 96);
    expect(Array.from(readChunks(out)[2].data)).toEqual([1, 2, 3, 4, 5]);
    expect(Array.from(readChunks(out)[0].data)).toEqual(IHDR.slice(8, 21));
  });

  it('an existing pHYs chunk is replaced, not duplicated', () => {
    const old = chunk('pHYs', [0, 0, 0, 1, 0, 0, 0, 1, 0]);
    const out = withPpi(new Uint8Array([...SIG, ...IHDR, ...IDAT, ...old, ...IEND]), 254);
    const chunks = readChunks(out);
    expect(chunks.map((c) => c.type)).toEqual(['IHDR', 'pHYs', 'IDAT', 'IEND']);
    const p = chunks[1].data;
    expect(new DataView(p.buffer, p.byteOffset, p.byteLength).getUint32(0)).toBe(10000);
  });

  it('tagging twice gives the same bytes as tagging once', () => {
    const png = new Uint8Array([...SIG, ...IHDR, ...IDAT, ...IEND]);
    const once = withPpi(png, 150);
    expect(Array.from(withPpi(once, 150))).toEqual(Array.from(once));
  });

  it('bytes that are not a PNG come back unchanged as a copy', () => {
    const junk = new Uint8Array(40).fill(9);
    const out = withPpi(junk, 300);
    expect(Array.from(out)).toEqual(Array.from(junk));
    expect(out).not.toBe(junk);
  });
});

describe('animation state parsing', () => {
  it('an empty reply gives the default 30 fps timeline from frame 0 to 120', () => {
    const a = parseAnim({});
    expect([a.fps, a.start, a.end, a.current]).toEqual([30, 0, 120, 0]);
    expect(a.active).toBeNull();
    expect(a.autoKey).toBe(false);
  });

  it('state nested under an anim field is read the same as a bare state', () => {
    const body = { fps: 24, start: 10, end: 48, current: 12, active_cache: 3, auto_key: true };
    expect(parseAnim({ anim: body })).toEqual(parseAnim(body));
    expect(parseAnim(body).fps).toBe(24);
    expect(parseAnim(body).active).toBe(3);
    expect(parseAnim(body).autoKey).toBe(true);
  });

  it('keyframes are ordered by frame whatever order they arrive in', () => {
    const a = parseAnim({ tracks: [{ id: 1, target: 'wind.strength', keys: [{ frame: 30, value: 2 }, { frame: 0, value: 1 }, { frame: 12, value: 5 }] }] });
    expect(a.tracks[0].keys.map((k) => k.frame)).toEqual([0, 12, 30]);
    expect(a.tracks[0].keys.map((k) => k.value)).toEqual([1, 5, 2]);
  });

  it('a missing or unknown interpolation falls back to linear while step and smooth are kept', () => {
    const a = parseAnim({ tracks: [{ id: 1, target: 't', keys: [{ frame: 0, interp: 'step' }, { frame: 1, interp: 'smooth' }, { frame: 2, interp: 'bezier' }, { frame: 3 }] }] });
    expect(a.tracks[0].keys.map((k) => k.interp)).toEqual(['step', 'smooth', 'linear', 'linear']);
  });

  it('non-finite and wrongly typed numbers fall back to defaults', () => {
    const a = parseAnim({ fps: 'fast', end: Infinity, start: null });
    expect([a.fps, a.start, a.end]).toEqual([30, 0, 120]);
  });

  it('caches default to visible pattern caches and malformed list entries are dropped', () => {
    const a = parseAnim({ caches: [{ id: 2, name: 'run', frames: 40 }, 'junk', null, { id: 3, kind: 'trim', visible: false, locked: true }] });
    expect(a.caches.length).toBe(2);
    expect(a.caches[0]).toMatchObject({ id: 2, name: 'run', frames: 40, visible: true, kind: 'pattern', locked: false });
    expect(a.caches[1]).toMatchObject({ id: 3, visible: false, kind: 'trim', locked: true });
  });

  it('timeline rows list caches first, then motions, then tracks', () => {
    const a = parseAnim({ tracks: [{ id: 5 }], motions: [{ id: 4 }], caches: [{ id: 9 }, { id: 8 }] });
    expect(rowsOf(a)).toEqual([{ kind: 'cache', id: 9 }, { kind: 'cache', id: 8 }, { kind: 'motion', id: 4 }, { kind: 'track', id: 5 }]);
  });

  it('body rows fall back to the id when a body has no name', () => {
    expect(bodyRows({ bodies: [{ id: 1, name: 'Ava' }, { id: 2 }] })).toEqual([{ id: 1, name: 'Ava' }, { id: 2, name: '2' }]);
    expect(bodyRows(null)).toEqual([]);
  });
});

describe('playback and job state', () => {
  it('progress is derived from frame over total when no progress is given', () => {
    expect(parseJob({ phase: 'running', frame: 30, total: 120 }).progress).toBeCloseTo(0.25, 12);
    expect(parseJob({ state: 'running', frame: 10, frames: 40 }).progress).toBeCloseTo(0.25, 12);
  });

  it('a percentage progress is scaled to a fraction and fractions pass through', () => {
    expect(parseJob({ progress: 50 }).progress).toBeCloseTo(0.5, 12);
    expect(parseJob({ progress: 0.75 }).progress).toBeCloseTo(0.75, 12);
  });

  it('progress is always within 0 and 1', () => {
    expect(parseJob({ progress: -3 }).progress).toBe(0);
    expect(parseJob({ progress: 250 }).progress).toBe(1);
    expect(parseJob({ frame: 5, total: 0 }).progress).toBe(0);
    expect(parseJob(undefined).progress).toBe(0);
  });

  it('a job is running when flagged or when its phase says so', () => {
    expect(parseJob({ running: true }).running).toBe(true);
    expect(parseJob({ phase: 'running' }).running).toBe(true);
    expect(parseJob({ phase: 'done' }).running).toBe(false);
  });

  it('a failed job reports its error text, or its message when there is no error field', () => {
    expect(parseJob({ error: 'disk full' }).failed).toBe('disk full');
    expect(parseJob({ state: 'failed', message: 'solver diverged' }).failed).toBe('solver diverged');
    expect(parseJob({ state: 'failed' }).failed).toBe('failed');
    expect(parseJob({ state: 'done', message: 'ok' }).failed).toBe('');
  });

  it('playback state carries the playing flag and current play frame', () => {
    expect(parseJob({ playing: true, play_frame: 17 })).toMatchObject({ playing: true, playFrame: 17 });
    expect(parseJob({})).toMatchObject({ playing: false, playFrame: null });
  });
});

describe('keyframe value text', () => {
  it('numbers typed as text become numbers', () => {
    expect(parseValue(' 3.5 ')).toBe(3.5);
    expect(parseValue('-2e3')).toBe(-2000);
  });

  it('blank text means no value', () => {
    expect(parseValue('   ')).toBeUndefined();
  });

  it('JSON arrays, objects, quoted strings and booleans are parsed', () => {
    expect(parseValue('[1, 2, 3]')).toEqual([1, 2, 3]);
    expect(parseValue('{"x": 1}')).toEqual({ x: 1 });
    expect(parseValue('"12"')).toBe('12');
    expect(parseValue('true')).toBe(true);
  });

  it('plain words and broken JSON stay as text', () => {
    expect(parseValue('silk')).toBe('silk');
    expect(parseValue('[1, 2')).toBe('[1, 2');
  });

  it('showing a value then parsing it returns the same value', () => {
    for (const v of [0, 4.25, [1, 2, 3], { a: [true, null] }, false, 'silk']) expect(parseValue(showValue(v))).toEqual(v);
    expect(showValue(null)).toBe('');
    expect(showValue(undefined)).toBe('');
  });
});
