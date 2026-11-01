import type { OrbitCamera } from './camera';

export class Frame {
  readonly view = new Float32Array(16);
  readonly proj = new Float32Array(16);
  readonly vp = new Float32Array(16);
  readonly inv = new Float32Array(16);
  readonly eye = new Float32Array(3);
  readonly fwd = new Float32Array(3);
  width = 1;
  height = 1;
  ortho = false;
  private invValid = false;
  private readonly tmp = new Float32Array(16);

  update(cam: OrbitCamera, width: number, height: number): void {
    this.width = width;
    this.height = height;
    this.ortho = cam.orthographic;
    const cp = Math.cos(cam.pitch);
    const dx = cp * Math.sin(cam.yaw);
    const dy = Math.sin(cam.pitch);
    const dz = cp * Math.cos(cam.yaw);
    const e = this.eye;
    e[0] = cam.target[0] + dx * cam.distance;
    e[1] = cam.target[1] + dy * cam.distance;
    e[2] = cam.target[2] + dz * cam.distance;
    this.fwd[0] = -dx;
    this.fwd[1] = -dy;
    this.fwd[2] = -dz;
    let xx = dz;
    let xz = -dx;
    const xl = Math.hypot(xx, xz) || 1;
    xx /= xl;
    xz /= xl;
    const yx = dy * xz;
    const yy = dz * xx - dx * xz;
    const yz = -dy * xx;
    const v = this.view;
    v[0] = xx; v[1] = yx; v[2] = dx; v[3] = 0;
    v[4] = 0; v[5] = yy; v[6] = dy; v[7] = 0;
    v[8] = xz; v[9] = yz; v[10] = dz; v[11] = 0;
    v[12] = -(xx * e[0] + xz * e[2]);
    v[13] = -(yx * e[0] + yy * e[1] + yz * e[2]);
    v[14] = -(dx * e[0] + dy * e[1] + dz * e[2]);
    v[15] = 1;
    const aspect = width / Math.max(height, 1);
    const near = Math.max(cam.distance * 0.001, 1e-4);
    const far = cam.distance * 100;
    const p = this.proj;
    p.fill(0);
    if (this.ortho) {
      const hh = cam.distance * Math.tan(cam.fov / 2);
      const nf = 1 / (-far - far);
      p[0] = 1 / (hh * aspect);
      p[5] = 1 / hh;
      p[10] = 2 * nf;
      p[14] = 0;
      p[15] = 1;
    } else {
      const f = 1 / Math.tan(cam.fov / 2);
      const nf = 1 / (near - far);
      p[0] = f / aspect;
      p[5] = f;
      p[10] = (far + near) * nf;
      p[11] = -1;
      p[14] = 2 * far * near * nf;
    }
    mulInto(this.vp, p, v);
    this.invValid = false;
  }

  project(x: number, y: number, z: number, out: Float64Array): boolean {
    const m = this.vp;
    const cw = m[3] * x + m[7] * y + m[11] * z + m[15];
    if (cw <= 1e-6) return false;
    const cx = m[0] * x + m[4] * y + m[8] * z + m[12];
    const cy = m[1] * x + m[5] * y + m[9] * z + m[13];
    out[0] = (cx / cw * 0.5 + 0.5) * this.width;
    out[1] = (0.5 - cy / cw * 0.5) * this.height;
    out[2] = cw;
    return true;
  }

  ray(px: number, py: number, origin: Float64Array, dir: Float64Array): boolean {
    if (!this.invValid) {
      if (!invertInto(this.inv, this.vp, this.tmp)) return false;
      this.invValid = true;
    }
    const nx = (2 * px) / this.width - 1;
    const ny = 1 - (2 * py) / this.height;
    const m = this.inv;
    unprojectInto(m, nx, ny, -1, origin);
    unprojectInto(m, nx, ny, 1, dir);
    const ox = origin[0];
    const oy = origin[1];
    const oz = origin[2];
    let dx = dir[0] - ox;
    let dy = dir[1] - oy;
    let dz = dir[2] - oz;
    const l = Math.hypot(dx, dy, dz) || 1;
    dx /= l;
    dy /= l;
    dz /= l;
    dir[0] = dx;
    dir[1] = dy;
    dir[2] = dz;
    return true;
  }
}

function unprojectInto(m: Float32Array, x: number, y: number, z: number, out: Float64Array): void {
  const w = m[3] * x + m[7] * y + m[11] * z + m[15];
  const iw = 1 / (w || 1);
  out[0] = (m[0] * x + m[4] * y + m[8] * z + m[12]) * iw;
  out[1] = (m[1] * x + m[5] * y + m[9] * z + m[13]) * iw;
  out[2] = (m[2] * x + m[6] * y + m[10] * z + m[14]) * iw;
}

export function mulInto(out: Float32Array, a: Float32Array, b: Float32Array): void {
  for (let c = 0; c < 4; c++) {
    const b0 = b[c * 4];
    const b1 = b[c * 4 + 1];
    const b2 = b[c * 4 + 2];
    const b3 = b[c * 4 + 3];
    for (let r = 0; r < 4; r++) out[c * 4 + r] = a[r] * b0 + a[4 + r] * b1 + a[8 + r] * b2 + a[12 + r] * b3;
  }
}

export function invertInto(out: Float32Array, a: Float32Array, scratch: Float32Array): boolean {
  const b00 = a[0] * a[5] - a[1] * a[4], b01 = a[0] * a[6] - a[2] * a[4], b02 = a[0] * a[7] - a[3] * a[4];
  const b03 = a[1] * a[6] - a[2] * a[5], b04 = a[1] * a[7] - a[3] * a[5], b05 = a[2] * a[7] - a[3] * a[6];
  const b06 = a[8] * a[13] - a[9] * a[12], b07 = a[8] * a[14] - a[10] * a[12], b08 = a[8] * a[15] - a[11] * a[12];
  const b09 = a[9] * a[14] - a[10] * a[13], b10 = a[9] * a[15] - a[11] * a[13], b11 = a[10] * a[15] - a[11] * a[14];
  let det = b00 * b11 - b01 * b10 + b02 * b09 + b03 * b08 - b04 * b07 + b05 * b06;
  if (!det) return false;
  det = 1 / det;
  const o = scratch;
  o[0] = (a[5] * b11 - a[6] * b10 + a[7] * b09) * det;
  o[1] = (a[2] * b10 - a[1] * b11 - a[3] * b09) * det;
  o[2] = (a[13] * b05 - a[14] * b04 + a[15] * b03) * det;
  o[3] = (a[10] * b04 - a[9] * b05 - a[11] * b03) * det;
  o[4] = (a[6] * b08 - a[4] * b11 - a[7] * b07) * det;
  o[5] = (a[0] * b11 - a[2] * b08 + a[3] * b07) * det;
  o[6] = (a[14] * b02 - a[12] * b05 - a[15] * b01) * det;
  o[7] = (a[8] * b05 - a[10] * b02 + a[11] * b01) * det;
  o[8] = (a[4] * b10 - a[5] * b08 + a[7] * b06) * det;
  o[9] = (a[1] * b08 - a[0] * b10 - a[3] * b06) * det;
  o[10] = (a[12] * b04 - a[13] * b02 + a[15] * b00) * det;
  o[11] = (a[9] * b02 - a[8] * b04 - a[11] * b00) * det;
  o[12] = (a[5] * b07 - a[4] * b09 - a[6] * b06) * det;
  o[13] = (a[0] * b09 - a[1] * b07 + a[2] * b06) * det;
  o[14] = (a[13] * b01 - a[12] * b03 - a[14] * b00) * det;
  o[15] = (a[8] * b03 - a[9] * b01 + a[10] * b00) * det;
  out.set(o);
  return true;
}
