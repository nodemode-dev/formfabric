import type { PanelFull, Quat, V3 } from './model';

const RAD = Math.PI / 180;

export function quatMul(a: Quat, b: Quat): Quat {
  return [
    a[3] * b[0] + a[0] * b[3] + a[1] * b[2] - a[2] * b[1],
    a[3] * b[1] - a[0] * b[2] + a[1] * b[3] + a[2] * b[0],
    a[3] * b[2] + a[0] * b[1] - a[1] * b[0] + a[2] * b[3],
    a[3] * b[3] - a[0] * b[0] - a[1] * b[1] - a[2] * b[2],
  ];
}

export function quatAxisAngle(axis: number, radians: number): Quat {
  const s = Math.sin(radians / 2);
  const q: Quat = [0, 0, 0, Math.cos(radians / 2)];
  q[axis] = s;
  return q;
}

export function quatNormalize(q: Quat): Quat {
  const l = Math.hypot(q[0], q[1], q[2], q[3]) || 1;
  return [q[0] / l, q[1] / l, q[2] / l, q[3] / l];
}

export function quatRotate(q: Quat, v: V3): V3 {
  const [qx, qy, qz, qw] = q;
  const tx = 2 * (qy * v[2] - qz * v[1]);
  const ty = 2 * (qz * v[0] - qx * v[2]);
  const tz = 2 * (qx * v[1] - qy * v[0]);
  return [v[0] + qw * tx + (qy * tz - qz * ty), v[1] + qw * ty + (qz * tx - qx * tz), v[2] + qw * tz + (qx * ty - qy * tx)];
}

export function placementOf(panel: PanelFull): { origin: V3; rot: Quat } {
  const p = panel.place3d;
  if (p && p.origin && p.rotation) return { origin: [p.origin[0], p.origin[1], p.origin[2]], rot: quatNormalize([p.rotation[0], p.rotation[1], p.rotation[2], p.rotation[3]]) };
  return { origin: [panel.position[0], panel.position[1], 0], rot: quatAxisAngle(2, panel.rotation * RAD) };
}

export function placeLocal(panel: PanelFull, local: Float32Array, out: Float32Array): void {
  const { origin, rot } = placementOf(panel);
  const [qx, qy, qz, qw] = rot;
  const flip = panel.mirror ? -1 : 1;
  const n = local.length >> 1;
  for (let i = 0; i < n; i++) {
    const x = local[i * 2] * flip;
    const y = local[i * 2 + 1];
    const tx = 2 * (qy * 0 - qz * y);
    const ty = 2 * (qz * x - qx * 0);
    const tz = 2 * (qx * y - qy * x);
    out[i * 3] = origin[0] + x + qw * tx + (qy * tz - qz * ty);
    out[i * 3 + 1] = origin[1] + y + qw * ty + (qz * tx - qx * tz);
    out[i * 3 + 2] = origin[2] + qw * tz + (qx * ty - qy * tx);
  }
}
