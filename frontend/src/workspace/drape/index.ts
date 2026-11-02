import { call, catalogue, onEngineLink } from '../../engine/service';
import { setClass, setDisabled, setHidden, setText, setTitle } from '../../ui/dom';
import { onLanguage, t } from '../../ui/i18n';
import { icon } from '../../ui/icons';
import { DEFAULT_PREFS, PREFS_EVENT, loadPrefs } from '../../ui/prefs';
import { toast } from '../../ui/toast';
import { PanelStrip } from './panelstate';
import { seamLines, type MeshInfo, type SeamJson } from './seams';
import { SimPanel } from './simpanel';
import { Tools } from './tools';
import type { DrapeHost, PanelStateJson, PinMark, SeamLine, SimState, TapeMark, Tool, WindJson } from './types';

export type { DrapeHost, PinMark, SeamLine } from './types';

const CSS = `
.ffd-group{display:flex;pointer-events:auto;overflow:hidden;border:1px solid var(--border-color);border-radius:var(--radius-sm);background:color-mix(in srgb,var(--bg-secondary) 88%,transparent)}
.ffd-group .ff3-btn{display:inline-flex;align-items:center;justify-content:center;gap:5px}
.ffd-group .ff3-btn:disabled{opacity:.4;cursor:default;pointer-events:none}
.ffd-group .ff3-btn svg{flex:none;display:block}
.ffd-run.is-on{background:var(--accent-color);color:var(--bg-primary)}
.ffd-run.is-on:hover{background:var(--accent-color);color:var(--bg-primary)}
.ffd-read{display:flex;align-items:center;justify-content:flex-end;box-sizing:border-box;width:17ch;height:24px;padding:0 6px;border:1px solid var(--border-color);border-radius:var(--radius-sm);background:color-mix(in srgb,var(--bg-secondary) 88%,transparent);color:var(--text-secondary);font:500 var(--fs-11)/1 ui-monospace,monospace;font-variant-numeric:tabular-nums;white-space:pre;pointer-events:none}
.ffd-read.is-empty{visibility:hidden}
`;

const PRESETS: Array<[string, string]> = [
  ['fast', 'Fast'],
  ['normal', 'Normal'],
  ['quality', 'Quality'],
  ['final', 'Final'],
];

const SYNC_KEYS = ['simSubsteps', 'simGravity', 'simThickness'] as const;
const POLL_RUN_MS = 350;
const POLL_IDLE_MS = 2000;
const OVERLAY_MS = 200;

interface GarmentPins {
  panels?: Array<{ id: number; outline?: unknown; inner?: unknown; particle_distance?: unknown; state?: PanelStateJson }>;
  pins?: Array<{ id: number; panel: number; point: [number, number] }>;
  tacks?: Array<{ id: number }>;
  wind?: WindJson;
}

interface MeshReply {
  boundary?: number[];
  boundary_param?: number[][];
  positions?: number[][];
}

interface GrabState {
  panel: number;
  vertex: number;
  target: [number, number, number] | null;
  dirty: boolean;
  busy: boolean;
  ended: boolean;
  pinch: boolean;
}

interface Caps {
  sim: boolean;
  pin: boolean;
  sew: boolean;
  mesh: boolean;
  apply: boolean;
  state: boolean;
  wind: boolean;
  refit: boolean;
  settings: boolean;
  shell: boolean;
}

interface TapeReply {
  tack?: number;
  panel?: number;
  vertex?: number;
  body_point?: [number, number, number];
}

export class Drape {
  readonly state: SimState = { running: false, frame: 0, vertices: 0, ms: 0, preset: 'normal', panels: 0 };
  private pinOn = false;
  private caps: Caps = { sim: false, pin: false, sew: false, mesh: false, apply: false, state: false, wind: false, refit: false, settings: false, shell: false };
  readonly tools: Tools;
  readonly tapes: TapeMark[] = [];
  wind: WindJson | null = null;
  private readonly strip: PanelStrip;
  private readonly panel: SimPanel;
  private readonly gearBtn: HTMLButtonElement;
  private selKey = new Set<number>();
  private readonly toggleBtn: HTMLButtonElement;
  private readonly playIcon: HTMLElement;
  private readonly pauseIcon: HTMLElement;
  private readonly stepBtn: HTMLButtonElement;
  private readonly resetBtn: HTMLButtonElement;
  private readonly applyBtn: HTMLButtonElement;
  private readonly pinBtn: HTMLButtonElement;
  private readonly clearBtn: HTMLButtonElement;
  private readonly presetBtns = new Map<string, HTMLButtonElement>();
  private readonly read: HTMLElement;
  private readonly meshes = new Map<number, MeshInfo>();
  private pollTimer = 0;
  private overlayTimer = 0;
  private overlaySeq = 0;
  private overlayKey = '';
  private grab: GrabState | null = null;
  private busy = false;

  constructor(bar: HTMLElement, private readonly host: DrapeHost) {
    if (!document.getElementById('ffd-style')) {
      const style = document.createElement('style');
      style.id = 'ffd-style';
      style.textContent = CSS;
      document.head.appendChild(style);
    }
    const group = (): HTMLElement => {
      const g = document.createElement('div');
      g.className = 'ffd-group';
      return g;
    };
    const button = (html: string, onClick: () => void): HTMLButtonElement => {
      const b = document.createElement('button');
      b.type = 'button';
      b.className = 'ff3-btn';
      b.innerHTML = html;
      b.disabled = true;
      b.addEventListener('click', () => { onClick(); b.blur(); });
      return b;
    };
    const run = group();
    this.toggleBtn = button(`<span class="ffd-i-play">${icon('simPlay', 13)}</span><span class="ffd-i-pause" hidden>${icon('simPause', 13)}</span><span class="ffd-l"></span>`, () => void this.toggle());
    this.toggleBtn.classList.add('ffd-run');
    this.playIcon = this.toggleBtn.querySelector('.ffd-i-play') as HTMLElement;
    this.pauseIcon = this.toggleBtn.querySelector('.ffd-i-pause') as HTMLElement;
    this.stepBtn = button(icon('simStep', 14), () => void this.command('sim.step', { frames: 1 }));
    this.resetBtn = button(icon('simReset', 14), () => void this.command('sim.reset'));
    run.append(this.toggleBtn, this.stepBtn, this.resetBtn);
    const quality = group();
    for (const [id, label] of PRESETS) {
      const b = button('', () => void this.setPreset(id));
      b.textContent = t(label);
      quality.appendChild(b);
      this.presetBtns.set(id, b);
    }
    const tools = group();
    this.applyBtn = button('', () => void this.command('sim.apply'));
    this.pinBtn = button(icon('pinTool', 14), () => this.setPinTool(!this.pinOn));
    this.clearBtn = button(icon('pinClear', 14), () => void this.command('pin.clear'));
    tools.append(this.applyBtn, this.pinBtn, this.clearBtn);
    this.read = document.createElement('div');
    this.read.className = 'ffd-read';
    this.tools = new Tools((tool) => this.toolChanged(tool), () => host.active());
    const settings = group();
    this.gearBtn = button(icon('settings', 14), () => { this.panel.toggle(this.gearBtn); this.paint(); });
    settings.appendChild(this.gearBtn);
    const pane = (bar.parentElement ?? bar) as HTMLElement;
    this.strip = new PanelStrip(pane);
    this.panel = new SimPanel(pane, { target: () => host.target() });
    bar.append(run, quality, tools, ...this.tools.root, settings, this.read);
    this.labels();
    this.paint();
    onLanguage(() => this.labels());
    onEngineLink((state) => {
      if (state !== 'up') return;
      void this.refreshCaps();
    });
    document.addEventListener('visibilitychange', () => { if (!document.hidden) this.schedule(0); });
    document.addEventListener(PREFS_EVENT, (e) => {
      const keys = (e as CustomEvent<{ keys?: string[] }>).detail?.keys ?? [];
      if (keys.some((k) => (SYNC_KEYS as readonly string[]).includes(k))) void this.pushPrefs(keys);
    });
  }

  get draping(): boolean { return this.state.running; }

  get pinTool(): boolean { return this.pinOn; }

  setPinTool(on: boolean): void {
    if (on === this.pinOn) return;
    this.pinOn = on;
    if (on) this.tools.set('none');
    this.paint();
    this.host.redraw();
  }

  private toolChanged(tool: Tool): void {
    if (tool !== 'none' && this.pinOn) this.setPinTool(false);
    this.host.redraw();
  }

  toggle(): Promise<void> {
    return this.command('sim.toggle');
  }

  private labels(): void {
    setText(this.toggleBtn.querySelector('.ffd-l'), t('Drape'));
    setTitle(this.toggleBtn, t('Drape on or off (Space)'));
    setTitle(this.stepBtn, t('Step'));
    setTitle(this.resetBtn, `${t('Reset')} - ${t('Back to the placed state')}`);
    setText(this.applyBtn, t('Apply'));
    setTitle(this.applyBtn, t('Keep the draped shape'));
    setTitle(this.pinBtn, t('Pin tool (P)'));
    setTitle(this.clearBtn, t('Remove every pin'));
    setTitle(this.gearBtn, t('Wind, refit and solver settings'));
    for (const [id, label] of PRESETS) {
      const b = this.presetBtns.get(id) as HTMLButtonElement;
      setText(b, t(label));
      setTitle(b, t(label));
    }
  }

  private paint(): void {
    const s = this.state;
    const c = this.caps;
    setClass(this.toggleBtn, 'is-on', s.running);
    setHidden(this.playIcon, s.running);
    setHidden(this.pauseIcon, !s.running);
    setDisabled(this.toggleBtn, !c.sim);
    setDisabled(this.stepBtn, !c.sim);
    setDisabled(this.resetBtn, !c.sim);
    setDisabled(this.applyBtn, !c.apply);
    setDisabled(this.pinBtn, !c.pin);
    setDisabled(this.clearBtn, !c.pin);
    setDisabled(this.gearBtn, !(c.wind || c.refit || c.settings || c.shell));
    setClass(this.pinBtn, 'is-on', this.pinOn);
    setClass(this.gearBtn, 'is-on', this.panel.open);
    for (const [id, b] of this.presetBtns) {
      setDisabled(b, !c.sim);
      setClass(b, 'is-on', id === s.preset);
    }
    setText(this.read, s.vertices > 0 ? `${s.ms.toFixed(0)} ms  ${s.vertices} v` : '');
    setClass(this.read, 'is-empty', s.vertices === 0);
  }

  private async refreshCaps(): Promise<void> {
    try {
      const cat = await catalogue(true);
      const has = (id: string): boolean => cat.commands.some((x) => x.id === id);
      this.caps = {
        sim: has('sim.state') && has('sim.toggle'), pin: has('pin.add') && has('pin.remove'), sew: has('sew.list'), mesh: has('mesh.get'), apply: has('sim.apply'),
        state: has('panel.state'), wind: has('wind.set'), refit: has('refit') && has('body.list'), settings: has('sim.settings'), shell: has('fitshell.set') && has('fitshell.get'),
      };
      this.tools.configure(cat.commands);
      this.panel.setCaps(this.caps);
      this.strip.setEnabled(this.caps.state);
    } catch { return; }
    this.paint();
    void this.pushPrefs(null);
    this.schedule(0);
    this.docChanged();
  }

  private schedule(ms: number): void {
    window.clearTimeout(this.pollTimer);
    if (!this.caps.sim || document.hidden) return;
    this.pollTimer = window.setTimeout(() => void this.poll(), ms);
  }

  private async poll(): Promise<void> {
    try {
      const r = await call<Record<string, unknown>>('sim.state');
      const last = (r.last ?? {}) as { ms?: number };
      const running = r.running === true;
      const changed = running !== this.state.running;
      this.state.running = running;
      this.state.frame = Number(r.frame ?? 0);
      this.state.vertices = Number(r.vertices ?? 0);
      this.state.ms = Number(last.ms ?? 0);
      this.state.preset = String(r.preset ?? 'normal');
      this.state.panels = Number(r.panels ?? 0);
      this.paint();
      if (changed) {
        this.host.redraw();
        if (!running) this.docChanged();
      }
    } catch { this.schedule(POLL_IDLE_MS); return; }
    this.schedule(this.state.running ? POLL_RUN_MS : POLL_IDLE_MS);
  }

  private async command(name: string, args: Record<string, unknown> = {}): Promise<void> {
    if (this.busy) return;
    this.busy = true;
    try {
      await call(name, args);
    } catch (err) {
      toast(err instanceof Error ? err.message : String(err), 'error');
    } finally {
      this.busy = false;
    }
    await this.poll();
  }

  private async setPreset(id: string): Promise<void> {
    try {
      const r = await call<{ preset?: string }>('sim.settings', { preset: id });
      this.state.preset = r.preset ?? id;
      this.paint();
    } catch (err) {
      toast(err instanceof Error ? err.message : String(err), 'error');
    }
  }

  private async pushPrefs(keys: string[] | null): Promise<void> {
    if (!this.caps.sim) return;
    const p = loadPrefs();
    const want = (k: (typeof SYNC_KEYS)[number]): boolean => (keys ? keys.includes(k) : p[k] !== DEFAULT_PREFS[k]);
    const args: Record<string, unknown> = {};
    if (want('simSubsteps')) args.substeps = Math.min(40, Math.round(p.simSubsteps));
    if (want('simGravity')) args.gravity = [0, -p.simGravity * 1000, 0];
    if (want('simThickness')) args.collision_thickness = p.simThickness;
    if (Object.keys(args).length === 0) return;
    try {
      const r = await call<{ preset?: string }>('sim.settings', args);
      this.state.preset = r.preset ?? 'custom';
      this.paint();
    } catch (err) {
      toast(err instanceof Error ? err.message : String(err), 'error');
    }
  }

  grabStart(panel: number, vertex: number, target: [number, number, number]): void {
    this.grab = { panel, vertex, target, dirty: true, busy: false, ended: false, pinch: this.tools.active === 'pinch' };
    this.pumpGrab();
  }

  grabMove(target: [number, number, number]): void {
    const g = this.grab;
    if (!g || g.ended) return;
    g.target = target;
    g.dirty = true;
    this.pumpGrab();
  }

  grabEnd(): void {
    const g = this.grab;
    if (!g || g.ended) return;
    g.ended = true;
    g.target = null;
    g.dirty = true;
    this.pumpGrab();
  }

  private pumpGrab(): void {
    const g = this.grab;
    if (!g || g.busy || !g.dirty) return;
    g.busy = true;
    g.dirty = false;
    const o = this.tools.opts;
    const task = g.pinch
      ? call('sim.pinch', { panel: g.panel, vertex: g.vertex, target: g.target, falloff: o.falloff, radius: o.radius, power: o.power, distance: o.distance })
      : call('sim.grab', { panel: g.panel, vertex: g.vertex, target: g.target });
    task.catch((err: unknown) => {
      g.ended = true;
      g.dirty = false;
      toast(err instanceof Error ? err.message : String(err), 'error');
    }).finally(() => {
      g.busy = false;
      if (g.dirty) this.pumpGrab();
      else if (g.ended && this.grab === g) this.grab = null;
    });
  }

  togglePin(panel: number, vertex: number): void {
    const id = this.host.pinAt(panel, vertex);
    const task = id > 0 ? call('pin.remove', { id }) : call('pin.add', { panel, vertex });
    task.catch((err: unknown) => toast(err instanceof Error ? err.message : String(err), 'error'));
  }

  syncSelection(sel: Set<number>): void {
    if (sel.size === this.selKey.size) {
      let same = true;
      for (const id of sel) {
        if (!this.selKey.has(id)) { same = false; break; }
      }
      if (same) return;
    }
    this.selKey = new Set(sel);
    this.syncStrip();
  }

  private syncStrip(): void {
    const ids = this.host.selected();
    this.strip.select(ids);
    const g = this.host.garment() as GarmentPins;
    const first = ids.length ? (g.panels ?? []).find((p) => p.id === ids[0]) : undefined;
    this.strip.sync(first?.state ?? null);
  }

  pinRegion(command: 'pin.box' | 'pin.lasso', hits: Array<[number, number[]]>): Promise<void> {
    return call(command, { panels: hits.map(([panel, vertices]) => ({ panel, vertices })) }).then(() => undefined, (err: unknown) => { toast(err instanceof Error ? err.message : String(err), 'error'); });
  }

  async flat(panel: number): Promise<number[][] | null> {
    try {
      return (await call<MeshReply>('mesh.get', { panel })).positions ?? null;
    } catch { return null; }
  }

  foldPlace(panel: number, line: [[number, number], [number, number]]): void {
    const o = this.tools.opts;
    call('fold.place', { panel, line, angle: o.angle, side: o.side }).catch((err: unknown) => toast(err instanceof Error ? err.message : String(err), 'error'));
  }

  async tape(panel: number, vertex: number): Promise<void> {
    try {
      const r = await call<TapeReply>('body.tape', { panel, vertex, length: this.tools.opts.length });
      if (r.tack !== undefined && r.body_point) {
        this.tapes.push({ id: r.tack, panel, vertex: r.vertex ?? vertex, point: r.body_point });
        this.host.redraw();
      }
    } catch (err) {
      toast(err instanceof Error ? err.message : String(err), 'error');
    }
  }

  docChanged(): void {
    const g = this.host.garment() as GarmentPins;
    this.wind = g.wind ?? null;
    this.panel.syncWind(this.wind);
    this.syncStrip();
    this.host.redraw();
    if (this.tapes.length) {
      const live = new Set((g.tacks ?? []).map((k) => k.id));
      for (let i = this.tapes.length - 1; i >= 0; i--) if (!live.has(this.tapes[i].id)) this.tapes.splice(i, 1);
    }
    window.clearTimeout(this.overlayTimer);
    this.overlayTimer = window.setTimeout(() => void this.refreshOverlay(), OVERLAY_MS);
  }

  private async meshOf(panel: number, key: string): Promise<MeshInfo | null> {
    const have = this.meshes.get(panel);
    if (have && have.key === key) return have;
    try {
      const r = await call<MeshReply>('mesh.get', { panel });
      if (!r.boundary || !r.boundary_param || !r.positions) return null;
      const info: MeshInfo = { key, segments: 0, boundary: r.boundary, param: r.boundary_param, positions: r.positions };
      this.meshes.set(panel, info);
      return info;
    } catch { return null; }
  }

  private async refreshOverlay(): Promise<void> {
    if (!this.caps.mesh) return;
    const seq = ++this.overlaySeq;
    const g = this.host.garment() as GarmentPins;
    const panels = new Map<number, { key: string; segments: number }>();
    for (const p of g.panels ?? []) {
      const outline = p.outline as unknown[] | undefined;
      panels.set(p.id, { key: JSON.stringify([p.outline, p.inner, p.particle_distance]), segments: Array.isArray(outline) ? outline.length : 0 });
    }
    const pins = g.pins ?? [];
    let seams: SeamJson[] = [];
    if (this.caps.sew && !this.state.running) {
      try { seams = ((await call<{ seams?: SeamJson[] }>('sew.list')).seams ?? []); } catch { seams = []; }
    }
    if (seq !== this.overlaySeq) return;
    const need = new Set<number>();
    for (const pin of pins) need.add(pin.panel);
    for (const s of seams) for (const sp of [...s.a, ...s.b]) need.add(sp.panel);
    const infos = new Map<number, MeshInfo>();
    await Promise.all([...need].map(async (id) => {
      const meta = panels.get(id);
      if (!meta) return;
      const info = await this.meshOf(id, meta.key);
      if (info) {
        info.segments = meta.segments;
        infos.set(id, info);
      }
    }));
    if (seq !== this.overlaySeq) return;
    for (const id of [...this.meshes.keys()]) if (!panels.has(id)) this.meshes.delete(id);
    const marks: PinMark[] = [];
    for (const pin of pins) {
      const info = infos.get(pin.panel);
      if (!info) continue;
      let best = -1;
      let bd = Infinity;
      for (let i = 0; i < info.positions.length; i++) {
        const dx = info.positions[i][0] - pin.point[0];
        const dy = info.positions[i][1] - pin.point[1];
        const d = dx * dx + dy * dy;
        if (d < bd) { bd = d; best = i; }
      }
      if (best >= 0) marks.push({ id: pin.id, panel: pin.panel, vertex: best });
    }
    const lines: SeamLine[] = this.state.running ? [] : seamLines(seams, infos);
    const key = JSON.stringify([marks, lines.length, lines[0], lines[lines.length - 1], this.state.running]);
    if (key === this.overlayKey) return;
    this.overlayKey = key;
    this.host.setPins(marks);
    this.host.setSeams(lines);
    this.host.redraw();
  }
}
