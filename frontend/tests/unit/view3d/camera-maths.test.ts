import { describe, it, expect } from 'vitest';
import { OrbitCamera } from '../../../src/workspace/view3d/camera';
import { cross, distance3, dot, invert, length, lookAt, multiply, normalize, ortho, perspective, sub, transformPoint, type Mat4, type Vec3 } from '../../../src/workspace/view3d/math';

const IDENTITY = new Float32Array([1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1]);

function expectVec(got: ArrayLike<number>, want: number[], digits = 4): void {
  for (let i = 0; i < want.length; i++) expect(got[i]).toBeCloseTo(want[i], digits);
}

function ndc(m: Mat4, p: Vec3): number[] {
  const c = transformPoint(m, p);
  return [c[0] / c[3], c[1] / c[3], c[2] / c[3]];
}

function front(distance = 5): OrbitCamera {
  const cam = new OrbitCamera();
  cam.snap('front');
  cam.distance = distance;
  return cam;
}

describe('vector maths', () => {
  it('cross product of x and y axes is the z axis', () => {
    expect(cross([1, 0, 0], [0, 1, 0])).toEqual([0, 0, 1]);
  });

  it('cross product is perpendicular to both inputs', () => {
    const a: Vec3 = [1, 2, 3];
    const b: Vec3 = [-4, 0.5, 2];
    const c = cross(a, b);
    expect(dot(c, a)).toBeCloseTo(0, 10);
    expect(dot(c, b)).toBeCloseTo(0, 10);
  });

  it('length of a 3-4-12 vector is 13 and normalising gives unit length', () => {
    expect(length([3, 4, 12])).toBeCloseTo(13, 12);
    expect(length(normalize([3, 4, 12]))).toBeCloseTo(1, 12);
  });

  it('normalising the zero vector returns zero instead of NaN', () => {
    expect(normalize([0, 0, 0])).toEqual([0, 0, 0]);
  });

  it('distance between two points matches Pythagoras', () => {
    expect(distance3([1, 1, 1], [4, 5, 1])).toBeCloseTo(5, 12);
  });
});

describe('matrix maths', () => {
  it('multiplying by the identity leaves a matrix unchanged', () => {
    const m = perspective(1, 1.5, 0.1, 50);
    expectVec(multiply(m, IDENTITY), Array.from(m), 6);
    expectVec(multiply(IDENTITY, m), Array.from(m), 6);
  });

  it('a matrix times its inverse is the identity', () => {
    const m = multiply(perspective(0.7, 1.6, 1, 100), lookAt([3, 4, 5], [0, 1, 0], [0, 1, 0]));
    const inv = invert(m);
    expect(inv).not.toBeNull();
    expectVec(multiply(m, inv as Mat4), Array.from(IDENTITY), 3);
  });

  it('a singular matrix has no inverse', () => {
    expect(invert(new Float32Array(16))).toBeNull();
  });

  it('look-at moves the eye to the origin and the target straight down negative z', () => {
    const eye: Vec3 = [3, 4, 5];
    const target: Vec3 = [1, -2, 0.5];
    const v = lookAt(eye, target, [0, 1, 0]);
    expectVec(transformPoint(v, eye), [0, 0, 0, 1]);
    expectVec(transformPoint(v, target), [0, 0, -distance3(eye, target), 1]);
  });

  it('look-at keeps distances between points (rigid transform)', () => {
    const v = lookAt([2, 1, 7], [0, 0, 0], [0, 1, 0]);
    const a = transformPoint(v, [1, 2, 3]);
    const b = transformPoint(v, [-2, 0, 5]);
    expect(Math.hypot(a[0] - b[0], a[1] - b[1], a[2] - b[2])).toBeCloseTo(distance3([1, 2, 3], [-2, 0, 5]), 4);
  });

  it('perspective maps the near plane to depth -1 and the far plane to depth +1', () => {
    const p = perspective(Math.PI / 2, 1, 1, 100);
    expect(ndc(p, [0, 0, -1])[2]).toBeCloseTo(-1, 4);
    expect(ndc(p, [0, 0, -100])[2]).toBeCloseTo(1, 4);
  });

  it('perspective with a 90 degree field of view puts a 45 degree point on the frame edge', () => {
    const p = perspective(Math.PI / 2, 2, 1, 100);
    const n = ndc(p, [0, 10, -10]);
    expect(n[1]).toBeCloseTo(1, 5);
    expect(ndc(p, [20, 0, -10])[0]).toBeCloseTo(1, 5);
  });

  it('orthographic projection maps the box corners to the unit cube corners', () => {
    const o = ortho(2, 2, 1, 9);
    expectVec(ndc(o, [4, 2, -1]), [1, 1, -1], 5);
    expectVec(ndc(o, [-4, -2, -9]), [-1, -1, 1], 5);
  });
});

describe('orbit camera', () => {
  it('front view places the eye on positive z at the camera distance', () => {
    const cam = front(5);
    expectVec(cam.eye(), [0, 0, 5], 10);
  });

  it('right view places the eye on positive x and left view on negative x', () => {
    const cam = front(3);
    cam.snap('right');
    expectVec(cam.eye(), [3, 0, 0], 10);
    cam.snap('left');
    expectVec(cam.eye(), [-3, 0, 0], 10);
  });

  it('top view looks straight down from above the target', () => {
    const cam = front(4);
    cam.target = [1, 2, 3];
    cam.snap('top');
    const e = cam.eye();
    expect(e[1]).toBeCloseTo(6, 4);
    expect(e[0]).toBeCloseTo(1, 2);
    expect(e[2]).toBeCloseTo(3, 2);
  });

  it('orbiting keeps the eye at the same distance from the target', () => {
    const cam = front(5);
    cam.target = [1, 2, 3];
    cam.orbit(137, -52);
    expect(distance3(cam.eye(), cam.target)).toBeCloseTo(5, 10);
  });

  it('a horizontal orbit drag does not change the eye height', () => {
    const cam = new OrbitCamera();
    const before = cam.eye()[1];
    cam.orbit(250, 0);
    expect(cam.eye()[1]).toBeCloseTo(before, 12);
  });

  it('orbiting there and back returns the eye to where it started', () => {
    const cam = new OrbitCamera();
    const before = cam.eye();
    cam.orbit(80, 30);
    cam.orbit(-80, -30);
    expectVec(cam.eye(), before, 10);
  });

  it('orbiting far past the pole stops just short of straight overhead with a finite view', () => {
    const cam = front(5);
    cam.orbit(0, 1e6);
    const e = cam.eye();
    expect(e[1]).toBeGreaterThan(4.99);
    expect(e[1]).toBeLessThan(5);
    expect(Array.from(cam.view()).every(Number.isFinite)).toBe(true);
  });

  it('trackball orbit keeps distance and keeps the up vector perpendicular to the view direction', () => {
    const cam = front(5);
    cam.setTrackball(true);
    cam.orbit(300, 900);
    cam.orbit(-120, 400);
    expect(distance3(cam.eye(), cam.target)).toBeCloseTo(5, 9);
    const fwd = normalize(sub(cam.target, cam.eye()));
    expect(dot(cam.screenUp(), fwd)).toBeCloseTo(0, 9);
    expect(length(cam.screenUp())).toBeCloseTo(1, 9);
  });

  it('switching trackball on and off leaves the eye where it was', () => {
    const cam = new OrbitCamera();
    const before = cam.eye();
    cam.setTrackball(true);
    expectVec(cam.eye(), before, 10);
    cam.setTrackball(false);
    expectVec(cam.eye(), before, 10);
  });

  it('panning up by the full viewport height moves the target by the visible height', () => {
    const cam = front(5);
    const visible = 2 * 5 * Math.tan(cam.fov / 2);
    cam.pan(0, 600, 600);
    expectVec(cam.target, [0, visible, 0], 10);
    expectVec(cam.eye(), [0, visible, 5], 10);
  });

  it('dragging right by the full viewport height moves the target left by the visible height', () => {
    const cam = front(5);
    const visible = 2 * 5 * Math.tan(cam.fov / 2);
    cam.pan(600, 0, 600);
    expectVec(cam.target, [-visible, 0, 0], 10);
  });

  it('panning never changes the camera distance or view direction', () => {
    const cam = new OrbitCamera();
    const dir = sub(cam.eye(), cam.target);
    cam.pan(33, -71, 480);
    expectVec(sub(cam.eye(), cam.target), dir, 10);
  });

  it('world size of one pixel times the viewport height is the visible height', () => {
    const cam = front(8);
    expect(cam.worldPerPixel(400) * 400).toBeCloseTo(2 * 8 * Math.tan(cam.fov / 2), 10);
  });

  it('zoom multiplies the distance and zooming in then out by the same factor restores it', () => {
    const cam = front(4);
    cam.zoom(0.5);
    expect(cam.distance).toBeCloseTo(2, 12);
    cam.zoom(2);
    expect(cam.distance).toBeCloseTo(4, 12);
  });

  it('zoom is clamped so the distance stays positive and bounded', () => {
    const cam = front(4);
    cam.zoom(0);
    expect(cam.distance).toBeGreaterThan(0);
    cam.zoom(1e30);
    expect(cam.distance).toBe(1e5);
  });

  it('framing a box centres the target and fits the bounding sphere inside the view cone with a margin', () => {
    const cam = new OrbitCamera();
    cam.frame([-1, 0, 2], [3, 4, 6]);
    expectVec(cam.target, [1, 2, 4], 12);
    const radius = 0.5 * Math.hypot(4, 4, 4);
    expect(cam.distance * Math.sin(cam.fov / 2)).toBeCloseTo(radius * 1.1, 9);
    expect(cam.distance * Math.sin(cam.fov / 2)).toBeGreaterThan(radius);
  });

  it('framing a single point still yields a usable positive distance', () => {
    const cam = new OrbitCamera();
    cam.frame([2, 2, 2], [2, 2, 2]);
    expect(cam.distance).toBeGreaterThan(0);
    expect(Number.isFinite(cam.distance)).toBe(true);
  });

  it('moving the pivot keeps the eye fixed and orbits around the new point', () => {
    const cam = new OrbitCamera();
    const eye = cam.eye();
    cam.pivotAt([0.5, -0.25, 1]);
    expectVec(cam.eye(), eye, 9);
    expect(cam.target).toEqual([0.5, -0.25, 1]);
    expect(cam.distance).toBeCloseTo(distance3(eye, [0.5, -0.25, 1]), 9);
  });

  it('looking around keeps the eye fixed', () => {
    const cam = new OrbitCamera();
    const eye = cam.eye();
    cam.look(120, -40);
    expectVec(cam.eye(), eye, 9);
  });

  it('walking forward moves eye and target along the view direction by the given amount', () => {
    const cam = front(5);
    cam.walk(2, 0, 0);
    expectVec(cam.target, [0, 0, -2], 10);
    expectVec(cam.eye(), [0, 0, 3], 10);
  });

  it('pose reports angles in degrees for the right view', () => {
    const cam = front(5);
    cam.snap('right');
    const p = cam.pose();
    expect(p.yaw).toBeCloseTo(90, 9);
    expect(p.pitch).toBeCloseTo(0, 9);
    expect(p.distance).toBe(5);
    expect(p.ortho).toBe(false);
  });

  it('a saved pose restores the same eye position on another camera', () => {
    const a = new OrbitCamera();
    a.target = [1, 2, 3];
    a.orbit(77, 21);
    a.zoom(1.7);
    a.orthographic = true;
    const b = new OrbitCamera();
    b.setPose(a.pose());
    expectVec(b.eye(), a.eye(), 9);
    expect(b.orthographic).toBe(true);
  });

  it('home restores the default viewing angles after orbiting', () => {
    const cam = new OrbitCamera();
    const start = cam.eye();
    cam.orbit(300, 100);
    cam.home();
    expectVec(cam.eye(), start, 10);
  });

  it('the target projects to the centre of the screen', () => {
    const cam = new OrbitCamera();
    cam.target = [1, 2, 3];
    cam.aspect = 1.5;
    const n = ndc(cam.viewProjection(), cam.target);
    expect(n[0]).toBeCloseTo(0, 4);
    expect(n[1]).toBeCloseTo(0, 4);
  });

  it('the picking ray through the screen centre points from the eye at the target', () => {
    const cam = new OrbitCamera();
    cam.target = [1, 2, 3];
    cam.aspect = 2;
    const r = cam.ray(400, 200, 800, 400);
    expect(r).not.toBeNull();
    if (!r) return;
    expectVec(r.dir, normalize(sub(cam.target, cam.eye())), 3);
  });

  it('projecting a world point and casting a ray through that pixel passes back through the point', () => {
    const cam = new OrbitCamera();
    cam.aspect = 800 / 600;
    const p: Vec3 = [0.6, -0.4, 0.3];
    const n = ndc(cam.viewProjection(), p);
    const x = (n[0] * 0.5 + 0.5) * 800;
    const y = (0.5 - n[1] * 0.5) * 600;
    const r = cam.ray(x, y, 800, 600);
    expect(r).not.toBeNull();
    if (!r) return;
    const off = length(cross(sub(p, r.origin), r.dir));
    expect(off).toBeLessThan(5e-3);
  });

  it('perspective rays all start near the eye while orthographic rays are parallel', () => {
    const cam = front(5);
    const a = cam.ray(10, 10, 400, 400);
    const b = cam.ray(390, 300, 400, 400);
    if (!a || !b) throw new Error('no ray');
    expect(distance3(a.origin, cam.eye())).toBeLessThan(0.05);
    expect(dot(a.dir, b.dir)).toBeLessThan(0.99);
    cam.orthographic = true;
    const c = cam.ray(10, 10, 400, 400);
    const d = cam.ray(390, 300, 400, 400);
    if (!c || !d) throw new Error('no ray');
    expectVec(c.dir, [0, 0, -1], 3);
    expectVec(d.dir, [0, 0, -1], 3);
  });

  it('an orthographic ray at the top edge starts half the visible height above the target', () => {
    const cam = front(5);
    cam.orthographic = true;
    const r = cam.ray(200, 0, 400, 400);
    if (!r) throw new Error('no ray');
    expect(r.origin[1]).toBeCloseTo(5 * Math.tan(cam.fov / 2), 3);
    expect(r.origin[0]).toBeCloseTo(0, 3);
  });
});
