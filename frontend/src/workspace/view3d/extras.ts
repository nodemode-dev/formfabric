import type { PinMark, SeamLine } from '../drape/types';
import type { Frame } from './frame';
import type { Scene } from './scene';

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
