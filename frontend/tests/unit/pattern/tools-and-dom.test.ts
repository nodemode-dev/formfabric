import { describe, it, expect, vi } from 'vitest';
import { geoOf } from '../../../src/workspace/geom';
import { Selection } from '../../../src/workspace/state';
import type { DocStore } from '../../../src/workspace/doc';
import type { Vec } from '../../../src/workspace/types';
import type { Ptr } from '../../../src/workspace/pattern-view/ctx';
import type { PatternView } from '../../../src/workspace/pattern-view/view';
import { constrain, moved, nearestVertex, normalAt } from '../../../src/workspace/pattern-view/tools-common';
import { gridStep, panelAt } from '../../../src/workspace/pattern-view/render';
import { mmText } from '../../../src/workspace/pattern-view/render-sew';
import { SeamCheck } from '../../../src/workspace/pattern-view/seam-check';
import type { Seam, SeamEnd, SewSnapshot } from '../../../src/workspace/pattern-view/sew-types';
import { StyleLineStore, segmentDistance, type StyleLineInfo } from '../../../src/workspace/styleline/store';
import { PortPicker } from '../../../src/workspace/components/pick';
import { openInner, quarterDisc, square } from './fixtures';

const ptr = (sx: number, sy: number): Ptr => ({ sx, sy } as unknown as Ptr);

describe('constraining a drag', () => {
  it('leaves the point alone without modifier keys', () => {
    expect(constrain([1, 2], [10, 9], { shift: false, mod: false })).toEqual([10, 9]);
  });

  it('locks to the horizontal axis when the drag is mostly sideways', () => {
    expect(constrain([1, 2], [10, 9], { shift: false, mod: true })).toEqual([10, 2]);
  });

  it('locks to the vertical axis when the drag is mostly up or down', () => {
    expect(constrain([1, 2], [4, 30], { shift: false, mod: true })).toEqual([1, 30]);
  });

  it('snaps the direction to 45 degrees and keeps the drag length with shift', () => {
    const r = constrain([0, 0], [10, 9], { shift: true, mod: false });
    const l = Math.sqrt(181);
    expect(r[0]).toBeCloseTo(l / Math.SQRT2, 9);
    expect(r[1]).toBeCloseTo(l / Math.SQRT2, 9);
  });

  it('snaps a nearly vertical drag straight down the axis with shift', () => {
    const r = constrain([5, 5], [6, -15], { shift: true, mod: false });
    expect(r[0]).toBeCloseTo(5, 9);
    expect(r[1]).toBeCloseTo(5 - Math.hypot(1, 20), 9);
  });

  it('counts a 5 pixel move as a drag and a 1 pixel move as a click', () => {
    expect(moved(ptr(0, 0), ptr(3, 4))).toBe(true);
    expect(moved(ptr(0, 0), ptr(1, 0))).toBe(false);
  });

  it('returns the closest vertex of a list and nothing for an empty list', () => {
    const pts: Vec[] = [[0, 0], [10, 0], [10, 10]];
    expect(nearestVertex(pts, [9, 7])).toEqual([10, 10]);
    expect(nearestVertex([], [9, 7])).toBeNull();
  });
});

describe('edge normals', () => {
  it('point away from the inside on every edge of a square, at unit length', () => {
    const geo = geoOf(square(1, [10, 20], 100));
    const expected: Vec[] = [[0, -1], [1, 0], [0, 1], [-1, 0]];
    expected.forEach((e, i) => {
      const n = normalAt(geo, i, 0.5);
      expect(n[0]).toBeCloseTo(e[0], 9);
      expect(n[1]).toBeCloseTo(e[1], 9);
    });
  });

  it('point radially outward on a quarter circle edge', () => {
    const geo = geoOf(quarterDisc(1, 100));
    const n = normalAt(geo, 0, 0.5);
    expect(n[0]).toBeCloseTo(Math.SQRT1_2, 2);
    expect(n[1]).toBeCloseTo(Math.SQRT1_2, 2);
    expect(Math.hypot(n[0], n[1])).toBeCloseTo(1, 9);
  });

  it('still point outward when the panel is mirrored', () => {
    const geo = geoOf(square(1, [0, 0], 100, { mirror: true }));
    const n = normalAt(geo, 1, 0.5);
    expect(n[0]).toBeCloseTo(-1, 9);
    expect(n[1]).toBeCloseTo(0, 9);
  });
});

describe('grid and panel lookup', () => {
  it('keeps grid lines at least 10 pixels apart with a 1, 2, 5 step at every zoom', () => {
    for (const s of [0.003, 0.05, 0.3, 1, 2.4, 7, 40, 900]) {
      const g = gridStep(s);
      expect(g.minor * s).toBeGreaterThanOrEqual(10);
      const mant = g.minor / 10 ** Math.floor(Math.log10(g.minor) + 1e-9);
      expect([1, 2, 5]).toContain(Math.round(mant));
      expect(Math.abs(mant - Math.round(mant))).toBeLessThan(1e-6);
      const ratio = g.major / g.minor;
      expect(Math.abs(ratio - Math.round(ratio))).toBeLessThan(1e-6);
      expect(g.major * 1).toBeCloseTo(10 ** Math.round(Math.log10(g.major)), 6);
    }
  });

  it('uses a 10 mm grid with 100 mm major lines at one pixel per millimetre', () => {
    expect(gridStep(1)).toEqual({ minor: 10, major: 100 });
  });

  it('finds the panel under a point, the higher layer first, and nothing outside', () => {
    const low = geoOf(square(1, [0, 0], 100, { layer: 0 }));
    const high = geoOf(square(2, [50, 50], 100, { layer: 3 }));
    expect(panelAt([low, high], [75, 75])).toBe(2);
    expect(panelAt([high, low], [75, 75])).toBe(2);
    expect(panelAt([low, high], [25, 25])).toBe(1);
    expect(panelAt([low, high], [140, 140])).toBe(2);
    expect(panelAt([low, high], [200, 10])).toBeNull();
  });

  it('writes a seam mismatch with a plus sign when longer and a minus sign when shorter', () => {
    expect(mmText(2.5)).toMatch(/^\+2\.50* mm$/);
    expect(mmText(-2.5)).toMatch(/^-2\.50* mm$/);
    expect(mmText(25.4, 'in')).toMatch(/^\+1\.0*\d? in$/);
  });
});

describe('style line hit testing', () => {
  const line = (id: number, panel: number, points: Vec[], closed: boolean): StyleLineInfo => ({ id, panel, closed, mode: 'none', stiffen: 8, width: 12, fold_angle: 150, fold_strength: 0.5, points, length: 0 });
  const shift = (panel: number, p: Vec): Vec => [p[0] + panel * 1000, p[1]];

  it('measures the distance from a point to a segment like a 3-4-5 triangle', () => {
    expect(segmentDistance([0, 0], [10, 0], [5, 4])).toBeCloseTo(4, 12);
    expect(segmentDistance([0, 0], [10, 0], [13, 4])).toBeCloseTo(5, 12);
    expect(segmentDistance([0, 0], [10, 0], [-3, -4])).toBeCloseTo(5, 12);
    expect(segmentDistance([2, 2], [2, 2], [5, 6])).toBeCloseTo(5, 12);
  });

  it('hits the nearest line inside the radius in world space', () => {
    const store = new StyleLineStore();
    store.lines = [line(1, 0, [[0, 0], [100, 0]], false), line(2, 1, [[0, 0], [100, 0]], false), line(3, 0, [[0, 6], [100, 6]], false)];
    expect(store.hit([50, 2], shift, 5)?.id).toBe(1);
    expect(store.hit([50, 5], shift, 5)?.id).toBe(3);
    expect(store.hit([1050, 3], shift, 5)?.id).toBe(2);
    expect(store.hit([50, 30], shift, 5)).toBeNull();
  });

  it('hits the closing edge of a closed line but not of an open one', () => {
    const tri: Vec[] = [[0, 0], [100, 0], [100, 100]];
    const store = new StyleLineStore();
    store.lines = [line(1, 0, tri, true)];
    expect(store.hit([50, 51], shift, 3)?.id).toBe(1);
    store.lines = [line(1, 0, tri, false)];
    expect(store.hit([50, 51], shift, 3)).toBeNull();
  });

  it('tells listeners once per real selection change and stops after unsubscribing', () => {
    const store = new StyleLineStore();
    store.lines = [line(1, 0, [[0, 0], [1, 1]], false)];
    const heard = vi.fn();
    const off = store.onChange(heard);
    store.select(1);
    store.select(1);
    expect(heard).toHaveBeenCalledTimes(1);
    expect(store.current?.id).toBe(1);
    store.select(null);
    expect(store.current).toBeNull();
    off();
    store.select(1);
    expect(heard).toHaveBeenCalledTimes(2);
  });
});

const end = (panel: number, segment: number): SeamEnd => ({ panel, inner: null, segment, t0: 0, t1: 1 });
const seam = (id: number, a: number, b: number): Seam => ({ id, a: [end(a, 0)], b: [end(b, 0)], reversed: false, fold: 'turned', fold_angle: 180, fold_strength: 1, ratio: 1, tension: 1, side: 'auto' });

function fakeDoc(sew: SewSnapshot): DocStore {
  const panels = [square(1, [0, 0], 100, { name: 'Front', inner: [openInner(7, [20, 50], [80, 50])] }), square(2, [300, 0], 100, { name: 'Back' })];
  const doc = { sew, garment: { units: 'mm', panels }, panel: (id: number) => panels.find((p) => p.id === id), geos: () => panels.map(geoOf) };
  return doc as unknown as DocStore;
}

describe('seam check list', () => {
  const sewBad: SewSnapshot = { seams: [seam(1, 1, 2), seam(2, 2, 1), seam(3, 1, 9)], tacks: [], info: new Map(), bad: new Map([[2, -3], [3, 4]]) };

  it('lists only the seams with a length mismatch, with panel names and a signed amount', () => {
    localStorage.clear();
    const host = document.createElement('div');
    new SeamCheck(host, fakeDoc(sewBad), new Selection(), () => undefined, () => undefined);
    const rows = [...host.querySelectorAll('.ff-sewcheck-row')];
    expect(rows).toHaveLength(2);
    expect(rows[0].querySelector('[data-k="n"]')?.textContent).toBe('#2  Back / Front');
    expect(rows[0].querySelector('[data-k="m"]')?.textContent).toMatch(/^-3\.?0* mm$/);
    expect(rows[1].querySelector('[data-k="n"]')?.textContent).toBe('#3  Front / 9');
    expect(rows[1].querySelector('[data-k="m"]')?.textContent).toMatch(/^\+4\.?0* mm$/);
    expect(host.querySelector('[data-k="count"]')?.textContent).toBe('2');
    expect(host.querySelector('[data-k="note"]')?.textContent).toBe('');
  });

  it('says all seams match when none is off and no seams when there are none', () => {
    localStorage.clear();
    const host = document.createElement('div');
    new SeamCheck(host, fakeDoc({ seams: [seam(1, 1, 2)], tacks: [], info: new Map(), bad: new Map() }), new Selection(), () => undefined, () => undefined);
    expect(host.querySelectorAll('.ff-sewcheck-row')).toHaveLength(0);
    expect(host.querySelector('[data-k="count"]')?.textContent).toBe('0');
    expect(host.querySelector('[data-k="note"]')?.textContent).toBe('All seams match');
    const empty = document.createElement('div');
    new SeamCheck(empty, fakeDoc({ seams: [], tacks: [], info: new Map(), bad: new Map() }), new Selection(), () => undefined, () => undefined);
    expect(empty.querySelector('[data-k="note"]')?.textContent).toBe('No seams');
  });

  it('says the request failed when the seam check could not be fetched', () => {
    localStorage.clear();
    const host = document.createElement('div');
    new SeamCheck(host, fakeDoc({ seams: [seam(1, 1, 2)], tacks: [], info: new Map(), bad: new Map(), failed: true }), new Selection(), () => undefined, () => undefined);
    expect(host.querySelector('[data-k="note"]')?.textContent).toBe('The request failed');
  });

  it('selects the seam of a clicked row, drops the tack selection and marks the row', () => {
    localStorage.clear();
    const host = document.createElement('div');
    const sel = new Selection();
    sel.tack = 5;
    const picked = vi.fn();
    const check = new SeamCheck(host, fakeDoc(sewBad), sel, picked, () => undefined);
    const row = host.querySelectorAll<HTMLElement>('.ff-sewcheck-row')[1];
    row.querySelector<HTMLElement>('[data-k="m"]')?.click();
    expect(sel.seam).toBe(3);
    expect(sel.tack).toBeNull();
    expect(picked).toHaveBeenCalledTimes(1);
    check.update();
    const rows = host.querySelectorAll('.ff-sewcheck-row');
    expect(rows[1].classList.contains('is-on')).toBe(true);
    expect(rows[0].classList.contains('is-on')).toBe(false);
  });

  it('opens and closes on toggle and comes back open in a new session', () => {
    localStorage.clear();
    const host = document.createElement('div');
    const changed = vi.fn();
    const check = new SeamCheck(host, fakeDoc(sewBad), new Selection(), () => undefined, changed);
    expect(check.open).toBe(false);
    expect(check.node.classList.contains('is-open')).toBe(false);
    check.toggle();
    expect(check.node.classList.contains('is-open')).toBe(true);
    expect(changed).toHaveBeenLastCalledWith(true);
    const again = new SeamCheck(document.createElement('div'), fakeDoc(sewBad), new Selection(), () => undefined, () => undefined);
    expect(again.open).toBe(true);
    check.toggle();
    expect(check.node.classList.contains('is-open')).toBe(false);
    expect(changed).toHaveBeenLastCalledWith(false);
    localStorage.clear();
  });
});

describe('component port picker', () => {
  function setup(): { pane: HTMLElement; picker: PortPicker; changes: () => number } {
    const pane = document.createElement('div');
    const canvas = document.createElement('canvas');
    canvas.className = 'ff-canvas';
    pane.appendChild(canvas);
    document.body.appendChild(pane);
    const view = { toWorld: (x: number, y: number): Vec => [x, y], toScreen: (w: Vec): Vec => w, tol: (px: number): number => px, cam: { x: 0, y: 0, s: 1 } };
    const sew: SewSnapshot = { seams: [], tacks: [], info: new Map(), bad: new Map() };
    const picker = new PortPicker(pane, view as unknown as PatternView, fakeDoc(sew));
    let n = 0;
    picker.onChange = () => { n += 1; };
    return { pane, picker, changes: () => n };
  }

  const press = (pane: HTMLElement, x: number, y: number, button = 0): void => {
    pane.dispatchEvent(new MouseEvent('pointerdown', { clientX: x, clientY: y, button, bubbles: true, cancelable: true }));
  };

  it('adds up port length from edge lengths times the covered fraction', () => {
    const { picker } = setup();
    expect(picker.lengthOf([{ panel: 1, segment: 0, t0: 0, t1: 1 }])).toBeCloseTo(100, 6);
    expect(picker.lengthOf([{ panel: 1, segment: 0, t0: 0, t1: 1 }, { panel: 2, segment: 3, t0: 0.25, t1: 0.75 }])).toBeCloseTo(150, 6);
    expect(picker.lengthOf([{ panel: 8, segment: 0, t0: 0, t1: 1 }])).toBe(0);
  });

  it('adds a clicked outline edge to the port and removes it on a second click', () => {
    const { pane, picker } = setup();
    picker.start();
    press(pane, 50, 3);
    expect(picker.current).toEqual([{ panel: 1, segment: 0, t0: 0, t1: 1 }]);
    press(pane, 398, 50);
    expect(picker.current.map((s) => [s.panel, s.segment])).toEqual([[1, 0], [2, 1]]);
    press(pane, 60, 2);
    expect(picker.current.map((s) => [s.panel, s.segment])).toEqual([[2, 1]]);
    picker.stop();
  });

  it('ignores clicks on internal lines, on empty space and with other buttons', () => {
    const { pane, picker } = setup();
    picker.start();
    press(pane, 50, 51);
    press(pane, 200, 200);
    press(pane, 50, 3, 2);
    expect(picker.current).toEqual([]);
    picker.stop();
  });

  it('stops listening for clicks once picking is stopped', () => {
    const { pane, picker } = setup();
    picker.start();
    picker.stop();
    press(pane, 50, 3);
    expect(picker.current).toEqual([]);
  });

  it('shows its overlay and marks the pane only while picking, and Escape ends it', () => {
    const { pane, picker } = setup();
    const overlay = pane.querySelector<HTMLCanvasElement>('canvas.ff-lib-pick');
    expect(overlay?.hidden).toBe(true);
    picker.start();
    expect(picker.active).toBe(true);
    expect(overlay?.hidden).toBe(false);
    expect(pane.classList.contains('ff-lib-picking')).toBe(true);
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', cancelable: true }));
    expect(picker.active).toBe(false);
    expect(overlay?.hidden).toBe(true);
    expect(pane.classList.contains('ff-lib-picking')).toBe(false);
  });

  it('commits the picked edges as a named port, refuses an empty one and removes by name', () => {
    const { pane, picker, changes } = setup();
    expect(picker.commit('hem')).toBe(false);
    picker.start();
    press(pane, 50, 3);
    picker.stop();
    const before = changes();
    expect(picker.commit('hem')).toBe(true);
    expect(changes()).toBe(before + 1);
    expect(picker.current).toEqual([]);
    expect(picker.ports).toEqual([{ name: 'hem', spans: [{ panel: 1, segment: 0, t0: 0, t1: 1 }] }]);
    expect(picker.lengthOf(picker.ports[0].spans)).toBeCloseTo(100, 6);
    picker.removePort('other');
    expect(picker.ports).toHaveLength(1);
    picker.removePort('hem');
    expect(picker.ports).toEqual([]);
    picker.reset();
    expect(picker.current).toEqual([]);
  });
});
