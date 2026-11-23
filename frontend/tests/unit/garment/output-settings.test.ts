import { describe, it, expect, beforeEach } from 'vitest';
import { humanize, parseFormats, parseUv, parseJob, parseRange, parseRetopo } from '../../../src/workspace/output/store';
import { recall, remember, swapExt } from '../../../src/workspace/output/files';

describe('export format lists', () => {
  it('a plain list of format names becomes formats with matching file extensions', () => {
    const f = parseFormats(['obj', 'dxf']);
    expect(f.map((x) => x.id)).toEqual(['obj', 'dxf']);
    expect(f.map((x) => x.ext)).toEqual(['obj', 'dxf']);
    expect(f[0].label).toBe('OBJ');
  });
  it('a format marked unavailable is not offered', () => {
    const f = parseFormats({ formats: [{ id: 'obj' }, { id: 'fbx', available: false }] });
    expect(f.map((x) => x.id)).toEqual(['obj']);
  });
  it('a leading dot on the extension is dropped', () => {
    expect(parseFormats([{ id: 'pattern', ext: '.dxf', label: 'Pattern' }])[0]).toMatchObject({ id: 'pattern', ext: 'dxf', label: 'Pattern' });
  });
  it('a map of formats keyed by name keeps each key as the format id', () => {
    const f = parseFormats({ formats: { gltf: { label: 'glTF' }, usd: {} } });
    expect(f.map((x) => [x.id, x.label])).toEqual([['gltf', 'glTF'], ['usd', 'USD']]);
  });
  it('JSON Schema option types, limits, choices and defaults are read as the schema states them', () => {
    const f = parseFormats([{
      id: 'obj',
      options: {
        type: 'object',
        properties: {
          weld: { type: 'boolean', default: true, description: 'Weld seams' },
          scale: { type: 'number', minimum: 0.001, maximum: 1000, default: 1 },
          subdiv: { type: 'integer', minimum: 0 },
          axis: { enum: ['y_up', 'z_up'], default: 'z_up' },
          name: { type: ['string', 'null'] },
        },
      },
    }]);
    const by = new Map(f[0].opts.map((o) => [o.key, o]));
    expect(by.get('weld')).toMatchObject({ kind: 'boolean', def: true, hint: 'Weld seams' });
    expect(by.get('scale')).toMatchObject({ kind: 'number', min: 0.001, max: 1000, def: 1 });
    expect(by.get('subdiv')).toMatchObject({ kind: 'integer', min: 0, max: null });
    expect(by.get('axis')?.kind).toBe('enum');
    expect(by.get('axis')?.choices.map((c) => c[0])).toEqual(['y_up', 'z_up']);
    expect(by.get('axis')?.def).toBe('z_up');
    expect(by.get('name')?.kind).toBe('string');
    expect(f[0].opts).toHaveLength(5);
  });
  it('a reply that is not a list or a map offers no formats', () => {
    expect(parseFormats(null)).toEqual([]);
    expect(parseFormats('obj')).toEqual([]);
  });
  it('option names are shown as words with known abbreviations in capitals', () => {
    expect(humanize('uv_padding')).toBe('UV padding');
    expect(humanize('frame_rate')).toBe('Frame rate');
    expect(humanize('export.fbx_units')).toBe('Export FBX units');
  });
});

describe('UV layout state', () => {
  it('a 2 by 3 tile grid has six tiles in two columns', () => {
    expect(parseUv({ tiles: [2, 3], islands: [] })).toMatchObject({ tiles: 6, cols: 2 });
    expect(parseUv({ tiles: { u: 2, v: 3 } })).toMatchObject({ tiles: 6, cols: 2 });
  });
  it('the tile count grows to hold an island placed on a later tile', () => {
    const uv = parseUv({ tiles: 1, islands: [{ panel: 4, tile: 3 }] });
    expect(uv.tiles).toBe(4);
    expect(uv.islands[0].name).toBe('#4');
  });
  it('an island given only as a bounding box gets its four corners as an outline', () => {
    const uv = parseUv({ islands: [{ panel: 1, bbox: [0.1, 0.2, 0.5, 0.9] }] });
    expect(uv.islands[0].loops).toEqual([[[0.1, 0.2], [0.5, 0.2], [0.5, 0.9], [0.1, 0.9]]]);
  });
  it('island outlines given as one loop or as several loops both come through point for point', () => {
    const one = parseUv({ islands: [{ panel: 1, outline: [[0, 0], [1, 0], [1, 1]] }] });
    const many = parseUv({ islands: [{ panel: 1, loops: [[[0, 0], [1, 0], [1, 1]], [[2, 2], [3, 2], [3, 3]]] }] });
    expect(one.islands[0].loops).toEqual([[[0, 0], [1, 0], [1, 1]]]);
    expect(many.islands[0].loops).toHaveLength(2);
    expect(many.islands[0].loops[1][2]).toEqual([3, 3]);
  });
  it('a missing reply gives an empty single tile layout', () => {
    expect(parseUv(undefined)).toMatchObject({ islands: [], tiles: 1 });
  });
});

describe('job progress and frame ranges', () => {
  it('progress given as a percentage becomes a fraction of one', () => {
    expect(parseJob({ running: true, percent: 50 }).progress).toBeCloseTo(0.5, 12);
    expect(parseJob({ progress: 250 }).progress).toBe(1);
  });
  it('progress given as a fraction is kept and never goes below zero', () => {
    expect(parseJob({ progress: 0.25 }).progress).toBe(0.25);
    expect(parseJob({ progress: -0.5 }).progress).toBe(0);
  });
  it('a job is running when its state says so and idle otherwise', () => {
    expect(parseJob({ state: 'running', message: 'Baking' })).toMatchObject({ running: true, label: 'Baking' });
    expect(parseJob({ state: 'done' }).running).toBe(false);
    expect(parseJob(null)).toEqual({ running: false, progress: null, label: '' });
  });
  it('a cache of 5 frames starting at frame 10 ends on frame 14', () => {
    expect(parseRange({ caches: [{ id: 1, start: 10, frames: 5 }], active_cache: 1 })).toEqual({ start: 10, end: 14, fromCache: true });
  });
  it('the active cache wins over other caches', () => {
    const r = parseRange({ caches: [{ id: 1, start: 0, frames: 10 }, { id: 2, start: 100, frames: 2 }], active_cache: 2 });
    expect(r).toEqual({ start: 100, end: 101, fromCache: true });
  });
  it('without a cache the timeline start and end are used', () => {
    expect(parseRange({ start: 1, end: 48 })).toEqual({ start: 1, end: 48, fromCache: false });
  });
  it('an empty cache and no timeline gives no range', () => {
    expect(parseRange({ caches: [{ id: 1, start: 0, frames: 0 }] })).toBeNull();
    expect(parseRange('x')).toBeNull();
  });
});

describe('retopology metrics', () => {
  it('per panel counts are summed and the quad share is quads over all faces', () => {
    const r = parseRetopo({ mesh: 'quad', density: 0.8, panels: [{ vertices: 10, quads: 6, triangles: 1 }, { vertices: 5, quads: 3, triangles: 2, max_angle_deviation: 12.5 }] });
    const m = new Map(r.metrics.map((x) => [x.key, x.value]));
    expect(r.mesh).toBe('quad');
    expect(r.density).toBe(0.8);
    expect(m.get('panels')).toBe('2');
    expect(m.get('vertices')).toBe('15');
    expect(m.get('quads')).toBe('9');
    expect(m.get('triangles')).toBe('3');
    expect(m.get('percent_quads')).toBe('75');
    expect(m.get('max_angle_deviation')).toBe('12.5');
  });
  it('guide lines keep their panel and points', () => {
    const r = parseRetopo({ guides: [{ panel: 3, points: [[0, 0], [5, 5]] }, { panel: 4 }] });
    expect(r.guides).toEqual([{ panel: 3, points: [[0, 0], [5, 5]] }]);
  });
});

describe('export file names and remembered settings', () => {
  beforeEach(() => { localStorage.clear(); });
  it('changing format swaps the file extension whatever its letter case', () => {
    expect(swapExt('/out/shirt.OBJ', 'obj', 'fbx')).toBe('/out/shirt.fbx');
    expect(swapExt('/out/shirt.obj', 'obj', 'gltf')).toBe('/out/shirt.gltf');
  });
  it('a path with a different extension is left alone', () => {
    expect(swapExt('/out/obj.notes/shirt.dxf', 'obj', 'fbx')).toBe('/out/obj.notes/shirt.dxf');
    expect(swapExt('/out/shirtobj', 'obj', 'fbx')).toBe('/out/shirtobj');
  });
  it('a remembered setting is recalled unchanged', () => {
    const value = { format: 'obj', scale: 0.001, range: [1, 48], weld: true };
    remember('export', value);
    expect(recall('export', null)).toEqual(value);
  });
  it('a setting never remembered or stored as broken text gives the fallback', () => {
    expect(recall('missing', 7)).toBe(7);
    localStorage.setItem('formfabric.output.bad', '{not json');
    expect(recall('bad', 'fallback')).toBe('fallback');
  });
});
