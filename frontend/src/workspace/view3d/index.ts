import { anyDialogOpen } from '../../ui/dialogs';
import { textFieldFocused } from '../../ui/native-menu';
import { Drape } from '../drape';
import type { Tool } from '../drape/types';
import { Selection, fmtLen } from '../state';
import { hasCommand, quiet } from '../viz/api';
import { VizUi } from '../viz';
import type { WorldHit } from '../viz/host';
import { vizOf, type VizJson } from '../viz/types';
import { drawTapes, drawWind } from './annot';
import { OrbitCamera } from './camera';
import { Extras } from './extras';
import { newFx } from './fx';
import { FurGL } from './furgl';
import { Frame } from './frame';
import { StageGL, parseBackground } from './stagegl';
import { StyleGL, parseStyleLines } from './stylegl';
import { Gizmo, type Handle } from './gizmo';
import { Hud, type Preset } from './hud';
import { VIEW_STYLES, type GarmentFull, type PanelFull, type ViewStyle } from './model';
import { Picker } from './pick';
import { placementOf, quatAxisAngle, quatMul, quatNormalize } from './place';
import { MeshRenderer, type DrawPalette } from './renderer';
import { PanelBuf, Scene } from './scene';
import { SelectLayer, type Drawn } from './selectlayer';
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

function insidePolygon(p: number[], x: number, y: number): boolean {
  let hit = false;
  for (let i = 0, j = p.length - 2; i < p.length; j = i, i += 2) {
    if ((p[i + 1] > y) !== (p[j + 1] > y) && x < ((p[j] - p[i]) * (y - p[i + 1])) / (p[j + 1] - p[i + 1]) + p[i]) hit = !hit;
  }
  return hit;
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
  private drag: { x: number; y: number; sx: number; sy: number; left: number; top: number; mode: 'orbit' | 'pan' | 'gizmo' | 'grab' | 'tool'; moved: boolean; shift: boolean; button: number } | null = null;
  private grabPlane: { panel: number; p: Float64Array } | null = null;
  private foldPanel = -1;
  private foldStart: { tri: number; u: number; v: number } | null = null;
  private readonly layer: SelectLayer;
  private extras: Extras | null = null;
  private readonly drape: Drape;
  private accentCss = '';
  private ringCss = '';
  private readonly ro = new Float64Array(3);
  private readonly rd = new Float64Array(3);
  private colors: { bg: [number, number, number]; line: [number, number, number]; x: [number, number, number]; z: [number, number, number]; text: string; bgCss: string } | null = null;
  private readonly pal: DrawPalette = { accent: new Float32Array(3), accentLin: new Float32Array(3), line: new Float32Array(3), bgLin: new Float32Array(3) };
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
  readonly fx = newFx();
  private stageGL: StageGL | null = null;
  private fur: FurGL | null = null;
  private styleGL: StyleGL | null = null;
  private viz: VizUi | null = null;
  private vizJson: VizJson = vizOf(null);
  private vizSource: unknown = null;
  private styleAt = 0;
  private styleBusy = false;
  private styleTimer = 0;
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
    this.layer = new SelectLayer(pane);
    this.drape = new Drape(this.hud.bar, {
      active: () => this.active(),
      selected: () => [...this.sel.panels],
      target: () => [this.camera.target[0], this.camera.target[1], this.camera.target[2]],
      redraw: () => this.redraw(),
      setSeams: (lines) => { if (this.extras) this.extras.seams = lines; },
      setPins: (pins) => { if (this.extras) this.extras.pins = pins; },
      pinAt: (panel, vertex) => this.extras?.pinAt(panel, vertex) ?? 0,
      garment: () => this.doc.garment,
    });
    current = this;
    this.probe = document.createElement('span');
    this.probe.style.cssText = 'position:absolute;width:0;height:0;visibility:hidden;pointer-events:none';
    pane.appendChild(this.probe);
    const sw = document.createElement('canvas');
    sw.width = 1;
    sw.height = 1;
    this.swatch = sw.getContext('2d', { willReadFrequently: true }) as CanvasRenderingContext2D;
    if (this.gl) this.setup(this.gl);
    this.feedIn.onScalars = (_id, f) => this.viz?.onScalars(f);
    this.viz = new VizUi({
      pane,
      bar: this.hud.bar,
      canvas,
      camera: this.camera,
      scene: this.scene,
      fx: this.fx,
      gl: () => this.gl,
      viz: () => this.vizJson,
      units: () => this.doc.garment.units,
      redraw: () => this.redraw(),
      paintNow: () => this.paint(),
      snapshot: (w, h, transparent) => this.renderSnapshot(w, h, transparent),
      maxSize: () => (this.gl ? (this.gl.getParameter(this.gl.MAX_RENDERBUFFER_SIZE) as number) : 0),
      clearScalars: () => { for (const b of this.scene.list) this.scene.setScalars(b, null); this.redraw(); },
      setAutoFrame: (on) => { this.autoFrame = on; },
    });
    new ResizeObserver(() => this.resize()).observe(pane);
    document.addEventListener('ff:thumb', (e) => {
      const d = (e as CustomEvent<{ out: HTMLCanvasElement | null; w: number; h: number }>).detail;
      d.out = this.renderSnapshot(d.w, d.h, false);
    });
    const dirty =(): void => { this.colors = null; this.redraw(); };
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
    this.doc.onChange(() => {
      this.feedIn.sync(this.doc.garment as GarmentFull);
      this.applyViz();
      this.applyStates();
      this.drape.docChanged();
      this.redraw();
    });
    this.feedIn.sync(this.doc.garment as GarmentFull);
    this.applyViz();
    void this.feedIn.refreshCaps();
    this.resize();
  }

  get available(): boolean { return this.gl !== null; }

  private applyViz(): void {
    if (this.vizSource !== this.doc.garment) {
      this.vizSource = this.doc.garment;
      this.vizJson = vizOf(this.doc.garment);
      this.stageGL?.setStage(this.vizJson.stage);
      this.fur?.setFur(this.vizJson.fur);
    }
    this.viz?.docChanged();
    this.scheduleStyle();
    this.redraw();
  }

  private scheduleStyle(): void {
    if (this.styleTimer) return;
    const wait = Math.max(120, 700 - (performance.now() - this.styleAt));
    this.styleTimer = window.setTimeout(() => { this.styleTimer = 0; void this.pullStyle(); }, wait);
  }

  private async pullStyle(): Promise<void> {
    const gl = this.styleGL;
    if (!gl || this.styleBusy) return;
    const g = this.doc.garment as unknown as { style_lines?: unknown[] };
    if (!g.style_lines || g.style_lines.length === 0) {
      if (gl.count > 0) { gl.set([]); this.redraw(); }
      return;
    }
    if (!hasCommand('styleline.list3d')) return;
    this.styleBusy = true;
    this.styleAt = performance.now();
    const r = await quiet<unknown>('styleline.list3d', {});
    this.styleBusy = false;
    if (r === undefined) return;
    gl.set(parseStyleLines(r));
    this.redraw();
  }

  private hitOf(id: number): WorldHit | null {
    const b = id >= 0 ? this.scene.get(id) : undefined;
    const tri = this.picker.hitTri;
    if (!b || tri < 0) return null;
    const i0 = b.idx[tri * 3] * 3;
    const i1 = b.idx[tri * 3 + 1] * 3;
    const i2 = b.idx[tri * 3 + 2] * 3;
    const u = this.picker.hitU;
    const v = this.picker.hitV;
    const w = 1 - u - v;
    const m = b.model;
    const x = w * b.pos[i0] + u * b.pos[i1] + v * b.pos[i2];
    const y = w * b.pos[i0 + 1] + u * b.pos[i1 + 1] + v * b.pos[i2 + 1];
    const z = w * b.pos[i0 + 2] + u * b.pos[i1 + 2] + v * b.pos[i2 + 2];
    return {
      panel: id,
      vertex: this.picker.vertex,
      point: [m[0] * x + m[4] * y + m[8] * z + m[12], m[1] * x + m[5] * y + m[9] * z + m[13], m[2] * x + m[6] * y + m[10] * z + m[14]],
    };
  }

  private applyStates(): void {
    let changed = false;
    for (const p of this.doc.garment.panels as PanelFull[]) {
      const b = this.scene.get(p.id);
      if (!b) continue;
      const k = p.state?.disabled ? 2 : p.state?.frozen ? 1 : 0;
      if (b.stateKind !== k) {
        b.stateKind = k;
        changed = true;
      }
    }
    if (changed) this.redraw();
  }

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
    this.scheduleStyle();
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
    if (e.defaultPrevented || e.ctrlKey || e.metaKey || e.shiftKey || !this.active()) return;
    if (anyDialogOpen() || textFieldFocused() || document.querySelector('.cv-home:not([hidden])')) return;
    if (e.altKey) {
      const n = ['Digit1', 'Digit2', 'Digit3', 'Digit4'].indexOf(e.code);
      if (n >= 0) this.setStyle(VIEW_STYLES[n]);
      else if (e.code === 'Digit0') this.setStats(!this.statsOn);
      else return;
      e.preventDefault();
      return;
    }
    const k = e.key.toLowerCase();
    const presets: Record<string, Preset> = { '1': 'front', '2': 'back', '3': 'left', '4': 'right', '5': 'top', '0': 'all' };
    if (presets[k]) this.preset(presets[k]);
    else if (k === 'f') this.frameSelection();
    else if (k === 'p') this.drape.setPinTool(!this.drape.pinTool);
    else if (k === 'escape' && this.drape.pinTool) this.drape.setPinTool(false);
    else if (e.code === 'Space' && !(e.target instanceof HTMLButtonElement)) {
      if (!e.repeat) void this.drape.toggle();
    } else return;
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
    this.extras = new Extras(gl);
    this.stageGL = new StageGL(gl);
    this.stageGL.onLoaded = () => { this.viz?.hdriStatus(this.stageGL?.hdriState ?? 'none'); this.redraw(); };
    this.fur = new FurGL(gl, () => this.redraw());
    this.styleGL = new StyleGL(gl);
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
    this.accentCss = `rgb(${Math.round(accent[0] * 255)} ${Math.round(accent[1] * 255)} ${Math.round(accent[2] * 255)})`;
    this.ringCss = bgCss;
    this.pal.accentLin.set([Math.pow(accent[0], 2.2), Math.pow(accent[1], 2.2), Math.pow(accent[2], 2.2)]);
    this.pal.bgLin.set([Math.pow(bg[0], 2.2), Math.pow(bg[1], 2.2), Math.pow(bg[2], 2.2)]);
    return this.colors;
  }

  private ensureFrame(): void {
    this.camera.aspect = this.w / Math.max(this.h, 1);
    this.frame.update(this.camera, this.w, this.h);
  }

  private gizmoTarget(): PanelBuf | null {
    if (!this.feedIn.arrangePlace || this.sel.panels.size !== 1 || this.drape.draping) return null;
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
    const tool = this.drape.tools.active;
    if (e.button === 0 && !e.shiftKey && this.startTool(tool, e.clientX - r.left, e.clientY - r.top)) {
      this.drag.mode = 'tool';
      return;
    }
    if (e.button === 0 && !e.shiftKey && this.drape.draping && !this.drape.pinTool && (tool === 'none' || tool === 'pinch') && this.startGrab(e.clientX - r.left, e.clientY - r.top)) {
      this.drag.mode = 'grab';
      return;
    }
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

  private startTool(tool: Tool, x: number, y: number): boolean {
    if (tool !== 'box' && tool !== 'lasso' && tool !== 'fold') return false;
    this.palette();
    if (tool === 'fold') {
      this.ensureFrame();
      const id = this.picker.pick(this.frame, this.scene, x, y);
      if (id < 0 || this.scene.get(id)?.source === 'feed') return false;
      this.foldPanel = id;
      this.foldStart = { tri: this.picker.hitTri, u: this.picker.hitU, v: this.picker.hitV };
      this.layer.begin('line', x, y, this.accentCss);
      return true;
    }
    this.layer.begin(tool, x, y, this.accentCss);
    return true;
  }

  private finishTool(r: Drawn | null): void {
    if (!r) return;
    const p = r.pts;
    if (r.kind === 'line') {
      this.finishFold(p);
      return;
    }
    if (r.kind === 'box' ? Math.hypot(p[2] - p[0], p[3] - p[1]) < 3 : p.length < 6) return;
    const x0 = Math.min(p[0], p[2]);
    const x1 = Math.max(p[0], p[2]);
    const y0 = Math.min(p[1], p[3]);
    const y1 = Math.max(p[1], p[3]);
    const out = new Float64Array(3);
    const hits: Array<[number, number[]]> = [];
    this.ensureFrame();
    for (const b of this.scene.list) {
      if (!b.ready || b.kind !== 'panel' || b.source === 'local' || b.source === 'feed') continue;
      const list: number[] = [];
      for (let i = 0; i < b.vcount; i++) {
        if (!this.frame.project(b.pos[i * 3], b.pos[i * 3 + 1], b.pos[i * 3 + 2], out)) continue;
        const inside = r.kind === 'box' ? out[0] >= x0 && out[0] <= x1 && out[1] >= y0 && out[1] <= y1 : insidePolygon(p, out[0], out[1]);
        if (inside) list.push(i);
      }
      if (list.length) hits.push([b.id, list]);
    }
    const command = r.kind === 'box' ? 'pin.box' : 'pin.lasso';
    if (hits.length) void this.drape.pinRegion(command, hits);
  }

  private finishFold(p: number[]): void {
    const id = this.foldPanel;
    const a = this.foldStart;
    const buf = this.scene.get(id);
    if (!a || !buf || p.length < 4) return;
    this.ensureFrame();
    if (this.picker.pick(this.frame, this.scene, p[2], p[3]) !== id) return;
    const b = { tri: this.picker.hitTri, u: this.picker.hitU, v: this.picker.hitV };
    void this.drape.flat(id).then((flat) => {
      if (!flat) return;
      const at = (h: { tri: number; u: number; v: number }): [number, number] | null => {
        const i0 = flat[buf.idx[h.tri * 3]];
        const i1 = flat[buf.idx[h.tri * 3 + 1]];
        const i2 = flat[buf.idx[h.tri * 3 + 2]];
        if (!i0 || !i1 || !i2) return null;
        const w = 1 - h.u - h.v;
        return [w * i0[0] + h.u * i1[0] + h.v * i2[0], w * i0[1] + h.u * i1[1] + h.v * i2[1]];
      };
      const s = at(a);
      const e = at(b);
      if (!s || !e || Math.hypot(e[0] - s[0], e[1] - s[1]) < 1) return;
      this.drape.foldPlace(id, [s, e]);
    });
  }

  private startGrab(x: number, y: number): boolean {
    this.ensureFrame();
    const id = this.picker.pick(this.frame, this.scene, x, y);
    const b = id >= 0 ? this.scene.get(id) : undefined;
    const v = this.picker.vertex;
    if (!b || b.source !== 'stream' || v < 0 || v * 3 + 2 >= b.pos.length) return false;
    const p = new Float64Array([b.pos[v * 3], b.pos[v * 3 + 1], b.pos[v * 3 + 2]]);
    this.grabPlane = { panel: id, p };
    this.drape.grabStart(id, v, [p[0], p[1], p[2]]);
    this.canvas.style.cursor = 'grabbing';
    return true;
  }

  private moveGrab(x: number, y: number): void {
    const g = this.grabPlane;
    if (!g) return;
    this.ensureFrame();
    if (!this.frame.ray(x, y, this.ro, this.rd)) return;
    const n = this.frame.fwd;
    const den = this.rd[0] * n[0] + this.rd[1] * n[1] + this.rd[2] * n[2];
    if (Math.abs(den) < 1e-9) return;
    const t = ((g.p[0] - this.ro[0]) * n[0] + (g.p[1] - this.ro[1]) * n[1] + (g.p[2] - this.ro[2]) * n[2]) / den;
    this.drape.grabMove([this.ro[0] + this.rd[0] * t, this.ro[1] + this.rd[1] * t, this.ro[2] + this.rd[2] * t]);
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
    if (d.mode === 'grab') {
      this.moveGrab(e.clientX - d.left, e.clientY - d.top);
      return;
    }
    if (d.mode === 'tool') {
      this.layer.move(e.clientX - d.left, e.clientY - d.top);
      return;
    }
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
    if (d.mode === 'grab') {
      this.grabPlane = null;
      this.drape.grabEnd();
      this.cursor = '';
      this.canvas.style.cursor = '';
      return;
    }
    if (d.mode === 'tool') {
      this.finishTool(e.type === 'pointerup' ? this.layer.finish() : (this.layer.finish(), null));
      return;
    }
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
    const aim = this.drape.pinTool || (this.drape.tools.active !== 'none' && this.drape.tools.active !== 'pinch') || !!this.viz?.measure.active;
    this.setCursor(hot ? 'grab' : aim ? 'crosshair' : id >= 0 ? (this.drape.draping ? 'grab' : 'pointer') : '');
    if (id !== this.hover || hotChanged) {
      this.hover = id;
      this.redraw();
    }
  }

  private click(x: number, y: number, shift: boolean): void {
    this.ensureFrame();
    const id = this.picker.pick(this.frame, this.scene, x, y);
    if (this.viz?.click(this.hitOf(id))) return;
    if (this.drape.tools.active === 'tape') {
      if (id >= 0 && this.picker.vertex >= 0 && this.scene.get(id)?.source !== 'local') void this.drape.tape(id, this.picker.vertex);
      return;
    }
    if (this.drape.pinTool) {
      if (id >= 0 && this.picker.vertex >= 0 && this.scene.get(id)?.source !== 'local') this.drape.togglePin(id, this.picker.vertex);
      return;
    }
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

  private resizeTimer = 0;

  private resize(): void {
    const r = this.pane.getBoundingClientRect();
    const w = Math.max(0, Math.round(r.width));
    const h = Math.max(0, Math.round(r.height));
    if (this.w > 0 && this.h > 0 && w > 0 && h > 0 && (w !== this.w || h !== this.h)) {
      this.canvas.style.width = `${w}px`;
      this.canvas.style.height = `${h}px`;
      window.clearTimeout(this.resizeTimer);
      this.resizeTimer = window.setTimeout(() => this.afterResize(), 140);
      return;
    }
    this.settle();
  }

  private afterResize(): void {
    if (document.documentElement.hasAttribute('data-ft-dock-drag')) {
      this.resizeTimer = window.setTimeout(() => this.afterResize(), 140);
      return;
    }
    this.settle();
  }

  private settle(): void {
    window.clearTimeout(this.resizeTimer);
    const r = this.pane.getBoundingClientRect();
    const w = Math.max(0, Math.round(r.width));
    const h = Math.max(0, Math.round(r.height));
    const dpr = window.devicePixelRatio || 1;
    const bw = Math.round(w * dpr);
    const bh = Math.round(h * dpr);
    if (w === this.w && h === this.h && this.canvas.width === bw) {
      this.canvas.style.width = `${w}px`;
      this.canvas.style.height = `${h}px`;
      return;
    }
    this.w = w;
    this.h = h;
    this.canvas.width = bw;
    this.canvas.height = bh;
    this.canvas.style.width = `${w}px`;
    this.canvas.style.height = `${h}px`;
    this.dpr = dpr;
    this.hud.resize(w, h, dpr);
    this.layer.resize(w, h, dpr);
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
    this.drape.syncSelection(this.sel.panels);
    this.scene.upload(gl, this.style === 'shaded-mesh' || this.style === 'mesh');
    this.scene.recount();
    this.ensureFrame();
    this.drawWorld(this.frame, this.canvas.width, this.canvas.height, this.sel.panels, this.hover, false, p);
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

  private drawWorld(f: Frame, pw: number, ph: number, sel: Set<number>, hover: number, still: boolean, p: NonNullable<View3D['colors']>): void {
    const gl = this.gl as WebGL2RenderingContext;
    const meshes = this.meshes as MeshRenderer;
    const sg = this.stageGL;
    const fx = this.fx;
    sg?.sync(this.scene, fx);
    this.fur?.sync(this.scene);
    gl.viewport(0, 0, pw, ph);
    const custom = sg?.active ? parseBackground(this.vizJson.stage.background) : null;
    const bg = custom ?? p.bg;
    gl.clearColor(bg[0], bg[1], bg[2], still ? 0 : 1);
    gl.depthMask(true);
    gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);
    const sky = !still && !!sg?.hasSky;
    const ground = !still && !!sg?.groundOn;
    if (sky && sg) sg.drawSky(f, fx);
    if (ground && sg) {
      sg.floor.set([Math.pow(p.bg[0] * 0.6 + p.line[0] * 0.25, 2.2), Math.pow(p.bg[1] * 0.6 + p.line[1] * 0.25, 2.2), Math.pow(p.bg[2] * 0.6 + p.line[2] * 0.25, 2.2)]);
      sg.drawGround(f, fx);
    }
    if (!still && !sky && !ground) {
      const dist = this.camera.distance;
      const cell = Math.pow(10, Math.floor(Math.log10(dist * 0.005)));
      gl.useProgram(this.program);
      gl.uniformMatrix4fv(this.u.uVP, false, f.vp);
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
    meshes.draw(f, this.scene, this.style, sel, hover, this.pal, fx);
    this.fur?.draw(f);
    if (this.extras && !this.drape.draping) this.extras.drawSeams(f, this.scene, this.pal.accent);
    this.styleGL?.draw(f, this.pal.accent);
    if (this.extras) {
      this.extras.syncTrims(this.doc.garment, this.scene);
      this.extras.drawTrims(f, this.pal);
    }
  }

  private renderSnapshot(w: number, h: number, transparent: boolean): HTMLCanvasElement | null {
    const gl = this.gl;
    if (!gl || !this.program || !this.meshes) return null;
    const p = this.palette();
    this.scene.upload(gl, this.style === 'shaded-mesh' || this.style === 'mesh');
    const samples = Math.min(4, gl.getParameter(gl.MAX_SAMPLES) as number);
    const fbo = gl.createFramebuffer();
    const resolve = gl.createFramebuffer();
    const color = gl.createRenderbuffer();
    const depth = gl.createRenderbuffer();
    const out = gl.createRenderbuffer();
    gl.bindRenderbuffer(gl.RENDERBUFFER, color);
    gl.renderbufferStorageMultisample(gl.RENDERBUFFER, samples, gl.RGBA8, w, h);
    gl.bindRenderbuffer(gl.RENDERBUFFER, depth);
    gl.renderbufferStorageMultisample(gl.RENDERBUFFER, samples, gl.DEPTH_COMPONENT24, w, h);
    gl.bindRenderbuffer(gl.RENDERBUFFER, out);
    gl.renderbufferStorage(gl.RENDERBUFFER, gl.RGBA8, w, h);
    gl.bindFramebuffer(gl.FRAMEBUFFER, fbo);
    gl.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER, color);
    gl.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.DEPTH_ATTACHMENT, gl.RENDERBUFFER, depth);
    gl.bindFramebuffer(gl.FRAMEBUFFER, resolve);
    gl.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER, out);
    this.stageGL?.sync(this.scene, this.fx);
    gl.bindFramebuffer(gl.FRAMEBUFFER, fbo);
    const ok = gl.checkFramebufferStatus(gl.FRAMEBUFFER) === gl.FRAMEBUFFER_COMPLETE;
    let canvas: HTMLCanvasElement | null = null;
    if (ok) {
      const frame = new Frame();
      const aspect = this.camera.aspect;
      this.camera.aspect = w / h;
      frame.update(this.camera, w, h);
      this.drawWorld(frame, w, h, new Set<number>(), -1, transparent, p);
      this.camera.aspect = aspect;
      gl.bindFramebuffer(gl.READ_FRAMEBUFFER, fbo);
      gl.bindFramebuffer(gl.DRAW_FRAMEBUFFER, resolve);
      gl.blitFramebuffer(0, 0, w, h, 0, 0, w, h, gl.COLOR_BUFFER_BIT, gl.NEAREST);
      gl.bindFramebuffer(gl.READ_FRAMEBUFFER, resolve);
      const raw = new Uint8Array(w * h * 4);
      gl.readPixels(0, 0, w, h, gl.RGBA, gl.UNSIGNED_BYTE, raw);
      const flipped = new Uint8ClampedArray(w * h * 4);
      const row = w * 4;
      for (let y = 0; y < h; y++) flipped.set(raw.subarray((h - 1 - y) * row, (h - y) * row), y * row);
      canvas = document.createElement('canvas');
      canvas.width = w;
      canvas.height = h;
      (canvas.getContext('2d') as CanvasRenderingContext2D).putImageData(new ImageData(flipped, w, h), 0, 0);
    }
    gl.bindFramebuffer(gl.FRAMEBUFFER, null);
    gl.deleteFramebuffer(fbo);
    gl.deleteFramebuffer(resolve);
    gl.deleteRenderbuffer(color);
    gl.deleteRenderbuffer(depth);
    gl.deleteRenderbuffer(out);
    gl.viewport(0, 0, this.canvas.width, this.canvas.height);
    this.redraw();
    return canvas;
  }

  paintNow(): void {
    this.paint();
  }

  private writeStats(now: number): void {
    this.statAt = now;
    this.statText = `${this.statMs.toFixed(1)} ms cpu   ${this.gpuMs.toFixed(0)} ms done   ${this.scene.triangles} tris   ${this.scene.vertices} verts   ${this.scene.panels} panels`;
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
    this.extras?.drawPins(ctx, this.frame, this.scene, this.dpr, this.accentCss, this.ringCss);
    const wind = this.drape.wind;
    if (wind?.enabled) drawWind(ctx, this.frame, this.dpr, wind, this.camera.worldPerPixel(this.h), this.accentCss);
    drawTapes(ctx, this.frame, this.scene, this.drape.tapes, this.dpr, this.accentCss, this.ringCss, (mm) => fmtLen(mm, this.doc.garment.units));
    this.viz?.draw(ctx, this.frame, this.scene, this.dpr, this.accentCss, this.ringCss);
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
