import { describe, it, expect } from 'vitest';
import { Selection, toUnit, fromUnit, fmtLen, fmtArea, fmtNum, decimals } from '../../../src/workspace/state';
import { place, unplace, placeVec, segsOf, segLength, bezAt, nearestOnSeg, boundsOf, unionBounds, pointInPoly, geoOf, hitTest } from '../../../src/workspace/geom';
import type { Panel, Pt, Vec } from '../../../src/workspace/types';

const corner = (x: number, y: number): Pt => ({ p: [x, y], kind: 'corner', h_in: null, h_out: null });

function square(id: number, size: number, position: Vec, layer = 0): Panel {
  return { id, name: `p${id}`, outline: [corner(0, 0), corner(size, 0), corner(size, size), corner(0, size)], inner: [], position, rotation: 0, mirror: false, grain: 0, particle_distance: 10, layer };
}

describe('unit conversions', () => {
  it('one inch is 25.4 mm in both directions', () => {
    expect(toUnit(25.4, 'in')).toBeCloseTo(1, 12);
    expect(fromUnit(1, 'in')).toBeCloseTo(25.4, 12);
  });
  it('one centimetre is 10 mm in both directions', () => {
    expect(toUnit(35, 'cm')).toBeCloseTo(3.5, 12);
    expect(fromUnit(3.5, 'cm')).toBeCloseTo(35, 12);
  });
  it('converting to a unit and back returns the same length', () => {
    for (const unit of ['mm', 'cm', 'in']) {
      for (const mm of [0, 0.5, 123.456, 2540]) expect(fromUnit(toUnit(mm, unit), unit)).toBeCloseTo(mm, 9);
    }
  });
  it('shows 254 mm as ten inches with the unit name', () => {
    expect(fmtLen(254, 'in')).toBe('10.000 in');
  });
  it('shows 15 mm as 1.50 cm and can leave the unit name off', () => {
    expect(fmtLen(15, 'cm')).toBe('1.50 cm');
    expect(fmtLen(15, 'cm', false)).toBe('1.50');
  });
  it('shows a square inch of area as 1.00 in2', () => {
    expect(fmtArea(25.4 * 25.4, 'in')).toBe('1.00 in2');
  });
  it('shows 250 square mm as 2.5 cm2', () => {
    expect(fmtArea(250, 'cm')).toBe('2.5 cm2');
  });
  it('never prints a negative zero', () => {
    expect(fmtNum(-0.0001, 2)).toBe('0.00');
    expect(fmtNum(-0.4, 0)).toBe('0');
    expect(fmtNum(-1.25, 1)).toMatch(/^-1\.[23]$/);
  });
  it('uses more decimal places for larger units', () => {
    expect(decimals('mm')).toBeLessThan(decimals('cm'));
    expect(decimals('cm')).toBeLessThan(decimals('in'));
  });
});

describe('selection', () => {
  it('selecting only one panel drops the others and any picked seam', () => {
    const s = new Selection();
    s.panels.add(1);
    s.panels.add(2);
    s.seam = 7;
    s.only(3);
    expect(s.list()).toEqual([3]);
    expect(s.seam).toBeNull();
  });
  it('pruning forgets panels, points and seams that no longer exist', () => {
    const s = new Selection();
    s.panels.add(1);
    s.panels.add(2);
    s.pts = { panel: 2, idx: [0] };
    s.seam = 5;
    s.prune(new Set([1]), new Set([9]));
    expect(s.list()).toEqual([1]);
    expect(s.pts).toBeNull();
    expect(s.seam).toBeNull();
  });
  it('pruning keeps point picks while the outline keeps its point count and drops them when it changes', () => {
    const s = new Selection();
    s.pts = { panel: 1, idx: [2] };
    s.prune(new Set([1]), undefined, undefined, new Map([[1, 4]]));
    s.prune(new Set([1]), undefined, undefined, new Map([[1, 4]]));
    expect(s.pts).toEqual({ panel: 1, idx: [2] });
    s.prune(new Set([1]), undefined, undefined, new Map([[1, 5]]));
    expect(s.pts).toBeNull();
  });
  it('clear leaves nothing selected', () => {
    const s = new Selection();
    s.only(4);
    s.tack = 2;
    s.clear();
    expect(s.list()).toEqual([]);
    expect(s.tack).toBeNull();
  });
});

describe('panel placement maths', () => {
  it('a quarter turn carries the x axis onto the y axis and then adds the position', () => {
    const r = place({ rotation: 90, mirror: false, position: [100, 50] }, [10, 0]);
    expect(r[0]).toBeCloseTo(100, 9);
    expect(r[1]).toBeCloseTo(60, 9);
  });
  it('mirroring flips x before rotating', () => {
    const r = placeVec({ rotation: 0, mirror: true, position: [0, 0] }, [10, 3]);
    expect(r).toEqual([-10, 3]);
  });
  it('unplace undoes place for rotated and mirrored panels', () => {
    for (const mirror of [false, true]) {
      for (const rotation of [0, 33, 90, 215]) {
        const pl = { rotation, mirror, position: [12, -7] as Vec };
        const back = unplace(pl, place(pl, [3.5, -8.25]));
        expect(back[0]).toBeCloseTo(3.5, 9);
        expect(back[1]).toBeCloseTo(-8.25, 9);
      }
    }
  });
  it('placement keeps distances between points', () => {
    const pl = { rotation: 71, mirror: true, position: [5, 5] as Vec };
    const a = place(pl, [0, 0]);
    const b = place(pl, [3, 4]);
    expect(Math.hypot(a[0] - b[0], a[1] - b[1])).toBeCloseTo(5, 9);
  });
});

describe('edge geometry', () => {
  it('a closed square outline has four straight edges whose lengths add up to the perimeter', () => {
    const p = square(1, 30, [0, 0]);
    const segs = segsOf(p, p.outline, true);
    expect(segs).toHaveLength(4);
    expect(segs.reduce((n, s) => n + segLength(s), 0)).toBeCloseTo(120, 9);
  });
  it('an open line has one edge fewer than a closed one', () => {
    const p = square(1, 30, [0, 0]);
    expect(segsOf(p, p.outline, false)).toHaveLength(3);
  });
  it('a quarter circle drawn as a cubic curve measures a quarter of the circumference', () => {
    const k = 0.5522847498 * 100;
    const p = square(1, 1, [0, 0]);
    const pts: Pt[] = [
      { p: [100, 0], kind: 'curve', h_in: null, h_out: [0, k] },
      { p: [0, 100], kind: 'curve', h_in: [k, 0], h_out: null },
    ];
    const s = segsOf(p, pts, false)[0];
    expect(s.curved).toBe(true);
    expect(Math.abs(segLength(s) - Math.PI * 50)).toBeLessThan(0.3);
    const mid = bezAt(s, 0.5);
    expect(Math.hypot(mid[0], mid[1])).toBeCloseTo(100, 1);
  });
  it('a curve starts and ends on its end points', () => {
    const p = square(1, 1, [0, 0]);
    const pts: Pt[] = [{ p: [0, 0], kind: 'curve', h_in: null, h_out: [10, 20] }, { p: [50, 0], kind: 'curve', h_in: [-10, 20], h_out: null }];
    const s = segsOf(p, pts, false)[0];
    expect(bezAt(s, 0)).toEqual([0, 0]);
    expect(bezAt(s, 1)).toEqual([50, 0]);
  });
  it('the nearest point on a straight edge is the perpendicular foot', () => {
    const p = square(1, 10, [0, 0]);
    const s = segsOf(p, p.outline, true)[0];
    const r = nearestOnSeg(s, [4, 3]);
    expect(r.t).toBeCloseTo(0.4, 9);
    expect(r.pt).toEqual([4, 0]);
    expect(r.d).toBeCloseTo(3, 9);
  });
  it('the nearest point is clamped to the end of the edge', () => {
    const p = square(1, 10, [0, 0]);
    const s = segsOf(p, p.outline, true)[0];
    const r = nearestOnSeg(s, [13, 4]);
    expect(r.t).toBe(1);
    expect(r.d).toBeCloseTo(5, 9);
  });
  it('the nearest point on a quarter circle to its centre is one radius away', () => {
    const k = 0.5522847498 * 100;
    const p = square(1, 1, [0, 0]);
    const s = segsOf(p, [{ p: [100, 0], kind: 'curve', h_in: null, h_out: [0, k] }, { p: [0, 100], kind: 'curve', h_in: [k, 0], h_out: null }], false)[0];
    const r = nearestOnSeg(s, [200, 200]);
    expect(r.t).toBeCloseTo(0.5, 2);
    expect(r.d).toBeCloseTo(Math.hypot(200, 200) - 100, 1);
  });
  it('bounds cover every point and a union covers both boxes', () => {
    expect(boundsOf([[3, -2], [-1, 5], [4, 0]])).toEqual([-1, -2, 4, 5]);
    expect(unionBounds([0, 0, 1, 1], [-2, 0.5, 0.5, 3])).toEqual([-2, 0, 1, 3]);
    expect(unionBounds(null, [1, 2, 3, 4])).toEqual([1, 2, 3, 4]);
  });
  it('a point is inside a concave outline only where the outline covers it', () => {
    const ell: Vec[] = [[0, 0], [10, 0], [10, 4], [4, 4], [4, 10], [0, 10]];
    expect(pointInPoly([2, 8], ell)).toBe(true);
    expect(pointInPoly([8, 2], ell)).toBe(true);
    expect(pointInPoly([8, 8], ell)).toBe(false);
    expect(pointInPoly([-1, 5], ell)).toBe(false);
  });
});

describe('panel hit testing', () => {
  it('a placed panel reports its bounds and centre in world space', () => {
    const g = geoOf(square(1, 20, [100, 200]));
    expect(g.bounds).toEqual([100, 200, 120, 220]);
    expect(g.center).toEqual([110, 210]);
  });
  it('a click near a corner picks that corner point', () => {
    const g = geoOf(square(1, 20, [0, 0]));
    const h = hitTest([g], [20.5, 19.5], 2, { handles: true, points: true, segments: true, panels: true });
    expect(h?.kind).toBe('point');
    expect(h?.index).toBe(2);
  });
  it('a click near the middle of an edge picks the edge and where along it', () => {
    const g = geoOf(square(1, 20, [0, 0]));
    const h = hitTest([g], [20.5, 5], 2, { handles: true, points: true, segments: true, panels: true });
    expect(h?.kind).toBe('segment');
    expect(h?.index).toBe(1);
    expect(h?.t).toBeCloseTo(0.25, 9);
  });
  it('a click inside overlapping panels picks the one on the higher layer', () => {
    const low = geoOf(square(1, 20, [0, 0], 0));
    const high = geoOf(square(2, 20, [5, 5], 3));
    const h = hitTest([low, high], [10, 10], 1, { handles: false, points: false, segments: false, panels: true });
    expect(h).toMatchObject({ kind: 'panel', panel: 2 });
  });
  it('a click on empty space picks nothing', () => {
    const g = geoOf(square(1, 20, [0, 0]));
    expect(hitTest([g], [50, 50], 2, { handles: true, points: true, segments: true, panels: true })).toBeNull();
  });
  it('points are ignored when point picking is off', () => {
    const g = geoOf(square(1, 20, [0, 0]));
    const h = hitTest([g], [20, 20], 2, { handles: false, points: false, segments: false, panels: true });
    expect(h).toBeNull();
  });
});
