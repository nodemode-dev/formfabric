import type { Frame } from './frame';

const VS = `#version 300 es
layout(location = 0) in vec3 aPos;
uniform mat4 uVP;
void main() {
  vec4 p = uVP * vec4(aPos, 1.0);
  p.z -= 0.0014 * p.w;
  gl_Position = p;
}`;

const FS = `#version 300 es
precision highp float;
uniform vec4 uColor;
out vec4 outColor;
void main() {
  outColor = uColor;
}`;

function compile(gl: WebGL2RenderingContext, type: number, src: string): WebGLShader {
  const sh = gl.createShader(type) as WebGLShader;
  gl.shaderSource(sh, src);
  gl.compileShader(sh);
  if (!gl.getShaderParameter(sh, gl.COMPILE_STATUS)) throw new Error(gl.getShaderInfoLog(sh) ?? 'shader');
  return sh;
}

export interface StylePoly {
  id: number;
  points: number[];
  closed: boolean;
}

export function parseStyleLines(reply: unknown): StylePoly[] {
  const o = reply as Record<string, unknown> | unknown[] | null;
  const arr = Array.isArray(o) ? o : Array.isArray((o as Record<string, unknown> | null)?.lines) ? ((o as Record<string, unknown>).lines as unknown[]) : null;
  if (!arr) return [];
  const out: StylePoly[] = [];
  for (const x of arr) {
    const e = x as Record<string, unknown>;
    const raw = (e.points3d ?? e.positions ?? e.points) as unknown;
    if (!Array.isArray(raw) || raw.length === 0) continue;
    const flat = Array.isArray(raw[0]) ? (raw as number[][]).flatMap((p) => [p[0], p[1], p[2] ?? 0]) : (raw as number[]);
    if (flat.length < 6 || flat.length % 3 !== 0 || flat.some((v) => typeof v !== 'number' || !Number.isFinite(v))) continue;
    out.push({ id: typeof e.id === 'number' ? e.id : out.length, points: flat, closed: e.closed === true });
  }
  return out;
}

export class StyleGL {
  private readonly prog: WebGLProgram;
  private readonly vao: WebGLVertexArrayObject | null;
  private readonly buf: WebGLBuffer | null;
  private readonly uVP: WebGLUniformLocation | null;
  private readonly uColor: WebGLUniformLocation | null;
  private scratch = new Float32Array(1024);
  private bytes = 0;
  count = 0;

  constructor(private readonly gl: WebGL2RenderingContext) {
    const p = gl.createProgram() as WebGLProgram;
    gl.attachShader(p, compile(gl, gl.VERTEX_SHADER, VS));
    gl.attachShader(p, compile(gl, gl.FRAGMENT_SHADER, FS));
    gl.linkProgram(p);
    if (!gl.getProgramParameter(p, gl.LINK_STATUS)) throw new Error(gl.getProgramInfoLog(p) ?? 'link');
    this.prog = p;
    this.uVP = gl.getUniformLocation(p, 'uVP');
    this.uColor = gl.getUniformLocation(p, 'uColor');
    this.vao = gl.createVertexArray();
    this.buf = gl.createBuffer();
    gl.bindVertexArray(this.vao);
    gl.bindBuffer(gl.ARRAY_BUFFER, this.buf);
    gl.enableVertexAttribArray(0);
    gl.vertexAttribPointer(0, 3, gl.FLOAT, false, 0, 0);
    gl.bindVertexArray(null);
    gl.bindBuffer(gl.ARRAY_BUFFER, null);
  }

  set(lines: StylePoly[]): void {
    let need = 0;
    for (const l of lines) need += (l.points.length / 3 - 1 + (l.closed ? 1 : 0)) * 6;
    if (this.scratch.length < need) this.scratch = new Float32Array(need * 2);
    const s = this.scratch;
    let n = 0;
    for (const l of lines) {
      const p = l.points;
      const m = p.length / 3;
      const last = l.closed ? m : m - 1;
      for (let i = 0; i < last; i++) {
        const a = i * 3;
        const b = ((i + 1) % m) * 3;
        s[n++] = p[a]; s[n++] = p[a + 1]; s[n++] = p[a + 2];
        s[n++] = p[b]; s[n++] = p[b + 1]; s[n++] = p[b + 2];
      }
    }
    this.count = n / 3;
    if (n === 0) return;
    const gl = this.gl;
    gl.bindBuffer(gl.ARRAY_BUFFER, this.buf);
    if (n * 4 > this.bytes) {
      this.bytes = Math.max(n * 4, this.bytes * 2);
      gl.bufferData(gl.ARRAY_BUFFER, this.bytes, gl.DYNAMIC_DRAW);
    }
    gl.bufferSubData(gl.ARRAY_BUFFER, 0, s, 0, n);
    gl.bindBuffer(gl.ARRAY_BUFFER, null);
  }

  draw(f: Frame, rgb: Float32Array): void {
    if (this.count === 0) return;
    const gl = this.gl;
    gl.useProgram(this.prog);
    gl.uniformMatrix4fv(this.uVP, false, f.vp);
    gl.uniform4f(this.uColor, rgb[0], rgb[1], rgb[2], 1);
    gl.enable(gl.DEPTH_TEST);
    gl.depthFunc(gl.LEQUAL);
    gl.depthMask(false);
    gl.bindVertexArray(this.vao);
    gl.drawArrays(gl.LINES, 0, this.count);
    gl.bindVertexArray(null);
    gl.depthMask(true);
    gl.disable(gl.DEPTH_TEST);
  }
}
