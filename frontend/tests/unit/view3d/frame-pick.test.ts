import { describe, it, expect } from 'vitest';
import { OrbitCamera } from '../../../src/workspace/view3d/camera';
import { Frame, invertInto, mulInto } from '../../../src/workspace/view3d/frame';
import { invert, multiply, perspective, lookAt } from '../../../src/workspace/view3d/math';
import { Picker } from '../../../src/workspace/view3d/pick';
import type { Scene } from '../../../src/workspace/view3d/scene';

interface Buf { id: number; kind: string; ready: boolean; min: number[]; max: number[]; pos: Float32Array; idx: Uint32Array }

function quad(id: number, z: number, half = 1, kind = 'panel'): Buf {
  return {
    id, kind, ready: true,
    min: [-half, -half, z], max: [half, half, z],
    pos: new Float32Array([-half, -half, z, half, -half, z, half, half, z, -half, half, z]),
    idx: new Uint32Array([0, 1, 2, 0, 2, 3]),
  };
}

function sceneOf(...bufs: Buf[]): Scene {
  return { list: bufs, get: (id: number) => bufs.find((b) => b.id === id) } as unknown as Scene;
}

function frontFrame(distance = 5, w = 200, h = 200): { cam: OrbitCamera; f: Frame } {
  const cam = new OrbitCamera();
  cam.snap('front');
  cam.distance = distance;
  cam.aspect = w / h;
  const f = new Frame();
  f.update(cam, w, h);
  return { cam, f };
}

describe('frame matrices', () => {
  it('frame view-projection agrees with the camera view-projection in perspective', () => {
    const cam = new OrbitCamera();
    cam.target = [1, 2, 3];
    cam.orbit(40, 25);
    cam.aspect = 640 / 480;
    const f = new Frame();
    f.update(cam, 640, 480);
    const want = cam.viewProjection();
    for (let i = 0; i < 16; i++) expect(f.vp[i]).toBeCloseTo(want[i], 3);
  });

  it('frame view-projection agrees with the camera view-projection in orthographic mode', () => {
    const cam = new OrbitCamera();
    cam.orthographic = true;
    cam.aspect = 2;
    const f = new Frame();
    f.update(cam, 800, 400);
    const want = cam.viewProjection();
    for (let i = 0; i < 16; i++) expect(f.vp[i]).toBeCloseTo(want[i], 4);
    expect(f.ortho).toBe(true);
  });

  it('frame eye matches the camera eye and forward is the unit direction to the target', () => {
    const cam = new OrbitCamera();
    cam.target = [0.5, 1, -2];
    const f = new Frame();
    f.update(cam, 300, 300);
    const e = cam.eye();
    for (let i = 0; i < 3; i++) {
      expect(f.eye[i]).toBeCloseTo(e[i], 5);
      expect(f.fwd[i]).toBeCloseTo((cam.target[i] - e[i]) / cam.distance, 5);
    }
  });

  it('in-place multiply agrees with the allocating multiply', () => {
    const a = perspective(0.9, 1.3, 0.5, 80);
    const b = lookAt([1, 2, 3], [0, 0, 0], [0, 1, 0]);
    const out = new Float32Array(16);
    mulInto(out, a, b);
    expect(Array.from(out)).toEqual(Array.from(multiply(a, b)));
  });

  it('in-place inverse agrees with the allocating inverse and may alias its input', () => {
    const m = multiply(perspective(0.9, 1.3, 0.5, 80), lookAt([1, 2, 3], [0, 0, 0], [0, 1, 0]));
    const want = invert(m);
    const copy = new Float32Array(m);
    expect(invertInto(copy, copy, new Float32Array(16))).toBe(true);
    expect(Array.from(copy)).toEqual(Array.from(want as Float32Array));
  });

  it('in-place inverse refuses a singular matrix and leaves the output untouched', () => {
    const out = new Float32Array(16).fill(7);
    expect(invertInto(out, new Float32Array(16), new Float32Array(16))).toBe(false);
    expect(out[0]).toBe(7);
  });
});

describe('frame projection', () => {
  it('the orbit target projects to the viewport centre at a depth equal to the camera distance', () => {
    const { f } = frontFrame(5, 320, 200);
    const out = new Float64Array(3);
    expect(f.project(0, 0, 0, out)).toBe(true);
    expect(out[0]).toBeCloseTo(160, 3);
    expect(out[1]).toBeCloseTo(100, 3);
    expect(out[2]).toBeCloseTo(5, 4);
  });

  it('a point half the visible height above the target projects to the top edge', () => {
    const { cam, f } = frontFrame(5, 200, 200);
    const out = new Float64Array(3);
    f.project(0, 5 * Math.tan(cam.fov / 2), 0, out);
    expect(out[1]).toBeCloseTo(0, 2);
    expect(out[0]).toBeCloseTo(100, 3);
  });

  it('a point to the right in the world lands right of centre and a higher point lands higher on screen', () => {
    const { f } = frontFrame();
    const out = new Float64Array(3);
    f.project(0.5, 0.5, 0, out);
    expect(out[0]).toBeGreaterThan(100);
    expect(out[1]).toBeLessThan(100);
  });

  it('a point behind the camera is refused', () => {
    const { f } = frontFrame(5);
    expect(f.project(0, 0, 9, new Float64Array(3))).toBe(false);
  });

  it('project then ray round trip passes back through the world point', () => {
    const cam = new OrbitCamera();
    cam.orbit(-60, 35);
    const f = new Frame();
    f.update(cam, 640, 480);
    const p = [0.7, -0.3, 0.45];
    const s = new Float64Array(3);
    expect(f.project(p[0], p[1], p[2], s)).toBe(true);
    const o = new Float64Array(3);
    const d = new Float64Array(3);
    expect(f.ray(s[0], s[1], o, d)).toBe(true);
    const v = [p[0] - o[0], p[1] - o[1], p[2] - o[2]];
    const c = [v[1] * d[2] - v[2] * d[1], v[2] * d[0] - v[0] * d[2], v[0] * d[1] - v[1] * d[0]];
    expect(Math.hypot(c[0], c[1], c[2])).toBeLessThan(5e-3);
    expect(Math.hypot(d[0], d[1], d[2])).toBeCloseTo(1, 9);
  });
});

describe('picking', () => {
  it('clicking the centre hits the panel in front of the camera at the eye-to-panel distance', () => {
    const { f } = frontFrame(5);
    const picker = new Picker();
    expect(picker.pick(f, sceneOf(quad(7, 0)), 100, 100)).toBe(7);
    expect(picker.panel).toBe(7);
    expect(picker.distance).toBeGreaterThan(4.9);
    expect(picker.distance).toBeLessThan(5.01);
  });

  it('the nearer of two overlapping panels wins whatever their order in the scene', () => {
    const { f } = frontFrame(5);
    const picker = new Picker();
    expect(picker.pick(f, sceneOf(quad(1, -1), quad(2, 1)), 100, 100)).toBe(2);
    expect(picker.pick(f, sceneOf(quad(2, 1), quad(1, -1)), 100, 100)).toBe(2);
  });

  it('clicking beside the panel hits nothing', () => {
    const { f } = frontFrame(5);
    const picker = new Picker();
    expect(picker.pick(f, sceneOf(quad(7, 0, 0.2)), 5, 5)).toBe(-1);
    expect(picker.vertex).toBe(-1);
  });

  it('a panel behind the camera is not hit', () => {
    const { f } = frontFrame(5);
    expect(new Picker().pick(f, sceneOf(quad(7, 9)), 100, 100)).toBe(-1);
  });

  it('avatar bodies and panels that are not ready are never picked', () => {
    const { f } = frontFrame(5);
    const body = quad(3, 1, 1, 'body');
    const pending = quad(4, 1.5);
    pending.ready = false;
    expect(new Picker().pick(f, sceneOf(body, pending, quad(5, 0)), 100, 100)).toBe(5);
  });

  it('the picked vertex is the corner nearest to the clicked point', () => {
    const { f } = frontFrame(5);
    const scene = sceneOf(quad(7, 0));
    const picker = new Picker();
    const s = new Float64Array(3);
    const cases: Array<[number, number, number]> = [[0.8, -0.7, 1], [0.7, 0.8, 2], [-0.8, 0.7, 3], [-0.8, -0.7, 0]];
    for (const [x, y, vertex] of cases) {
      f.project(x, y, 0, s);
      expect(picker.pick(f, scene, s[0], s[1])).toBe(7);
      expect(picker.vertex).toBe(vertex);
    }
  });

  it('picking works the same in orthographic mode', () => {
    const cam = new OrbitCamera();
    cam.snap('front');
    cam.orthographic = true;
    const f = new Frame();
    f.update(cam, 200, 200);
    const picker = new Picker();
    expect(picker.pick(f, sceneOf(quad(1, -1), quad(2, 1)), 100, 100)).toBe(2);
    expect(picker.pick(f, sceneOf(quad(1, 0, 0.1)), 2, 2)).toBe(-1);
  });
});
