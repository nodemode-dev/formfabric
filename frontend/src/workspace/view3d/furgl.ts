import type { FurJson } from '../viz/types';
import type { Frame } from './frame';
import type { Scene } from './scene';

const VS = `#version 300 es
layout(location = 0) in vec4 aPos;
uniform mat4 uVP;
out float vT;
void main() {
  vT = aPos.w;
  vec4 p = uVP * vec4(aPos.xyz, 1.0);
  p.z -= 0.0006 * p.w;
  gl_Position = p;
}`;

const FS = `#version 300 es
precision highp float;
in float vT;
uniform vec3 uColor;
uniform float uAlpha;
out vec4 outColor;
void main() {
  vec3 c = uColor * (0.45 + 0.55 * vT);
  outColor = vec4(c, uAlpha);
}`;

const SEGMENTS = 4;
const PER_ENTRY = 24000;
const TOTAL = 60000;
const REBUILD_MS = 90;

interface Strand {
  tri: number;
  u: number;
  v: number;
  seed: number;
}

interface Entry {
  key: string;
  idxLen: number;
  strands: Strand[];
}

interface Run {
  color: [number, number, number];
  alpha: number;
  first: number;
  count: number;
}

function rng(seed: number): () => number {
  let s = seed >>> 0 || 1;
  return () => {
    s ^= s << 13;
    s >>>= 0;
    s ^= s >>> 17;
    s ^= s << 5;
    s >>>= 0;
    return s / 4294967296;
  };
}

function compile(gl: WebGL2RenderingContext, type: number, src: string): WebGLShader {
  const sh = gl.createShader(type) as WebGLShader;
  gl.shaderSource(sh, src);
  gl.compileShader(sh);
  if (!gl.getShaderParameter(sh, gl.COMPILE_STATUS)) throw new Error(gl.getShaderInfoLog(sh) ?? 'shader');
  return sh;
}

export class FurGL {
  private readonly prog: WebGLProgram;
  private readonly vao: WebGLVertexArrayObject | null;
  private readonly buf: WebGLBuffer | null;
  private readonly uVP: WebGLUniformLocation | null;
  private readonly uColor: WebGLUniformLocation | null;
  private readonly uAlpha: WebGLUniformLocation | null;
  private readonly entries = new Map<number, Entry>();
  private runs: Run[] = [];
  private scratch = new Float32Array(4096);
  private bytes = 0;
  private fur: FurJson[] = [];
  private version = -1;
  private built = 0;
  private timer = 0;
  private dirty = true;

  constructor(private readonly gl: WebGL2RenderingContext, private readonly redraw: () => void) {
    const p = gl.createProgram() as WebGLProgram;
    gl.attachShader(p, compile(gl, gl.VERTEX_SHADER, VS));
    gl.attachShader(p, compile(gl, gl.FRAGMENT_SHADER, FS));
    gl.linkProgram(p);
    if (!gl.getProgramParameter(p, gl.LINK_STATUS)) throw new Error(gl.getProgramInfoLog(p) ?? 'link');
    this.prog = p;
    this.uVP = gl.getUniformLocation(p, 'uVP');
    this.uColor = gl.getUniformLocation(p, 'uColor');
    this.uAlpha = gl.getUniformLocation(p, 'uAlpha');
    this.vao = gl.createVertexArray();
    this.buf = gl.createBuffer();
    gl.bindVertexArray(this.vao);
    gl.bindBuffer(gl.ARRAY_BUFFER, this.buf);
    gl.enableVertexAttribArray(0);
    gl.vertexAttribPointer(0, 4, gl.FLOAT, false, 0, 0);
    gl.bindVertexArray(null);
    gl.bindBuffer(gl.ARRAY_BUFFER, null);
  }

  get active(): boolean {
    return this.fur.length > 0;
  }

  setFur(fur: FurJson[]): void {
    if (fur === this.fur) return;
    this.fur = fur;
    this.dirty = true;
    if (fur.length === 0) this.runs = [];
  }

  private seeds(f: FurJson, idx: Uint32Array, pos: Float32Array): Strand[] {
    const tris = idx.length / 3;
    const cum = new Float64Array(tris);
    let total = 0;
    for (let t = 0; t < tris; t++) {
      const a = idx[t * 3] * 3;
      const b = idx[t * 3 + 1] * 3;
      const c = idx[t * 3 + 2] * 3;
      const e1x = pos[b] - pos[a], e1y = pos[b + 1] - pos[a + 1], e1z = pos[b + 2] - pos[a + 2];
      const e2x = pos[c] - pos[a], e2y = pos[c + 1] - pos[a + 1], e2z = pos[c + 2] - pos[a + 2];
      total += 0.5 * Math.hypot(e1y * e2z - e1z * e2y, e1z * e2x - e1x * e2z, e1x * e2y - e1y * e2x);
      cum[t] = total;
    }
    const want = Math.min(PER_ENTRY, Math.max(0, Math.round((total / 100) * Math.max(0, f.density))));
    const out: Strand[] = [];
    const rand = rng(f.id * 7919 + f.panel * 104729 + 17);
    for (let i = 0; i < want; i++) {
      const r = rand() * total;
      let lo = 0;
      let hi = tris - 1;
      while (lo < hi) {
        const mid = (lo + hi) >> 1;
        if (cum[mid] < r) lo = mid + 1; else hi = mid;
      }
      let u = rand();
      let v = rand();
      if (u + v > 1) { u = 1 - u; v = 1 - v; }
      out.push({ tri: lo, u, v, seed: Math.floor(rand() * 1e9) });
    }
    return out;
  }

  sync(scene: Scene): void {
    if (this.fur.length === 0) return;
    if (!this.dirty && scene.version === this.version) return;
    const now = performance.now();
    if (now - this.built < REBUILD_MS) {
      if (!this.timer) this.timer = window.setTimeout(() => { this.timer = 0; this.redraw(); }, REBUILD_MS);
      return;
    }
    this.built = now;
    this.version = scene.version;
    this.dirty = false;
    const gl = this.gl;
    const runs: Run[] = [];
    let at = 0;
    let budget = TOTAL;
    for (const f of this.fur) {
      const b = scene.get(f.panel);
      if (!b || !b.ready || b.kind !== 'panel' || budget <= 0) continue;
      const key = `${f.id}|${f.panel}|${f.density}`;
      let e = this.entries.get(f.id);
      if (!e || e.key !== key || e.idxLen !== b.idx.length) {
        e = { key, idxLen: b.idx.length, strands: this.seeds(f, b.idx, b.pos) };
        this.entries.set(f.id, e);
      }
      const n = Math.min(e.strands.length, budget);
      budget -= n;
      const need = at + n * SEGMENTS * 8;
      if (this.scratch.length < need) {
        const g = new Float32Array(need * 2);
        g.set(this.scratch.subarray(0, at));
        this.scratch = g;
      }
      const s = this.scratch;
      const pos = b.pos;
      const nrm = b.nrm;
      const idx = b.idx;
      const len = Math.max(0.1, f.length);
      const curl = Math.max(0, f.curl);
      const first = at / 4;
      for (let i = 0; i < n; i++) {
        const st = e.strands[i];
        const ia = idx[st.tri * 3] * 3;
        const ib = idx[st.tri * 3 + 1] * 3;
        const ic = idx[st.tri * 3 + 2] * 3;
        const w = 1 - st.u - st.v;
        const rx = w * pos[ia] + st.u * pos[ib] + st.v * pos[ic];
        const ry = w * pos[ia + 1] + st.u * pos[ib + 1] + st.v * pos[ic + 1];
        const rz = w * pos[ia + 2] + st.u * pos[ib + 2] + st.v * pos[ic + 2];
        let nx = w * nrm[ia] + st.u * nrm[ib] + st.v * nrm[ic];
        let ny = w * nrm[ia + 1] + st.u * nrm[ib + 1] + st.v * nrm[ic + 1];
        let nz = w * nrm[ia + 2] + st.u * nrm[ib + 2] + st.v * nrm[ic + 2];
        const nl = Math.hypot(nx, ny, nz) || 1;
        nx /= nl; ny /= nl; nz /= nl;
        let tx = ny * 0 - nz * 1;
        let ty = nz * 0 - nx * 0;
        let tz = nx * 1 - ny * 0;
        if (Math.hypot(tx, ty, tz) < 1e-3) { tx = 1; ty = 0; tz = 0; }
        const tl = Math.hypot(tx, ty, tz) || 1;
        tx /= tl; ty /= tl; tz /= tl;
        const bx = ny * tz - nz * ty;
        const by = nz * tx - nx * tz;
        const bz = nx * ty - ny * tx;
        const phase = (st.seed % 6283) / 1000;
        const jitter = 0.75 + ((st.seed >> 3) % 100) / 400;
        let px = rx, py = ry, pz = rz;
        for (let k = 1; k <= SEGMENTS; k++) {
          const sk = k / SEGMENTS;
          const a = phase + sk * 5.0;
          const lc = curl * len * 0.25 * sk;
          const qx = rx + nx * len * jitter * sk + (tx * Math.cos(a) + bx * Math.sin(a)) * lc;
          const qy = ry + ny * len * jitter * sk + (ty * Math.cos(a) + by * Math.sin(a)) * lc - len * 0.18 * sk * sk;
          const qz = rz + nz * len * jitter * sk + (tz * Math.cos(a) + bz * Math.sin(a)) * lc;
          s[at++] = px; s[at++] = py; s[at++] = pz; s[at++] = (k - 1) / SEGMENTS;
          s[at++] = qx; s[at++] = qy; s[at++] = qz; s[at++] = sk;
          px = qx; py = qy; pz = qz;
        }
      }
      runs.push({ color: [f.color[0], f.color[1], f.color[2]], alpha: Math.max(0.3, Math.min(1, 0.45 + f.thickness * 0.4)), first, count: at / 4 - first });
    }
    this.runs = runs;
    if (at > 0) {
      gl.bindBuffer(gl.ARRAY_BUFFER, this.buf);
      if (at * 4 > this.bytes) {
        this.bytes = Math.max(at * 4, this.bytes * 2);
        gl.bufferData(gl.ARRAY_BUFFER, this.bytes, gl.DYNAMIC_DRAW);
      }
      gl.bufferSubData(gl.ARRAY_BUFFER, 0, this.scratch, 0, at);
      gl.bindBuffer(gl.ARRAY_BUFFER, null);
    }
  }

  draw(f: Frame): void {
    if (this.runs.length === 0) return;
    const gl = this.gl;
    gl.useProgram(this.prog);
    gl.uniformMatrix4fv(this.uVP, false, f.vp);
    gl.enable(gl.DEPTH_TEST);
    gl.depthFunc(gl.LEQUAL);
    gl.depthMask(false);
    gl.enable(gl.BLEND);
    gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);
    gl.bindVertexArray(this.vao);
    for (const r of this.runs) {
      gl.uniform3f(this.uColor, r.color[0], r.color[1], r.color[2]);
      gl.uniform1f(this.uAlpha, r.alpha);
      gl.drawArrays(gl.LINES, r.first, r.count);
    }
    gl.bindVertexArray(null);
    gl.depthMask(true);
    gl.disable(gl.BLEND);
    gl.disable(gl.DEPTH_TEST);
  }
}
