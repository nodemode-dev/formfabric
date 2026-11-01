import { anyDialogOpen } from '../../ui/dialogs';
import { textFieldFocused } from '../../ui/native-menu';
import { Selection } from '../state';
import { OrbitCamera } from './camera';
import { Frame } from './frame';
import { Gizmo, type Handle } from './gizmo';
import { Hud, type Preset } from './hud';
import { VIEW_STYLES, type GarmentFull, type PanelFull, type ViewStyle } from './model';
import { Picker } from './pick';
import { placementOf, quatAxisAngle, quatMul, quatNormalize } from './place';
import { MeshRenderer, type DrawPalette } from './renderer';
import { PanelBuf, Scene } from './scene';
import { MeshFeed } from './source';
import { StandaloneDoc, type DocLike } from './standalone';

export interface View3DLinks {
  doc: DocLike;
  sel: Selection;
  selected(): void;
}

const STYLE_KEY = 'formfabric.view3d.style';
const STATS_KEY = 'formfabric.view3d.stats';
const CLICK_SLOP = 4;
const HOLD_MS = 1500;
const STATS_MS = 250;

let current: View3D | null = null;

export function currentView3D(): View3D | null {
  return current;
}

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
  readonly scene = new Scene();
  readonly frame = new Frame();
  private readonly gl: WebGL2RenderingContext | null;
  private program: WebGLProgram | null = null;
  private vao: WebGLVertexArrayObject | null = null;
  private u: Record<string, WebGLUniformLocation | null> = {};
  private w = 0;
  private h = 0;
  private dpr = 1;
  private queued = false;
  private drag: { x: number; y: number; sx: number; sy: number; left: number; top: number; mode: 'orbit' | 'pan' | 'gizmo'; moved: boolean; shift: boolean; button: number } | null = null;
  private colors: { bg: [number, number, number]; line: [number, number, number]; x: [number, number, number]; z: [number, number, number]; text: string; bgCss: string } | null = null;
  private readonly pal: DrawPalette = { accent: new Float32Array(3), accentLin: new Float32Array(3), line: new Float32Array(3) };
  private readonly probe: HTMLElement;
  private readonly swatch: CanvasRenderingContext2D;
  private meshes: MeshRenderer | null = null;
  private readonly feedIn: MeshFeed;
  private readonly picker = new Picker();
  private readonly gizmo = new Gizmo();
  private readonly hud: Hud;
  private readonly doc: DocLike;
  private readonly sel: Selection;
  private readonly onSelected: () => void;
  private style: ViewStyle;
  private statsOn: boolean;
  private hover = -1;
  private hoverQueued = false;
  private hoverX = 0;
  private hoverY = 0;
  private pointerIn = false;
  private autoFrame = true;
  private framedCount = 0;
  private gizmoId = -1;
  private cursor = '';
  private statText = '';
  private statAt = 0;
  private statMs = 0;
  private font = '';
  private fontCss = '';
  private statShown = '';
  private statW = 0;
  private holdTimer = 0;
  private fence: WebGLSync | null = null;
  private fenceAt = 0;
  lastPaintMs = 0;
  gpuMs = 0;

  constructor(private readonly pane: HTMLElement, private readonly canvas: HTMLCanvasElement, links?: View3DLinks) {
    this.camera.target = [0, 900, 0];
    this.camera.distance = 4200;
    this.camera.yaw = Math.PI / 6;
    this.camera.pitch = Math.PI / 9;
    this.doc = links?.doc ?? new StandaloneDoc();
    this.sel = links?.sel ?? new Selection();
    this.onSelected = links?.selected ?? ((): void => undefined);
    const savedStyle = localStorage.getItem(STYLE_KEY) as ViewStyle | null;
    this.style = savedStyle && VIEW_STYLES.includes(savedStyle) ? savedStyle : 'shaded';
    this.statsOn = localStorage.getItem(STATS_KEY) === '1';
    this.gl = canvas.getContext('webgl2', { antialias: true, alpha: false });
    this.hud = new Hud(pane, { style: (s) => this.setStyle(s), preset: (p) => this.preset(p), stats: (on) => this.setStats(on) }, this.style, this.statsOn);
    this.feedIn = new MeshFeed(this.scene, () => this.onData(), () => this.redraw());
    current = this;
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
    canvas.addEventListener('pointerleave', () => this.leave());
    canvas.addEventListener('contextmenu', (e) => e.preventDefault());
    canvas.addEventListener('wheel', (e) => { e.preventDefault(); this.autoFrame = false; this.camera.zoom(Math.exp(e.deltaY * (e.ctrlKey ? 0.01 : 0.0012))); this.redraw(); }, { passive: false });
    canvas.addEventListener('dblclick', () => this.frameAll());
    pane.addEventListener('pointerenter', () => { this.pointerIn = true; });
    pane.addEventListener('pointerleave', () => { this.pointerIn = false; });
    document.addEventListener('keydown', (e) => this.onKey(e), true);
    this.doc.onChange(() => this.feedIn.sync(this.doc.garment as GarmentFull));
    this.feedIn.sync(this.doc.garment as GarmentFull);
    void this.feedIn.refreshCaps();
    this.resize();
  }

  get available(): boolean { return this.gl !== null; }

  feed(id: number, positions: Float32Array, indices: Uint32Array): void {
    this.feedIn.feed(id, positions, indices);
  }

  clearFed(): void {
    this.feedIn.clearFed();
  }

  setStyle(s: ViewStyle): void {
    if (s === this.style) return;
    this.style = s;
    localStorage.setItem(STYLE_KEY, s);
    this.hud.setStyle(s);
    this.redraw();
  }

  setStats(on: boolean): void {
    this.statsOn = on;
    localStorage.setItem(STATS_KEY, on ? '1' : '0');
    this.hud.setStats(on);
    this.statAt = 0;
    this.redraw();
  }

  preset(p: Preset): void {
    if (p === 'all') { this.frameAll(); return; }
    this.autoFrame = false;
    this.camera.snap(p);
    this.redraw();
  }

  frameAll(): void {
    this.autoFrame = false;
    this.frameBounds(null);
  }

  frameSelection(): void {
    this.autoFrame = false;
    this.frameBounds(this.sel.panels.size ? this.sel.panels : null);
  }

  private frameBounds(ids: Set<number> | null): void {
    const lo = new Float32Array(3);
    const hi = new Float32Array(3);
    if (!this.scene.bounds(ids, lo, hi) && !(ids && this.scene.bounds(null, lo, hi))) {
      this.camera.target = [0, 900, 0];
      this.camera.distance = 4200;
    } else {
      this.camera.frame([lo[0], lo[1], lo[2]], [hi[0], hi[1], hi[2]]);
      const aspect = this.w / Math.max(this.h, 1);
      if (aspect > 0 && aspect < 1) {
        const radius = 0.5 * Math.hypot(hi[0] - lo[0], hi[1] - lo[1], hi[2] - lo[2]);
        const half = Math.atan(Math.tan(this.camera.fov / 2) * aspect);
        this.camera.distance = Math.max(this.camera.distance, (radius / Math.sin(half)) * 1.1);
      }
    }
    this.redraw();
  }

  private onData(): void {
    const count = this.scene.list.length;
    if (count === 0) {
      this.autoFrame = true;
      this.framedCount = 0;
    } else if (this.autoFrame && count !== this.framedCount && this.scene.list.some((b) => b.ready)) {
      this.framedCount = count;
      this.frameBounds(null);
      return;
    }
    this.redraw();
  }

  private active(): boolean {
    if (this.pointerIn || document.activeElement === this.canvas) return true;
    return this.pane.closest('.ff-ws')?.getAttribute('data-layout') === '3d';
  }

  private onKey(e: KeyboardEvent): void {
    if (e.defaultPrevented || e.ctrlKey || e.metaKey || e.altKey || e.shiftKey || !this.active()) return;
    if (anyDialogOpen() || textFieldFocused() || document.querySelector('.cv-home:not([hidden])')) return;
    const k = e.key.toLowerCase();
    const presets: Record<string, Preset> = { '1': 'front', '2': 'back', '3': 'left', '4': 'right', '5': 'top', '0': 'all' };
    if (presets[k]) this.preset(presets[k]);
    else if (k === 'f') this.frameSelection();
    else return;
    e.preventDefault();
  }

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
    this.meshes = new MeshRenderer(gl, () => this.redraw());
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
    if (this.colors) return this.colors;
    const line = this.rgb('--text-primary');
    const accent = this.rgb('--accent-color');
    const bg = this.rgb('--canvas-bg');
    const bgCss = getComputedStyle(this.probe).color;
    this.probe.style.color = 'var(--text-secondary)';
    const text = getComputedStyle(this.probe).color;
    this.colors = { bg, line, x: [0.86, 0.32, 0.32], z: [0.32, 0.55, 0.9], text, bgCss };
    this.pal.line.set(line);
    this.pal.accent.set(accent);
    this.pal.accentLin.set([Math.pow(accent[0], 2.2), Math.pow(accent[1], 2.2), Math.pow(accent[2], 2.2)]);
    return this.colors;
  }

  private ensureFrame(): void {
    this.camera.aspect = this.w / Math.max(this.h, 1);
    this.frame.update(this.camera, this.w, this.h);
  }

  private gizmoTarget(): PanelBuf | null {
    if (!this.feedIn.arrangePlace || this.sel.panels.size !== 1) return null;
    const list = this.scene.list;
    for (let i = 0; i < list.length; i++) {
      const b = list[i];
      if (b.ready && b.source !== 'feed' && this.sel.panels.has(b.id)) return b;
    }
    return null;
  }

  private placeGizmo(b: PanelBuf): void {
    const m = b.model;
    const cx = (b.min[0] + b.max[0]) * 0.5;
    const cy = (b.min[1] + b.max[1]) * 0.5;
    const cz = (b.min[2] + b.max[2]) * 0.5;
    this.gizmo.place(
      m[0] * cx + m[4] * cy + m[8] * cz + m[12],
      m[1] * cx + m[5] * cy + m[9] * cz + m[13],
      m[2] * cx + m[6] * cy + m[10] * cz + m[14],
      this.camera.worldPerPixel(this.h),
    );
  }

  private onDown(e: PointerEvent): void {
    this.canvas.setPointerCapture(e.pointerId);
    const r = this.canvas.getBoundingClientRect();
    const pan = e.button === 1 || e.button === 2 || (e.button === 0 && e.shiftKey);
    let mode: 'orbit' | 'pan' | 'gizmo' = pan ? 'pan' : 'orbit';
    this.drag = { x: e.clientX, y: e.clientY, sx: e.clientX, sy: e.clientY, left: r.left, top: r.top, mode, moved: false, shift: e.shiftKey, button: e.button };
    const target = e.button === 0 && !e.shiftKey ? this.gizmoTarget() : null;
    if (!target) return;
    this.ensureFrame();
    this.placeGizmo(target);
    const handle = this.gizmo.hit(this.frame, e.clientX - r.left, e.clientY - r.top);
    if (!handle) return;
    mode = 'gizmo';
    this.drag.mode = mode;
    this.gizmoId = target.id;
    target.resetModel();
    this.gizmo.begin(handle, e.clientX - r.left, e.clientY - r.top);
  }

  private onMove(e: PointerEvent): void {
    const d = this.drag;
    if (!d) {
      const r = this.canvas.getBoundingClientRect();
      this.hoverX = e.clientX - r.left;
      this.hoverY = e.clientY - r.top;
      if (!this.hoverQueued) {
        this.hoverQueued = true;
        requestAnimationFrame(this.hoverCb);
      }
      return;
    }
    if (!d.moved && Math.hypot(e.clientX - d.sx, e.clientY - d.sy) > CLICK_SLOP) d.moved = true;
    if (d.mode === 'gizmo') {
      const b = this.scene.get(this.gizmoId);
      if (!b) return;
      this.ensureFrame();
      this.gizmo.update(this.frame, e.clientX - d.left, e.clientY - d.top);
      this.gizmo.matrix(b.model);
      b.moved = true;
      this.redraw();
      return;
    }
    const dx = e.clientX - d.x;
    const dy = e.clientY - d.y;
    d.x = e.clientX;
    d.y = e.clientY;
    if (!d.moved) return;
    this.autoFrame = false;
    if (d.mode === 'orbit') this.camera.orbit(dx, dy);
    else this.camera.pan(dx, dy, this.h);
    this.redraw();
  }

  private onUp(e: PointerEvent): void {
    if (this.canvas.hasPointerCapture(e.pointerId)) this.canvas.releasePointerCapture(e.pointerId);
    const d = this.drag;
    this.drag = null;
    if (!d) return;
    if (d.mode === 'gizmo') {
      void this.finishGizmo();
      return;
    }
    if (!d.moved && e.type === 'pointerup' && d.button === 0) this.click(e.clientX - d.left, e.clientY - d.top, d.shift);
  }

  private leave(): void {
    if (this.hover === -1 && !this.gizmo.hot) return;
    this.hover = -1;
    this.gizmo.hot = null;
    this.redraw();
  }

  private setCursor(c: string): void {
    if (c === this.cursor) return;
    this.cursor = c;
    this.canvas.style.cursor = c;
  }

  private hoverTick(): void {
    this.hoverQueued = false;
    if (this.drag || this.w === 0) return;
    this.ensureFrame();
    const target = this.gizmoTarget();
    let hot: Handle | null = null;
    if (target) {
      this.placeGizmo(target);
      hot = this.gizmo.hit(this.frame, this.hoverX, this.hoverY);
    }
    const before = this.gizmo.hot;
    const hotChanged = (hot?.kind ?? '') !== (before?.kind ?? '') || (hot?.axis ?? -1) !== (before?.axis ?? -1);
    this.gizmo.hot = hot;
    const id = hot ? -1 : this.picker.pick(this.frame, this.scene, this.hoverX, this.hoverY);
    this.setCursor(hot ? 'grab' : id >= 0 ? 'pointer' : '');
    if (id !== this.hover || hotChanged) {
      this.hover = id;
      this.redraw();
    }
  }

  private click(x: number, y: number, shift: boolean): void {
    this.ensureFrame();
    const id = this.picker.pick(this.frame, this.scene, x, y);
    const sel = this.sel;
    let changed = false;
    if (id >= 0) {
      if (shift) {
        if (!sel.panels.delete(id)) sel.panels.add(id);
        sel.pts = null;
        sel.seg = null;
        changed = true;
      } else if (!(sel.panels.size === 1 && sel.panels.has(id))) {
        sel.only(id);
        changed = true;
      }
    } else if (!shift && (sel.panels.size || sel.pts || sel.seg)) {
      sel.clear();
      changed = true;
    }
    if (changed) this.onSelected();
    this.redraw();
  }

  private async finishGizmo(): Promise<void> {
    const g = this.gizmo;
    const d = g.drag;
    const b = this.scene.get(this.gizmoId);
    const panel = b ? (this.doc.garment.panels as PanelFull[]).find((p) => p.id === b.id) : undefined;
    const changed = !!d && (d.handle.kind === 'move' ? Math.hypot(g.translate[0], g.translate[1], g.translate[2]) > 1e-6 : Math.abs(d.degrees) > 1e-4);
    if (!d || !b || !panel || !changed) {
      b?.resetModel();
      g.end();
      this.redraw();
      return;
    }
    const m = b.model;
    const { origin, rot } = placementOf(panel);
    const next: [number, number, number] = [
      m[0] * origin[0] + m[4] * origin[1] + m[8] * origin[2] + m[12],
      m[1] * origin[0] + m[5] * origin[1] + m[9] * origin[2] + m[13],
      m[2] * origin[0] + m[6] * origin[1] + m[10] * origin[2] + m[14],
    ];
    const turn = d.handle.kind === 'rotate' ? quatAxisAngle(d.handle.axis, (d.degrees * Math.PI) / 180) : ([0, 0, 0, 1] as [number, number, number, number]);
    const rotation = quatNormalize(quatMul(turn, rot));
    b.holdModel = true;
    g.end();
    window.clearTimeout(this.holdTimer);
    this.holdTimer = window.setTimeout(() => {
      if (!b.holdModel) return;
      b.resetModel();
      this.redraw();
    }, HOLD_MS);
    const result = await this.doc.run('arrange.place', { panel: b.id, origin: next, rotation });
    if (result === undefined && b.holdModel) {
      b.resetModel();
      this.redraw();
    }
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
    this.dpr = dpr;
    this.hud.resize(w, h, dpr);
    this.fontCss = `${Math.round(11 * dpr)}px ui-monospace, monospace`;
    this.font = '';
    this.paint();
  }

  redraw(): void {
    if (this.queued) return;
    this.queued = true;
    requestAnimationFrame(this.tick);
  }

  private readonly hoverCb = (): void => this.hoverTick();

  private readonly tick = (): void => {
    this.queued = false;
    this.paint();
  };

  private paint(): void {
    const gl = this.gl;
    if (!gl || !this.program || !this.meshes || this.w === 0 || this.h === 0) return;
    const t0 = performance.now();
    const p = this.palette();
    this.scene.upload(gl, this.style === 'shaded-mesh' || this.style === 'mesh');
    this.scene.recount();
    this.ensureFrame();
    gl.viewport(0, 0, this.canvas.width, this.canvas.height);
    gl.clearColor(p.bg[0], p.bg[1], p.bg[2], 1);
    gl.depthMask(true);
    gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);
    const vp = this.frame.vp;
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
    this.meshes.draw(this.frame, this.scene, this.style, this.sel.panels, this.hover, this.pal);
    const ms = performance.now() - t0;
    this.lastPaintMs = ms;
    this.statMs = this.statMs === 0 ? ms : this.statMs * 0.7 + ms * 0.3;
    if (this.statsOn && !this.fence) {
      this.fence = gl.fenceSync(gl.SYNC_GPU_COMMANDS_COMPLETE, 0);
      this.fenceAt = t0;
      gl.flush();
      window.setTimeout(this.pollCb, 1);
    }
    if (this.statsOn && t0 - this.statAt > STATS_MS) this.writeStats(t0);
    this.paintOverlay(p.text);
  }

  private writeStats(now: number): void {
    this.statAt = now;
    this.statText = `${this.statMs.toFixed(1)} ms cpu   ${this.gpuMs.toFixed(0)} ms done   ${this.scene.triangles} tris   ${this.scene.vertices} verts   ${this.scene.list.length} panels`;
  }

  private readonly pollCb = (): void => {
    const gl = this.gl;
    const f = this.fence;
    if (!gl || !f) return;
    if (gl.clientWaitSync(f, 0, 0) === gl.TIMEOUT_EXPIRED) {
      window.setTimeout(this.pollCb, 2);
      return;
    }
    this.gpuMs = performance.now() - this.fenceAt;
    gl.deleteSync(f);
    this.fence = null;
    if (!this.statsOn || !this.colors) return;
    this.writeStats(performance.now());
    this.paintOverlay(this.colors.text);
  };

  private paintOverlay(text: string): void {
    const ctx = this.hud.ctx;
    const W = this.hud.overlay.width;
    const H = this.hud.overlay.height;
    ctx.clearRect(0, 0, W, H);
    const target = this.gizmoTarget();
    if (target) {
      if (!this.gizmo.drag) this.placeGizmo(target);
      this.gizmo.draw(ctx, this.frame, this.dpr);
    }
    if (!this.statsOn || !this.statText) return;
    if (this.font !== this.fontCss) {
      this.font = this.fontCss;
      this.statShown = '';
      ctx.font = this.fontCss;
      ctx.textBaseline = 'bottom';
    }
    if (this.statText !== this.statShown) {
      this.statShown = this.statText;
      this.statW = ctx.measureText(this.statText).width;
    }
    ctx.globalAlpha = 0.78;
    ctx.fillStyle = this.colors?.bgCss ?? text;
    ctx.fillRect(4 * this.dpr, H - 26 * this.dpr, this.statW + 12 * this.dpr, 20 * this.dpr);
    ctx.globalAlpha = 1;
    ctx.fillStyle = text;
    ctx.fillText(this.statText, 10 * this.dpr, H - 10 * this.dpr);
  }
}
