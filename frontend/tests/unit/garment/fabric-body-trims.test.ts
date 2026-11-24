import { describe, it, expect } from 'vitest';
import { toHex, fromHex } from '../../../src/workspace/fabric/types';
import { fmtNum } from '../../../src/workspace/fabric/widgets';
import { eulerOf } from '../../../src/workspace/body/pose';
import { placeItems } from '../../../src/workspace/body/store';
import { seamLines, type MeshInfo, type SeamJson } from '../../../src/workspace/drape/seams';
import { trimDoc, findPrint, itemExists, num, str, flag, styleOf } from '../../../src/workspace/trims/types';
import type { Garment, Panel } from '../../../src/workspace/types';

type Quat = [number, number, number, number];

function quatMul(a: Quat, b: Quat): Quat {
  const [ax, ay, az, aw] = a;
  const [bx, by, bz, bw] = b;
  return [
    aw * bx + ax * bw + ay * bz - az * by,
    aw * by - ax * bz + ay * bw + az * bx,
    aw * bz + ax * by - ay * bx + az * bw,
    aw * bw - ax * bx - ay * by - az * bz,
  ];
}

function axisQuat(axis: 0 | 1 | 2, deg: number): Quat {
  const h = (deg * Math.PI) / 360;
  const q: Quat = [0, 0, 0, Math.cos(h)];
  q[axis] = Math.sin(h);
  return q;
}

describe('fabric colours', () => {
  it('pure red, green and blue map to their hex codes', () => {
    expect(toHex([1, 0, 0, 1])).toBe('#ff0000');
    expect(toHex([0, 1, 0, 1])).toBe('#00ff00');
    expect(toHex([0, 0, 1, 0.2])).toBe('#0000ff');
  });
  it('out of range channels are clamped to the 00 to ff range', () => {
    expect(toHex([2, -1, 0.2, 1])).toBe('#ff0033');
  });
  it('a hex code reads back as channel fractions with the given alpha', () => {
    const c = fromHex('#336699', 0.5);
    expect(c[0]).toBeCloseTo(0.2, 12);
    expect(c[1]).toBeCloseTo(0.4, 12);
    expect(c[2]).toBeCloseTo(0.6, 12);
    expect(c[3]).toBe(0.5);
  });
  it('hex to colour and back gives the same hex', () => {
    for (const hex of ['#000000', '#ffffff', '#1a2b3c', '#f97316', '#808080']) expect(toHex(fromHex(hex, 1))).toBe(hex);
  });
  it('fabric numbers are shown without floating point noise', () => {
    expect(fmtNum(0.1 + 0.2)).toBe('0.3');
    expect(fmtNum(150)).toBe('150');
    expect(fmtNum(0.05)).toBe('0.05');
  });
});

describe('joint angles from a rotation', () => {
  it('no rotation reads as zero degrees on every axis', () => {
    expect(eulerOf([0, 0, 0, 1])).toEqual([0, 0, 0]);
  });
  it('a turn about a single axis reads as that angle on that axis only', () => {
    expect(eulerOf(axisQuat(0, 90))).toEqual([90, 0, 0]);
    expect(eulerOf(axisQuat(1, 30))).toEqual([0, 30, 0]);
    expect(eulerOf(axisQuat(2, -45))).toEqual([0, 0, -45]);
  });
  it('angles read from a combined rotation rebuild the same rotation', () => {
    const q = quatMul(quatMul(axisQuat(2, 40), axisQuat(1, 30)), axisQuat(0, 20));
    const [ex, ey, ez] = eulerOf(q);
    const back = quatMul(quatMul(axisQuat(2, ez), axisQuat(1, ey)), axisQuat(0, ex));
    const dotp = Math.abs(q[0] * back[0] + q[1] * back[1] + q[2] * back[2] + q[3] * back[3]);
    expect(dotp).toBeCloseTo(1, 5);
    expect([ex, ey, ez]).toEqual([20, 30, 40]);
  });
});

describe('body placement points', () => {
  it('plain names become points keyed by name', () => {
    expect(placeItems(['waist', 'chest'])).toEqual([{ key: 'waist', name: 'waist' }, { key: 'chest', name: 'chest' }]);
  });
  it('points inside a reply keep their id, name and offset', () => {
    const items = placeItems({ points: [{ id: 7, name: 'Left sleeve', offset: 12.5 }, { name: 'Back' }, 42] });
    expect(items).toEqual([{ key: '7', name: 'Left sleeve', offset: 12.5 }, { key: 'Back', name: 'Back', offset: undefined }]);
  });
  it('a reply with no points gives an empty list', () => {
    expect(placeItems(null)).toEqual([]);
    expect(placeItems({ points: 'none' })).toEqual([]);
  });
});

function edgeMesh(base: number, ts: number[]): MeshInfo {
  const boundary = ts.map((_, i) => base + i);
  const param = ts.map((t) => [0, t]);
  boundary.push(base + 90);
  param.push([1, 0.5]);
  return { key: String(base), segments: 4, boundary, param, positions: [] };
}

describe('sewing lines between draped panels', () => {
  const meshes = new Map<number, MeshInfo>([[1, edgeMesh(10, [0, 0.5, 1])], [2, edgeMesh(20, [0, 0.5, 1])], [3, edgeMesh(30, [0, 0.25, 0.5, 0.75, 1])]]);
  const seam = (b: number, reversed: boolean): SeamJson => ({ id: 1, a: [{ panel: 1, segment: 0, t0: 0, t1: 1 }], b: [{ panel: b, segment: 0, t0: 0, t1: 1 }], reversed });

  it('an unreversed seam joins the two edges start to start and end to end', () => {
    const lines = seamLines([seam(2, false)], meshes);
    expect(lines.map((l) => [l.va, l.vb])).toEqual([[10, 20], [11, 21], [12, 22]]);
    expect(lines.every((l) => l.pa === 1 && l.pb === 2)).toBe(true);
  });
  it('a reversed seam joins the start of one edge to the end of the other', () => {
    const lines = seamLines([seam(2, true)], meshes);
    expect(lines.map((l) => [l.va, l.vb])).toEqual([[10, 22], [11, 21], [12, 20]]);
  });
  it('edges with different vertex counts still meet at both ends and in the middle', () => {
    const lines = seamLines([seam(3, false)], meshes);
    expect(lines.map((l) => [l.va, l.vb])).toEqual([[10, 30], [11, 32], [12, 34]]);
  });
  it('a span given backwards walks the edge backwards', () => {
    const s: SeamJson = { id: 2, a: [{ panel: 1, segment: 0, t0: 1, t1: 0 }], b: [{ panel: 2, segment: 0, t0: 0, t1: 1 }], reversed: false };
    expect(seamLines([s], meshes).map((l) => [l.va, l.vb])).toEqual([[12, 20], [11, 21], [10, 22]]);
  });
  it('a partial span only sews the vertices inside it', () => {
    const s: SeamJson = { id: 3, a: [{ panel: 3, segment: 0, t0: 0.25, t1: 0.75 }], b: [{ panel: 2, segment: 0, t0: 0, t1: 1 }], reversed: false };
    expect(seamLines([s], meshes).map((l) => l.va)).toEqual([31, 32, 33]);
  });
  it('the corner vertex that starts the next edge closes a full span', () => {
    const mesh: MeshInfo = { key: 'c', segments: 4, boundary: [50, 51, 52], param: [[0, 0], [0, 0.5], [1, 0]], positions: [] };
    const m = new Map<number, MeshInfo>([[5, mesh], [2, meshes.get(2) as MeshInfo]]);
    const s: SeamJson = { id: 4, a: [{ panel: 5, segment: 0, t0: 0, t1: 1 }], b: [{ panel: 2, segment: 0, t0: 0, t1: 1 }], reversed: false };
    expect(seamLines([s], m).map((l) => [l.va, l.vb])).toEqual([[50, 20], [51, 21], [52, 22]]);
  });
  it('a seam to a panel that has no mesh draws nothing', () => {
    expect(seamLines([seam(99, false)], meshes)).toEqual([]);
  });
});

function panel(id: number, prints: unknown[]): Panel {
  return { id, name: `p${id}`, outline: [], inner: [], position: [0, 0], rotation: 0, mirror: false, grain: 0, particle_distance: 10, layer: 0, prints } as unknown as Panel;
}

describe('trim records', () => {
  it('a garment without trims reads as empty lists', () => {
    const g: Garment = { units: 'mm', panels: [] };
    expect(trimDoc(g)).toEqual({ styles: [], hardware: [], zippers: [], stitches: [], finishes: [], ripped: [] });
  });
  it('style settings fall back when missing or of the wrong type', () => {
    const params = { diameter: 15, label: 'Horn', sew_through: true, holes: '4', bad: Number.NaN };
    expect(num(params, 'diameter', 10)).toBe(15);
    expect(num(params, 'holes', 2)).toBe(2);
    expect(num(params, 'bad', 3)).toBe(3);
    expect(str(params, 'label', 'x')).toBe('Horn');
    expect(str(params, 'diameter', 'x')).toBe('x');
    expect(flag(params, 'sew_through', false)).toBe(true);
    expect(flag(undefined, 'sew_through', false)).toBe(false);
  });
  it('a print is found on the panel that carries it', () => {
    const g: Garment = { units: 'mm', panels: [panel(1, []), panel(2, [{ id: 9, name: 'Logo' }])] };
    const hit = findPrint(g, 9);
    expect(hit?.panel.id).toBe(2);
    expect(hit?.print.name).toBe('Logo');
    expect(findPrint(g, 10)).toBeNull();
  });
  it('an item exists only under its own kind and id', () => {
    const g = { units: 'mm', panels: [panel(1, [{ id: 3 }])], hardware: [{ id: 3 }], zippers: [{ id: 4 }], styles: [{ id: 8, kind: 'button', name: 'B', params: {} }] } as unknown as Garment;
    expect(itemExists(g, { kind: 'hardware', id: 3 })).toBe(true);
    expect(itemExists(g, { kind: 'zipper', id: 3 })).toBe(false);
    expect(itemExists(g, { kind: 'zipper', id: 4 })).toBe(true);
    expect(itemExists(g, { kind: 'print', id: 3 })).toBe(true);
    expect(itemExists(g, { kind: 'stitch', id: 3 })).toBe(false);
    expect(styleOf(trimDoc(g), 8)?.name).toBe('B');
    expect(styleOf(trimDoc(g), 9)).toBeUndefined();
  });
});
