import { registerKind, type ScalarsFrame } from '../../engine/stream';

export const KIND_SCALARS = 3;
export const NO_DATA = -1e30;
const NONE = 65535;
const STEPS = 65534;

export const MAP_KINDS = ['none', 'strain', 'stress', 'pressure', 'fit', 'contact', 'thickness'] as const;
export type MapKind = (typeof MAP_KINDS)[number];

// Kind 3 (ffb-stream/src/frame.rs): common 44 byte header with origin[0] = min, origin[1] = max,
// scale[0] = map index, scale[1] = component; payload is vertexCount little-endian u16 quantised
// between min and max (65535 means no data).
export function decodeScalars(buffer: ArrayBuffer, view: DataView, head: { frame: number; object: number; vertexCount: number }): ScalarsFrame {
  const n = head.vertexCount;
  if (buffer.byteLength !== 44 + n * 2) throw new Error('scalars frame length mismatch');
  const min = view.getFloat32(20, true);
  const max = view.getFloat32(24, true);
  const map = view.getFloat32(32, true);
  const extent = max - min;
  const values = new Float32Array(n);
  let at = 44;
  for (let i = 0; i < n; i++) {
    const q = view.getUint16(at, true);
    values[i] = q === NONE ? NO_DATA : min + (q / STEPS) * extent;
    at += 2;
  }
  return { kind: 'scalars', frame: head.frame, object: head.object, vertexCount: n, values, min, max, map };
}

let installed = false;

export function installScalarDecoder(): void {
  if (installed) return;
  installed = true;
  registerKind(KIND_SCALARS, decodeScalars);
}
