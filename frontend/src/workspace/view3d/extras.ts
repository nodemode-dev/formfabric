import type { PinMark, SeamLine } from '../drape/types';
import type { Frame } from './frame';
import type { Scene } from './scene';
import type { Garment } from '../types';
import { trimBuffers } from '../trims/hook3d';
import type { DrawPalette } from './renderer';

const PVS = `#version 300 es
layout(location = 0) in vec4 aPos;
uniform mat4 uVP;
uniform float uScale;
void main() {
  vec4 p = uVP * vec4(aPos.xyz, 1.0);
  p.z -= 0.0012 * p.w;
  gl_Position = p;
  gl_PointSize = clamp(aPos.w * uScale / max(p.w, 1e-4), 3.0, 64.0);
}`;

const PFS = `#version 300 es
precision highp float;
uniform vec4 uColor;
out vec4 outColor;
void main() {
  vec2 c = gl_PointCoord * 2.0 - 1.0;
  if (dot(c, c) > 1.0) discard;
  outColor = uColor;
}`;

interface TrimRun { accent: boolean; first: number; count: number }

const VS = `#version 300 es
layout(location = 0) in vec3 aPos;
uniform mat4 uVP;
void main() {
  vec4 p = uVP * vec4(aPos, 1.0);
  p.z -= 0.0008 * p.w;
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

export class Extras {
  seams: SeamLine[] = [];
  pins: PinMark[] = [];
  private readonly prog: WebGLProgram;
  private readonly vao: WebGLVertexArrayObject | null;
  private readonly buf: WebGLBuffer | null;
  private readonly uVP: WebGLUniformLocation | null;
  private readonly uColor: WebGLUniformLocation | null;
  private scratch = new Float32Array(1024);
  private gpuBytes = 0;
  private readonly tmp = new Float64Array(3);
  private readonly pprog: WebGLProgram;
  private readonly puVP: WebGLUniformLocation | null;
  private readonly puScale: WebGLUniformLocation | null;
  private readonly puColor: WebGLUniformLocation | null;
  private readonly tvao: WebGLVertexArrayObject | null;
  private readonly tbuf: WebGLBuffer | null;
  private readonly pvao: WebGLVertexArrayObject | null;
  private readonly pbuf: WebGLBuffer | null;
  private tscratch = new Float32Array(1024);
  private pscratch = new Float32Array(1024);
  private tbytes = 0;
  private pbytes = 0;
  private truns: TrimRun[] = [];
  private pcount = 0;
  private tgarment: Garment | null = null;
  private tversion = -1;

  constructor(private readonly gl: WebGL2RenderingContext) {
    const prog = gl.createProgram() as WebGLProgram;
    gl.attachShader(prog, compile(gl, gl.VERTEX_SHADER, VS));
    gl.attachShader(prog, compile(gl, gl.FRAGMENT_SHADER, FS));
    gl.linkProgram(prog);
    if (!gl.getProgramParameter(prog, gl.LINK_STATUS)) throw new Error(gl.getProgramInfoLog(prog) ?? 'link');
    this.prog = prog;
    this.uVP = gl.getUniformLocation(prog, 'uVP');
    this.uColor = gl.getUniformLocation(prog, 'uColor');
    this.vao = gl.createVertexArray();
    this.buf = gl.createBuffer();
    gl.bindVertexArray(this.vao);
    gl.bindBuffer(gl.ARRAY_BUFFER, this.buf);
    gl.enableVertexAttribArray(0);
    gl.vertexAttribPointer(0, 3, gl.FLOAT, false, 0, 0);
    gl.bindVertexArray(null);
    gl.bindBuffer(gl.ARRAY_BUFFER, null);
    const pprog = gl.createProgram() as WebGLProgram;
    gl.attachShader(pprog, compile(gl, gl.VERTEX_SHADER, PVS));
    gl.attachShader(pprog, compile(gl, gl.FRAGMENT_SHADER, PFS));
    gl.linkProgram(pprog);
    if (!gl.getProgramParameter(pprog, gl.LINK_STATUS)) throw new Error(gl.getProgramInfoLog(pprog) ?? 'link');
    this.pprog = pprog;
    this.puVP = gl.getUniformLocation(pprog, 'uVP');
    this.puScale = gl.getUniformLocation(pprog, 'uScale');
    this.puColor = gl.getUniformLocation(pprog, 'uColor');
    this.tvao = gl.createVertexArray();
    this.tbuf = gl.createBuffer();
    gl.bindVertexArray(this.tvao);
    gl.bindBuffer(gl.ARRAY_BUFFER, this.tbuf);
    gl.enableVertexAttribArray(0);
    gl.vertexAttribPointer(0, 3, gl.FLOAT, false, 0, 0);
    this.pvao = gl.createVertexArray();
    this.pbuf = gl.createBuffer();
    gl.bindVertexArray(this.pvao);
    gl.bindBuffer(gl.ARRAY_BUFFER, this.pbuf);
    gl.enableVertexAttribArray(0);
    gl.vertexAttribPointer(0, 4, gl.FLOAT, false, 0, 0);
    gl.bindVertexArray(null);
    gl.bindBuffer(gl.ARRAY_BUFFER, null);
  }

  syncTrims(garment: Garment, scene: Scene): void {
    if (garment === this.tgarment && scene.version === this.tversion) return;
    this.tgarment = garment;
    this.tversion = scene.version;
    const gl = this.gl;
    const t = trimBuffers(garment, (id) => {
      const b = scene.get(id);
      return b && b.ready && b.local ? { local: b.local, idx: b.idx, pos: b.pos } : null;
    });
    let lf = 0;
    for (const l of t.lines) lf += l.positions.length;
    let pf = 0;
    for (const p of t.points) pf += p.diameters.length * 4;
    if (this.tscratch.length < lf) this.tscratch = new Float32Array(lf * 2);
    if (this.pscratch.length < pf) this.pscratch = new Float32Array(pf * 2);
    this.truns.length = 0;
    let at = 0;
    for (const l of t.lines) {
      this.tscratch.set(l.positions, at);
      this.truns.push({ accent: l.layer === 'finish' || l.layer === 'fray' || l.layer === 'buttonhole', first: at / 3, count: l.positions.length / 3 });
      at += l.positions.length;
    }
    let pn = 0;
    for (const p of t.points) {
      for (let i = 0; i < p.diameters.length; i++) {
        this.pscratch[pn++] = p.positions[i * 3];
        this.pscratch[pn++] = p.positions[i * 3 + 1];
        this.pscratch[pn++] = p.positions[i * 3 + 2];
        this.pscratch[pn++] = p.diameters[i];
      }
    }
    this.pcount = pn / 4;
    if (at > 0) {
      gl.bindBuffer(gl.ARRAY_BUFFER, this.tbuf);
      if (at * 4 > this.tbytes) {
        this.tbytes = Math.max(at * 4, this.tbytes * 2);
        gl.bufferData(gl.ARRAY_BUFFER, this.tbytes, gl.DYNAMIC_DRAW);
      }
      gl.bufferSubData(gl.ARRAY_BUFFER, 0, this.tscratch, 0, at);
    }
    if (pn > 0) {
      gl.bindBuffer(gl.ARRAY_BUFFER, this.pbuf);
      if (pn * 4 > this.pbytes) {
        this.pbytes = Math.max(pn * 4, this.pbytes * 2);
        gl.bufferData(gl.ARRAY_BUFFER, this.pbytes, gl.DYNAMIC_DRAW);
      }
      gl.bufferSubData(gl.ARRAY_BUFFER, 0, this.pscratch, 0, pn);
    }
    gl.bindBuffer(gl.ARRAY_BUFFER, null);
  }

  drawTrims(f: Frame, pal: DrawPalette): void {
    if (this.truns.length === 0 && this.pcount === 0) return;
    const gl = this.gl;
    gl.enable(gl.DEPTH_TEST);
    gl.depthFunc(gl.LEQUAL);
    gl.depthMask(false);
    if (this.truns.length > 0) {
      gl.useProgram(this.prog);
      gl.uniformMatrix4fv(this.uVP, false, f.vp);
      gl.bindVertexArray(this.tvao);
      for (const r of this.truns) {
        const c = r.accent ? pal.accent : pal.line;
        gl.uniform4f(this.uColor, c[0], c[1], c[2], 0.95);
        gl.drawArrays(gl.LINES, r.first, r.count);
      }
    }
    if (this.pcount > 0) {
      gl.useProgram(this.pprog);
      gl.uniformMatrix4fv(this.puVP, false, f.vp);
      gl.uniform1f(this.puScale, 0.5 * f.height * f.proj[5]);
      gl.uniform4f(this.puColor, pal.line[0], pal.line[1], pal.line[2], 1);
      gl.bindVertexArray(this.pvao);
      gl.drawArrays(gl.POINTS, 0, this.pcount);
    }
    gl.depthMask(true);
    gl.disable(gl.DEPTH_TEST);
    gl.bindVertexArray(null);
  }

  pinAt(panel: number, vertex: number): number {
    for (const p of this.pins) if (p.panel === panel && p.vertex === vertex) return p.id;
    return 0;
  }

  drawSeams(f: Frame, scene: Scene, rgb: Float32Array): void {
    const lines = this.seams;
    if (lines.length === 0) return;
    const need = lines.length * 6;
    if (this.scratch.length < need) this.scratch = new Float32Array(need * 2);
    const s = this.scratch;
    let n = 0;
    for (let i = 0; i < lines.length; i++) {
      const l = lines[i];
      const a = scene.get(l.pa);
      const b = scene.get(l.pb);
      if (!a || !b || !a.ready || !b.ready || l.va * 3 + 2 >= a.pos.length || l.vb * 3 + 2 >= b.pos.length) continue;
      s[n++] = a.pos[l.va * 3];
      s[n++] = a.pos[l.va * 3 + 1];
      s[n++] = a.pos[l.va * 3 + 2];
      s[n++] = b.pos[l.vb * 3];
      s[n++] = b.pos[l.vb * 3 + 1];
      s[n++] = b.pos[l.vb * 3 + 2];
    }
    if (n === 0) return;
    const gl = this.gl;
    gl.useProgram(this.prog);
    gl.uniformMatrix4fv(this.uVP, false, f.vp);
    gl.uniform4f(this.uColor, rgb[0], rgb[1], rgb[2], 0.95);
    gl.bindVertexArray(this.vao);
    gl.bindBuffer(gl.ARRAY_BUFFER, this.buf);
    const bytes = n * 4;
    if (bytes > this.gpuBytes) {
      this.gpuBytes = Math.max(bytes, this.gpuBytes * 2);
      gl.bufferData(gl.ARRAY_BUFFER, this.gpuBytes, gl.DYNAMIC_DRAW);
    }
    gl.bufferSubData(gl.ARRAY_BUFFER, 0, s, 0, n);
    gl.enable(gl.DEPTH_TEST);
    gl.depthFunc(gl.LEQUAL);
    gl.depthMask(false);
    gl.drawArrays(gl.LINES, 0, n / 3);
    gl.depthMask(true);
    gl.disable(gl.DEPTH_TEST);
    gl.bindVertexArray(null);
    gl.bindBuffer(gl.ARRAY_BUFFER, null);
  }

  drawPins(ctx: CanvasRenderingContext2D, f: Frame, scene: Scene, dpr: number, accent: string, ring: string): void {
    const pins = this.pins;
    if (pins.length === 0) return;
    const t = this.tmp;
    const r = 4.5 * dpr;
    ctx.lineWidth = 1.5 * dpr;
    for (let i = 0; i < pins.length; i++) {
      const b = scene.get(pins[i].panel);
      const v = pins[i].vertex;
      if (!b || !b.ready || v * 3 + 2 >= b.pos.length) continue;
      if (!f.project(b.pos[v * 3], b.pos[v * 3 + 1], b.pos[v * 3 + 2], t)) continue;
      const x = t[0] * dpr;
      const y = t[1] * dpr;
      ctx.beginPath();
      ctx.arc(x, y, r, 0, Math.PI * 2);
      ctx.fillStyle = accent;
      ctx.fill();
      ctx.strokeStyle = ring;
      ctx.stroke();
    }
  }
}
