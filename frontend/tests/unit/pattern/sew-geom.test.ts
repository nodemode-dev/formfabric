import { describe, it, expect } from 'vitest';
import { geoOf } from '../../../src/workspace/geom';
import {
  Tracker, buildSewScene, chainOf, lineClosed, lineSegs, lineSpan, nearestOnChain, nearestParam, pickLine,
  pointAt, pointAtParam, runsToEnds, seamAt, seamColor, spansBetween, tackAt, walkSign,
} from '../../../src/workspace/pattern-view/sew-geom';
import { spanArg, type Seam, type SeamEnd, type SewSnapshot, type Tack } from '../../../src/workspace/pattern-view/sew-types';
import { geoMap, openInner, quarterDisc, square } from './fixtures';

const end = (panel: number, segment: number, t0 = 0, t1 = 1, inner: number | null = null): SeamEnd => ({ panel, inner, segment, t0, t1 });

const seamOf = (id: number, a: SeamEnd[], b: SeamEnd[]): Seam => ({ id, a, b, reversed: false, fold: 'turned', fold_angle: 180, fold_strength: 1, ratio: 1, tension: 1, side: 'auto' });

describe('seam chain lengths', () => {
  it('measures the whole outline of a 100 mm square as 400 mm', () => {
    const geos = geoMap(square(1, [10, 20], 100));
    const c = chainOf(geos, [end(1, 0), end(1, 1), end(1, 2), end(1, 3)]);
    expect(c.total).toBeCloseTo(400, 9);
    expect(c.cum[c.cum.length - 1]).toBeCloseTo(400, 9);
  });

  it('measures a half edge span as half the edge length', () => {
    const geos = geoMap(square(1, [0, 0], 100));
    expect(chainOf(geos, [end(1, 2, 0.25, 0.75)]).total).toBeCloseTo(50, 9);
  });

  it('measures a quarter circle edge of radius 100 close to pi times 50', () => {
    const geos = geoMap(quarterDisc(1, 100));
    const total = chainOf(geos, [end(1, 0)]).total;
    expect(Math.abs(total - Math.PI * 50)).toBeLessThan(0.3);
  });

  it('gives the same length walking an edge backwards', () => {
    const geos = geoMap(quarterDisc(1, 80));
    expect(chainOf(geos, [end(1, 0, 1, 0)]).total).toBeCloseTo(chainOf(geos, [end(1, 0, 0, 1)]).total, 9);
  });

  it('returns an empty chain for a span on a panel that does not exist', () => {
    const c = chainOf(geoMap(square(1, [0, 0], 100)), [end(9, 0)]);
    expect(c.pts).toEqual([]);
    expect(c.total).toBe(0);
  });
});

describe('points along a chain', () => {
  const geos = geoMap(square(1, [10, 20], 100));
  const around = chainOf(geos, [end(1, 0), end(1, 1), end(1, 2), end(1, 3)]);

  it('puts fraction one quarter of a square outline on the second corner', () => {
    const p = pointAt(around, 0.25);
    expect(p[0]).toBeCloseTo(110, 9);
    expect(p[1]).toBeCloseTo(20, 9);
  });

  it('puts fraction 0.375 halfway up the second edge', () => {
    const p = pointAt(around, 0.375);
    expect(p[0]).toBeCloseTo(110, 9);
    expect(p[1]).toBeCloseTo(70, 9);
  });

  it('clamps fractions outside zero to one onto the chain ends', () => {
    const one = chainOf(geos, [end(1, 0)]);
    expect(pointAt(one, -3)).toEqual([10, 20]);
    expect(pointAt(one, 7)).toEqual([110, 20]);
  });

  it('answers the origin for an empty chain', () => {
    expect(pointAt({ pts: [], cum: [], total: 0 }, 0.5)).toEqual([0, 0]);
  });

  it('finds the perpendicular distance and fraction of a point beside an edge', () => {
    const one = chainOf(geos, [end(1, 0)]);
    const r = nearestOnChain(one, [35, -10]);
    expect(r.d).toBeCloseTo(30, 9);
    expect(r.f).toBeCloseTo(0.25, 9);
  });

  it('measures to the chain end when the point lies past it', () => {
    const one = chainOf(geos, [end(1, 0)]);
    const r = nearestOnChain(one, [113, 24]);
    expect(r.d).toBeCloseTo(5, 9);
    expect(r.f).toBeCloseTo(1, 9);
  });

  it('agrees with pointAt when the nearest fraction is fed back', () => {
    const r = nearestOnChain(around, [112, 95]);
    const p = pointAt(around, r.f);
    expect(p[0]).toBeCloseTo(110, 6);
    expect(p[1]).toBeCloseTo(95, 6);
  });
});

describe('picking lines under the pointer', () => {
  const inner = openInner(7, [20, 50], [80, 50]);
  const a = geoOf(square(1, [0, 0], 100, { inner: [inner] }));
  const b = geoOf(square(2, [300, 0], 100));

  it('picks the outline edge within tolerance and reports where along it', () => {
    const hit = pickLine([a, b], [30, -4], 5);
    expect(hit).not.toBeNull();
    expect(hit?.panel).toBe(1);
    expect(hit?.inner).toBeNull();
    expect(hit?.segment).toBe(0);
    expect(hit?.t).toBeCloseTo(0.3, 9);
    expect(hit?.d).toBeCloseTo(4, 9);
    expect(hit?.pt[0]).toBeCloseTo(30, 9);
    expect(hit?.pt[1]).toBeCloseTo(0, 9);
  });

  it('picks nothing when the pointer is further away than the tolerance', () => {
    expect(pickLine([a, b], [30, -6], 5)).toBeNull();
  });

  it('picks an internal line by its id', () => {
    const hit = pickLine([a, b], [50, 52], 5);
    expect(hit?.inner).toBe(7);
    expect(hit?.t).toBeCloseTo(0.5, 9);
  });

  it('picks the edge of the second panel when the pointer is there', () => {
    const hit = pickLine([a, b], [402, 50], 5);
    expect(hit?.panel).toBe(2);
    expect(hit?.segment).toBe(1);
    expect(hit?.t).toBeCloseTo(0.5, 9);
  });

  it('reports a line parameter as segment index plus fraction', () => {
    const r = nearestParam(a, null, [103, 25]);
    expect(r?.p).toBeCloseTo(1.25, 9);
    expect(r?.d).toBeCloseTo(3, 9);
  });

  it('turns a line parameter back into the point it came from', () => {
    const p = pointAtParam(a, null, 2.5);
    expect(p?.[0]).toBeCloseTo(50, 9);
    expect(p?.[1]).toBeCloseTo(100, 9);
    const last = pointAtParam(a, null, 99);
    expect(last?.[0]).toBeCloseTo(0, 9);
    expect(last?.[1]).toBeCloseTo(0, 9);
    expect(pointAtParam(undefined, null, 1)).toBeNull();
  });

  it('treats the outline as closed and an open internal line as open', () => {
    expect(lineClosed(a, null)).toBe(true);
    expect(lineClosed(a, 7)).toBe(false);
    expect(lineClosed(undefined, null)).toBe(false);
    expect(lineSegs(a, null)).toHaveLength(4);
    expect(lineSegs(a, 7)).toHaveLength(1);
    expect(lineSegs(a, 99)).toEqual([]);
  });
});

describe('splitting a parameter range into per-edge runs', () => {
  it('splits a forward range at every corner it crosses', () => {
    const runs = spansBetween(4, 0.5, 2.25);
    expect(runs.map((r) => r.segment)).toEqual([0, 1, 2]);
    expect(runs[0].t0).toBeCloseTo(0.5, 9);
    expect(runs[0].t1).toBeCloseTo(1, 9);
    expect(runs[1].t0).toBeCloseTo(0, 9);
    expect(runs[1].t1).toBeCloseTo(1, 9);
    expect(runs[2].t1).toBeCloseTo(0.25, 9);
  });

  it('walks backwards with decreasing fractions', () => {
    const runs = spansBetween(4, 1.5, 0.5);
    expect(runs.map((r) => r.segment)).toEqual([1, 0]);
    expect(runs[0].t0).toBeCloseTo(0.5, 9);
    expect(runs[0].t1).toBeCloseTo(0, 9);
    expect(runs[1].t0).toBeCloseTo(1, 9);
    expect(runs[1].t1).toBeCloseTo(0.5, 9);
  });

  it('wraps past the last edge back to the first on a closed line', () => {
    const runs = spansBetween(4, 3.5, 4.5);
    expect(runs.map((r) => r.segment)).toEqual([3, 0]);
  });

  it('wraps below zero onto the last edge when walking backwards', () => {
    const runs = spansBetween(4, 0.5, -0.5);
    expect(runs.map((r) => r.segment)).toEqual([0, 3]);
  });

  it('keeps the total covered parameter equal to the requested range', () => {
    for (const [p0, p1] of [[0.2, 3.9], [3.1, 0.4], [1, 5], [2.75, 2.8]] as Array<[number, number]>) {
      const sum = spansBetween(4, p0, p1).reduce((s, r) => s + (r.t1 - r.t0), 0);
      expect(sum).toBeCloseTo(p1 - p0, 9);
    }
  });

  it('gives no runs for an empty range or a line without edges', () => {
    expect(spansBetween(4, 1.5, 1.5)).toEqual([]);
    expect(spansBetween(0, 0, 2)).toEqual([]);
  });

  it('makes runs whose chain length equals the walked distance on the square', () => {
    const geos = geoMap(square(1, [0, 0], 100));
    const ends = runsToEnds({ panel: 1, inner: null }, spansBetween(4, 0.5, 2.25));
    expect(chainOf(geos, ends).total).toBeCloseTo(175, 9);
  });
});

describe('recovering a span from seam ends', () => {
  const geos = geoMap(square(1, [0, 0], 100), square(2, [300, 0], 100));

  it('round trips a range through runs, ends and back', () => {
    const ends = runsToEnds({ panel: 1, inner: null }, spansBetween(4, 0.5, 2.25));
    const span = lineSpan(geos, ends);
    expect(span?.n).toBe(4);
    expect(span?.closed).toBe(true);
    expect(span?.p0).toBeCloseTo(0.5, 9);
    expect(span?.p1).toBeCloseTo(2.25, 9);
    expect(span?.ref).toEqual({ panel: 1, inner: null });
  });

  it('round trips a range that wraps around the closed outline', () => {
    const span = lineSpan(geos, runsToEnds({ panel: 1, inner: null }, spansBetween(4, 3.5, 4.5)));
    expect(span?.p0).toBeCloseTo(3.5, 9);
    expect(span?.p1).toBeCloseTo(4.5, 9);
  });

  it('refuses ends that leave a gap', () => {
    expect(lineSpan(geos, [end(1, 0, 0, 1), end(1, 2, 0, 1)])).toBeNull();
  });

  it('refuses ends that lie on two different panels', () => {
    expect(lineSpan(geos, [end(1, 0, 0, 1), end(2, 1, 0, 1)])).toBeNull();
  });

  it('refuses an empty list and an unknown panel', () => {
    expect(lineSpan(geos, [])).toBeNull();
    expect(lineSpan(geos, [end(5, 0)])).toBeNull();
  });

  it('reads the walking direction of a seam side', () => {
    expect(walkSign([end(1, 0, 0, 1), end(1, 1, 0, 0.5)])).toBe(1);
    expect(walkSign([end(1, 1, 0.5, 0), end(1, 0, 1, 0)])).toBe(-1);
  });

  it('leaves the internal line out of an engine span for an outline edge and keeps it otherwise', () => {
    expect(spanArg(end(1, 2, 0.1, 0.9))).toEqual({ panel: 1, segment: 2, t0: 0.1, t1: 0.9 });
    expect(spanArg(end(1, 0, 0, 1, 7))).toEqual({ panel: 1, inner: 7, segment: 0, t0: 0, t1: 1 });
  });
});

describe('tracking a drag along a line', () => {
  it('clamps to the ends of an open line', () => {
    const tr = new Tracker(1, 3, false);
    expect(tr.feed(-2)).toBe(0);
    expect(tr.feed(2.5)).toBe(2.5);
    expect(tr.feed(8)).toBe(3);
  });

  it('keeps counting upwards when the pointer crosses the start of a closed outline', () => {
    const tr = new Tracker(3.8, 4, true);
    expect(tr.feed(3.9)).toBeCloseTo(3.9, 9);
    expect(tr.feed(0.1)).toBeCloseTo(4.1, 9);
    expect(tr.feed(0.6)).toBeCloseTo(4.6, 9);
  });

  it('counts downwards below zero when dragged backwards across the start', () => {
    const tr = new Tracker(0.2, 4, true);
    expect(tr.feed(3.9)).toBeCloseTo(-0.1, 9);
  });

  it('never covers more than one full lap of a closed outline', () => {
    const tr = new Tracker(0, 4, true);
    let p = 0;
    for (let i = 1; i <= 30; i++) p = tr.feed((i * 0.5) % 4);
    expect(p).toBe(4);
  });
});

describe('sewing scene', () => {
  const p1 = square(1, [0, 0], 100);
  const p2 = square(2, [300, 0], 100, { rotation: 90 });
  const geos = [geoOf(p1), geoOf(p2)];
  const tack: Tack = { id: 4, a: { panel: 2, point: [10, 0], body_point: null }, b: { panel: null, point: [0, 0], body_point: [1, 2, 3] }, length: 5 };
  const sew: SewSnapshot = {
    seams: [seamOf(1, [end(1, 1)], [end(2, 3, 0, 0.5)]), seamOf(2, [end(1, 3)], [end(2, 1)])],
    tacks: [tack],
    info: new Map(),
    bad: new Map([[1, 50]]),
  };
  const scene = buildSewScene(geos, sew);

  it('carries the reported mismatch only on the seam that has one', () => {
    expect(scene.seams[0].mismatch).toBe(50);
    expect(scene.seams[1].mismatch).toBeNull();
  });

  it('builds seam sides whose length difference equals the geometric difference', () => {
    expect(scene.seams[0].a.total - scene.seams[0].b.total).toBeCloseTo(50, 9);
    expect(scene.seams[1].a.total).toBeCloseTo(scene.seams[1].b.total, 9);
  });

  it('places a tack end through the rotation and position of its panel', () => {
    expect(scene.tacks[0].a?.[0]).toBeCloseTo(300, 9);
    expect(scene.tacks[0].a?.[1]).toBeCloseTo(10, 9);
  });

  it('has no pattern position for a tack end that sits on the body', () => {
    expect(scene.tacks[0].b).toBeNull();
  });

  it('finds the seam side under the pointer with the fraction along it', () => {
    const hit = seamAt(scene, [103, 25], 5);
    expect(hit?.seam.seam.id).toBe(1);
    expect(hit?.side).toBe('a');
    expect(hit?.f).toBeCloseTo(0.25, 9);
    expect(hit?.d).toBeCloseTo(3, 9);
  });

  it('finds no seam on an unsewn edge or outside the tolerance', () => {
    expect(seamAt(scene, [50, -2], 5)).toBeNull();
    expect(seamAt(scene, [108, 25], 5)).toBeNull();
  });

  it('finds a tack within tolerance of its end and none further away', () => {
    expect(tackAt(scene, [303, 14], 6)?.tack.id).toBe(4);
    expect(tackAt(scene, [303, 14], 4)).toBeNull();
  });

  it('gives different seams different colours spaced by the golden angle', () => {
    const golden = 360 * (1 - 2 / (1 + Math.sqrt(5)));
    expect(seamColor(0)).toBe('hsl(0, 68%, 60%)');
    expect(seamColor(1)).toBe(`hsl(${Math.round(golden)}, 68%, 60%)`);
    expect(seamColor(3)).toBe(`hsl(${Math.round((3 * golden) % 360)}, 68%, 60%)`);
    expect(new Set(Array.from({ length: 20 }, (_, i) => seamColor(i))).size).toBe(20);
  });
});
