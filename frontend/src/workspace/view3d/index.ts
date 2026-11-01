import { OrbitCamera } from './camera';
import { multiply } from './math';

const VERT = `#version 300 es
in vec2 aPos;
uniform mat4 uVP;
uniform vec2 uCenter;
uniform float uSize;
out vec3 vW;
void main() {
  vec3 w = vec3(uCenter.x + aPos.x * uSize, 0.0, uCenter.y + aPos.y * uSize);
  vW = w;
  gl_Position = uVP * vec4(w, 1.0);
}`;

const FRAG = `#version 300 es
precision highp float;
in vec3 vW;
uniform vec2 uCenter;
uniform float uCell;
uniform float uFade;
uniform vec3 uLine;
uniform vec3 uAxisX;
uniform vec3 uAxisZ;
out vec4 outColor;
float lines(vec2 p, float cell) {
  vec2 c = p / cell;
  vec2 d = max(fwidth(c), vec2(1e-6));
  vec2 g = abs(fract(c - 0.5) - 0.5) / d;
  float line = 1.0 - min(min(g.x, g.y), 1.0);
  return line * (1.0 - smoothstep(0.2, 0.5, max(d.x, d.y)));
}
void main() {
  vec2 p = vW.xz;
  float fade = 1.0 - smoothstep(uFade * 0.15, uFade, distance(p, uCenter));
  float a = max(lines(p, uCell) * 0.28, max(lines(p, uCell * 10.0) * 0.6, lines(p, uCell * 100.0) * 0.8)) * fade;
  vec3 col = uLine;
  vec2 d = max(fwidth(p), vec2(1e-6));
  float ax = (1.0 - min(abs(p.y) / d.y, 1.0)) * fade;
  float az = (1.0 - min(abs(p.x) / d.x, 1.0)) * fade;
  if (ax > a) { a = ax * 0.9; col = uAxisX; }
  if (az > a) { a = az * 0.9; col = uAxisZ; }
  if (a < 0.003) discard;
  outColor = vec4(col, a);
}`;

function compile(gl: WebGL2RenderingContext, type: number, src: string): WebGLShader {
  const sh = gl.createShader(type) as WebGLShader;
  gl.shaderSource(sh, src);
  gl.compileShader(sh);
  if (!gl.getShaderParameter(sh, gl.COMPILE_STATUS)) throw new Error(gl.getShaderInfoLog(sh) ?? 'shader');
  return sh;
}

export class View3D {
  readonly camera = new OrbitCamera();
  private readonly gl: WebGL2RenderingContext | null;
  private program: WebGLProgram | null = null;
  private vao: WebGLVertexArrayObject | null = null;
  private u: Record<string, WebGLUniformLocation | null> = {};
  private w = 0;
  private h = 0;
  private queued = false;
  private drag: { x: number; y: number; mode: 'orbit' | 'pan' } | null = null;
  private colors: { bg: [number, number, number]; line: [number, number, number]; x: [number, number, number]; z: [number, number, number] } | null = null;
  private readonly probe: HTMLElement;
  private readonly swatch: CanvasRenderingContext2D;

  constructor(private readonly pane: HTMLElement, private readonly canvas: HTMLCanvasElement) {
    this.camera.target = [0, 900, 0];
    this.camera.distance = 4200;
    this.camera.yaw = Math.PI / 6;
    this.camera.pitch = Math.PI / 9;
    this.gl = canvas.getContext('webgl2', { antialias: true, alpha: false });
    this.probe = document.createElement('span');
    this.probe.style.cssText = 'position:absolute;width:0;height:0;visibility:hidden;pointer-events:none';
    pane.appendChild(this.probe);
    const sw = document.createElement('canvas');
    sw.width = 1;
    sw.height = 1;
    this.swatch = sw.getContext('2d', { willReadFrequently: true }) as CanvasRenderingContext2D;
    if (this.gl) this.setup(this.gl);
    new ResizeObserver(() => this.resize()).observe(pane);
    const dirty = (): void => { this.colors = null; this.redraw(); };
    new MutationObserver(dirty).observe(document.documentElement, { attributes: true });
    new MutationObserver(dirty).observe(document.body, { attributes: true, attributeFilter: ['class'] });
    document.addEventListener('nm:ui-settings-changed', dirty);
    canvas.addEventListener('pointerdown', (e) => this.onDown(e));
    canvas.addEventListener('pointermove', (e) => this.onMove(e));
    canvas.addEventListener('pointerup', (e) => this.onUp(e));
    canvas.addEventListener('pointercancel', (e) => this.onUp(e));
    canvas.addEventListener('contextmenu', (e) => e.preventDefault());
    canvas.addEventListener('wheel', (e) => { e.preventDefault(); this.camera.zoom(Math.exp(e.deltaY * (e.ctrlKey ? 0.01 : 0.0012))); this.redraw(); }, { passive: false });
    canvas.addEventListener('dblclick', () => { this.camera.home(); this.camera.target = [0, 900, 0]; this.camera.distance = 4200; this.redraw(); });
    this.resize();
  }

  get available(): boolean { return this.gl !== null; }

  private setup(gl: WebGL2RenderingContext): void {
    const prog = gl.createProgram() as WebGLProgram;
    gl.attachShader(prog, compile(gl, gl.VERTEX_SHADER, VERT));
    gl.attachShader(prog, compile(gl, gl.FRAGMENT_SHADER, FRAG));
    gl.bindAttribLocation(prog, 0, 'aPos');
    gl.linkProgram(prog);
    if (!gl.getProgramParameter(prog, gl.LINK_STATUS)) throw new Error(gl.getProgramInfoLog(prog) ?? 'link');
    this.program = prog;
    for (const n of ['uVP', 'uCenter', 'uSize', 'uCell', 'uFade', 'uLine', 'uAxisX', 'uAxisZ']) this.u[n] = gl.getUniformLocation(prog, n);
    this.vao = gl.createVertexArray();
    gl.bindVertexArray(this.vao);
    const buf = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buf);
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([-1, -1, 1, -1, -1, 1, 1, 1]), gl.STATIC_DRAW);
    gl.enableVertexAttribArray(0);
    gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 0, 0);
    gl.bindVertexArray(null);
  }

  private rgb(name: string): [number, number, number] {
    this.probe.style.color = `var(${name})`;
    const css = getComputedStyle(this.probe).color;
    this.swatch.clearRect(0, 0, 1, 1);
    this.swatch.fillStyle = css;
    this.swatch.fillRect(0, 0, 1, 1);
    const d = this.swatch.getImageData(0, 0, 1, 1).data;
    return [d[0] / 255, d[1] / 255, d[2] / 255];
  }

  private palette(): NonNullable<View3D['colors']> {
    this.colors ??= { bg: this.rgb('--canvas-bg'), line: this.rgb('--text-primary'), x: [0.86, 0.32, 0.32], z: [0.32, 0.55, 0.9] };
    return this.colors;
  }

  private onDown(e: PointerEvent): void {
    this.canvas.setPointerCapture(e.pointerId);
    const pan = e.button === 1 || e.button === 2 || (e.button === 0 && e.shiftKey);
    this.drag = { x: e.clientX, y: e.clientY, mode: pan ? 'pan' : 'orbit' };
  }

  private onMove(e: PointerEvent): void {
    if (!this.drag) return;
    const dx = e.clientX - this.drag.x;
    const dy = e.clientY - this.drag.y;
    this.drag.x = e.clientX;
    this.drag.y = e.clientY;
    if (this.drag.mode === 'orbit') this.camera.orbit(dx, dy);
    else this.camera.pan(dx, dy, this.h);
    this.redraw();
  }

  private onUp(e: PointerEvent): void {
    if (this.canvas.hasPointerCapture(e.pointerId)) this.canvas.releasePointerCapture(e.pointerId);
    this.drag = null;
  }

  private resize(): void {
    const r = this.pane.getBoundingClientRect();
    const w = Math.max(0, Math.round(r.width));
    const h = Math.max(0, Math.round(r.height));
    const dpr = window.devicePixelRatio || 1;
    const bw = Math.round(w * dpr);
    const bh = Math.round(h * dpr);
    if (w === this.w && h === this.h && this.canvas.width === bw) return;
    this.w = w;
    this.h = h;
    this.canvas.width = bw;
    this.canvas.height = bh;
    this.canvas.style.width = `${w}px`;
    this.canvas.style.height = `${h}px`;
    this.paint();
  }

  redraw(): void {
    if (this.queued) return;
    this.queued = true;
    requestAnimationFrame(() => { this.queued = false; this.paint(); });
  }

  private paint(): void {
    const gl = this.gl;
    if (!gl || !this.program || this.w === 0 || this.h === 0) return;
    const p = this.palette();
    gl.viewport(0, 0, this.canvas.width, this.canvas.height);
    gl.clearColor(p.bg[0], p.bg[1], p.bg[2], 1);
    gl.clear(gl.COLOR_BUFFER_BIT);
    this.camera.aspect = this.w / this.h;
    const vp = multiply(this.camera.projection(), this.camera.view());
    const dist = this.camera.distance;
    const cell = Math.pow(10, Math.floor(Math.log10(dist * 0.005)));
    gl.useProgram(this.program);
    gl.uniformMatrix4fv(this.u.uVP, false, vp);
    gl.uniform2f(this.u.uCenter, this.camera.target[0], this.camera.target[2]);
    gl.uniform1f(this.u.uSize, dist * 60);
    gl.uniform1f(this.u.uCell, cell);
    gl.uniform1f(this.u.uFade, dist * 8);
    gl.uniform3f(this.u.uLine, p.line[0], p.line[1], p.line[2]);
    gl.uniform3f(this.u.uAxisX, p.x[0], p.x[1], p.x[2]);
    gl.uniform3f(this.u.uAxisZ, p.z[0], p.z[1], p.z[2]);
    gl.enable(gl.BLEND);
    gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);
    gl.disable(gl.DEPTH_TEST);
    gl.bindVertexArray(this.vao);
    gl.drawArrays(gl.TRIANGLE_STRIP, 0, 4);
    gl.bindVertexArray(null);
  }
}
