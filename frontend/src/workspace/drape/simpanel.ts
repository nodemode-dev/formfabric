import { call, onEngineChanged } from '../../engine/service';
import { patchList, setDisabled, setHidden, setStyleVar, setText } from '../../ui/dom';
import { onLanguage, t } from '../../ui/i18n';
import { toast } from '../../ui/toast';
import type { WindJson } from './types';
import { button, ensureStyle, labelled, row, selectField, sliderField, toggleField, type Field } from './ui';

interface BodyRow {
  id: number;
  name: string;
  visible: boolean;
  triangles: number;
}

interface SettingsReply {
  params?: { method?: string };
  device?: { requested?: string; active?: string; gpu_available?: boolean; message?: string };
}

const KINDS: Array<[string, string]> = [['directional', 'Directional'], ['radial', 'Radial']];
const METHODS: Array<[string, string]> = [['auto', 'Auto'], ['blend', 'Blend'], ['sweep', 'Sweep'], ['redrape', 'Redrape']];
const SOLVERS: Array<[string, string]> = [['vbd', 'VBD'], ['implicit', 'Implicit']];
const DEVICES: Array<[string, string]> = [['cpu', 'CPU'], ['gpu', 'GPU']];
const REFIT_TOTAL = 80;
const POLL_MS = 350;

function fail(err: unknown): void {
  toast(err instanceof Error ? err.message : String(err), 'error');
}

export class SimPanel {
  readonly root: HTMLElement;
  private readonly secWind: HTMLElement;
  private readonly secRefit: HTMLElement;
  private readonly secSolver: HTMLElement;
  private readonly secShell: HTMLElement;
  private readonly shellBody: ReturnType<typeof selectField>;
  private readonly shellOn: Field<boolean>;
  private readonly shellOffset: Field<number>;
  private readonly enable: Field<boolean>;
  private readonly kind: ReturnType<typeof selectField>;
  private readonly dir: HTMLInputElement[] = [];
  private readonly strength: Field<number>;
  private readonly decay: Field<number>;
  private readonly turbulence: Field<number>;
  private readonly target: ReturnType<typeof selectField>;
  private readonly method: ReturnType<typeof selectField>;
  private readonly run: HTMLButtonElement;
  private readonly meter: HTMLElement;
  private readonly status: HTMLElement;
  private readonly solver: ReturnType<typeof selectField>;
  private readonly device: ReturnType<typeof selectField>;
  private readonly note: HTMLElement;
  private wind: WindJson | null = null;
  private bodies: BodyRow[] = [];
  private gpu = false;
  private busy = false;
  private pollTimer = 0;
  private from = 0;

  constructor(private readonly pane: HTMLElement, private readonly host: { target(): [number, number, number] }) {
    ensureStyle();
    const root = document.createElement('div');
    root.className = 'ffd-pop';
    root.hidden = true;
    this.root = root;

    this.secWind = this.section('Wind');
    const wbody = this.secWind.querySelector('.ffd-sec') as HTMLElement;
    this.enable = toggleField('Enable', (on) => this.sendWind({ enabled: on }));
    this.kind = selectField('Kind', KINDS, 'directional', (v) => this.sendWind({ kind: v }));
    const xyz = document.createElement('div');
    xyz.className = 'ffd-xyz';
    for (let i = 0; i < 3; i++) {
      const input = document.createElement('input');
      input.type = 'number';
      input.step = '0.1';
      input.value = '0';
      input.addEventListener('change', () => this.sendDirection());
      xyz.appendChild(input);
      this.dir.push(input);
    }
    const pct = (v: number): string => v.toFixed(0);
    this.strength = sliderField({ english: 'Strength', min: 0, max: 100, step: 0.5, value: 0, fmt: (v) => `${v.toFixed(1)}`, commit: (v) => this.sendWind({ strength: v }), row: true });
    this.decay = sliderField({ english: 'Decay', min: 0, max: 100, step: 1, value: 0, fmt: pct, commit: (v) => this.sendWind({ decay: v }), row: true });
    this.turbulence = sliderField({ english: 'Turbulence', min: 0, max: 100, step: 1, value: 0, fmt: pct, commit: (v) => this.sendWind({ turbulence: v }), row: true });
    const origin = button('Origin at view centre', () => this.sendWind({ origin: this.host.target() }));
    wbody.append(this.enable.root, this.kind.root, row('Direction', xyz), this.strength.root, this.decay.root, this.turbulence.root, origin);

    this.secRefit = this.section('Refit');
    const rbody = this.secRefit.querySelector('.ffd-sec') as HTMLElement;
    this.target = selectField('Target body', [], '', () => undefined);
    this.method = selectField('Method', METHODS, 'auto', () => undefined);
    this.run = button('Refit', () => void this.refit());
    this.meter = document.createElement('div');
    this.meter.className = 'ffd-meter';
    this.meter.appendChild(document.createElement('i'));
    this.status = document.createElement('div');
    rbody.append(this.target.root, this.method.root, this.run, this.meter, this.status);

    this.secSolver = this.section('Solver');
    const sbody = this.secSolver.querySelector('.ffd-sec') as HTMLElement;
    this.solver = selectField('Method', SOLVERS, 'implicit', (v) => void this.sendSettings({ method: v }));
    this.device = selectField('Device', DEVICES, 'cpu', (v) => void this.sendSettings({ device: v }));
    this.note = document.createElement('div');
    sbody.append(this.solver.root, this.device.root, this.note);

    this.secShell = this.section('Fit shell');
    const hbody = this.secShell.querySelector('.ffd-sec') as HTMLElement;
    this.shellBody = selectField('Body', [], '', () => void this.refreshShell());
    this.shellOn = toggleField('Enable', (on) => this.sendShell({ enabled: on }));
    this.shellOffset = sliderField({ english: 'Offset', min: 0, max: 100, step: 0.5, value: 10, fmt: (v) => `${v.toFixed(1)} mm`, commit: (v) => this.sendShell({ offset: v }), row: true });
    hbody.append(this.shellBody.root, this.shellOn.root, this.shellOffset.root);

    root.append(this.secWind, this.secRefit, this.secShell, this.secSolver);
    pane.appendChild(root);
    this.paintDevice();
    onLanguage(() => this.paintDevice());
    onEngineChanged((e) => { if (!this.root.hidden && /^(body|file|history|fitshell|sample)\./.test(e.name)) void this.refreshBodies(); });
  }

  private section(english: string): HTMLElement {
    const d = document.createElement('details');
    d.open = true;
    const s = document.createElement('summary');
    labelled(s, english);
    const body = document.createElement('div');
    body.className = 'ffd-sec';
    d.append(s, body);
    return d;
  }

  get open(): boolean { return !this.root.hidden; }

  setCaps(caps: { wind: boolean; refit: boolean; settings: boolean; shell: boolean }): void {
    setHidden(this.secShell, !caps.shell);
    setHidden(this.secWind, !caps.wind);
    setHidden(this.secRefit, !caps.refit);
    setHidden(this.secSolver, !caps.settings);
  }

  toggle(anchor: HTMLElement): void {
    if (this.open) { this.root.hidden = true; return; }
    this.root.hidden = false;
    const a = anchor.getBoundingClientRect();
    const p = this.pane.getBoundingClientRect();
    const left = Math.min(a.left - p.left, p.width - this.root.offsetWidth - 8);
    setStyleVar(this.root, 'left', `${Math.max(8, Math.round(left))}px`);
    setStyleVar(this.root, 'top', `${Math.round(a.bottom - p.top + 6)}px`);
    this.paintWind();
    void this.refreshSettings();
    void this.refreshBodies();
  }

  syncWind(w: WindJson | null): void {
    this.wind = w;
    if (!this.root.hidden) this.paintWind();
  }

  private paintWind(): void {
    const w = this.wind;
    this.enable.set(w?.enabled === true);
    this.kind.set(w?.kind || 'directional');
    for (let i = 0; i < 3; i++) {
      const input = this.dir[i];
      const v = String(Math.round((w?.direction[i] ?? 0) * 1000) / 1000);
      if (document.activeElement !== input && input.value !== v) input.value = v;
    }
    this.strength.set(w?.strength ?? 0);
    this.decay.set(w?.decay ?? 0);
    this.turbulence.set(w?.turbulence ?? 0);
  }

  private sendWind(args: Record<string, unknown>): void {
    call('wind.set', args).catch(fail);
  }

  private sendDirection(): void {
    const v = this.dir.map((i) => Number(i.value));
    if (v.some((x) => !Number.isFinite(x)) || v.every((x) => x === 0)) { this.paintWind(); return; }
    this.sendWind({ direction: v });
  }

  private async refreshSettings(): Promise<void> {
    try {
      const r = await call<SettingsReply>('sim.settings', {});
      this.solver.set(r.params?.method ?? 'implicit');
      this.gpu = r.device?.gpu_available === true;
      this.device.set(r.device?.active ?? 'cpu');
      this.paintDevice();
      setText(this.note, r.device?.message ?? '');
    } catch (err) { fail(err); }
  }

  private paintDevice(): void {
    const gpu = this.device.option('gpu');
    if (!gpu) return;
    setText(gpu, this.gpu ? t('GPU') : `${t('GPU')} (${t('unavailable')})`);
    if (gpu.disabled === this.gpu) gpu.disabled = !this.gpu;
  }

  private async sendSettings(args: Record<string, unknown>): Promise<void> {
    try {
      await call('sim.settings', args);
    } catch (err) { fail(err); }
    await this.refreshSettings();
  }

  private async refreshBodies(): Promise<void> {
    try {
      const r = await call<{ bodies?: BodyRow[] }>('body.list', {});
      this.bodies = (r.bodies ?? []).filter((b) => b.triangles > 0);
    } catch { return; }
    const first = this.bodies.find((b) => b.visible) ?? this.bodies[0];
    this.from = first ? first.id : 0;
    const sel = this.target.select;
    const keep = sel.value;
    patchList(sel, this.bodies.filter((b) => b.id !== this.from), {
      key: (b) => String(b.id),
      create: () => document.createElement('option'),
      update: (node, b) => {
        const o = node as HTMLOptionElement;
        if (o.value !== String(b.id)) o.value = String(b.id);
        setText(o, b.name);
      },
    });
    if (keep && Array.from(sel.options).some((o) => o.value === keep)) sel.value = keep;
    setDisabled(this.run, sel.options.length === 0 || this.busy);
    const shellSel = this.shellBody.select;
    const shellKeep = shellSel.value;
    patchList(shellSel, this.bodies, {
      key: (b) => String(b.id),
      create: () => document.createElement('option'),
      update: (node, b) => {
        const o = node as HTMLOptionElement;
        if (o.value !== String(b.id)) o.value = String(b.id);
        setText(o, b.name);
      },
    });
    const wanted = shellKeep && this.bodies.some((b) => String(b.id) === shellKeep) ? shellKeep : String(this.from || '');
    if (wanted && shellSel.value !== wanted) shellSel.value = wanted;
    await this.refreshShell();
  }

  private async refreshShell(): Promise<void> {
    const body = Number(this.shellBody.select.value);
    if (!body) return;
    try {
      const r = await call<{ offset?: number; enabled?: boolean }>('fitshell.get', { body });
      this.shellOn.set(r.enabled === true);
      this.shellOffset.set(Number.isFinite(Number(r.offset)) ? Number(r.offset) : 10);
    } catch { /* the engine may lack fitshell.get */ }
  }

  private sendShell(args: Record<string, unknown>): void {
    const body = Number(this.shellBody.select.value);
    if (!body) return;
    call('fitshell.set', { body, ...args }).then(() => this.refreshShell()).catch(fail);
  }

  private async refit(): Promise<void> {
    const to = Number(this.target.select.value);
    if (this.busy || !to) return;
    this.busy = true;
    setDisabled(this.run, true);
    this.meter.classList.add('is-indet');
    setStyleVar(this.meter, '--p', '0');
    setText(this.status, t('Refitting'));
    let start = NaN;
    const poll = async (): Promise<void> => {
      try {
        const s = await call<{ frame?: number }>('sim.state', {});
        const f = Number(s.frame);
        if (!Number.isFinite(f)) return;
        if (Number.isNaN(start)) { start = f; return; }
        if (f > start && this.busy) {
          this.meter.classList.remove('is-indet');
          setStyleVar(this.meter, '--p', String(Math.min(0.98, (f - start) / REFIT_TOTAL)));
        }
      } catch { return; }
    };
    void poll();
    this.pollTimer = window.setInterval(() => void poll(), POLL_MS);
    try {
      const args: Record<string, unknown> = { to, method: this.method.select.value };
      if (this.from) args.from = this.from;
      await call('refit', args);
      setStyleVar(this.meter, '--p', '1');
      setText(this.status, t('Refit done'));
    } catch (err) {
      fail(err);
      setStyleVar(this.meter, '--p', '0');
      setText(this.status, '');
    } finally {
      window.clearInterval(this.pollTimer);
      this.meter.classList.remove('is-indet');
      this.busy = false;
      setDisabled(this.run, false);
      void this.refreshBodies();
    }
  }
}
