import type { ScalarsFrame } from '../../engine/stream';
import { setClass, setDisabled, setHidden, setText, setTitle } from '../../ui/dom';
import { onLanguage, t } from '../../ui/i18n';
import { MAP_KINDS } from '../view3d/scalars';
import { hasCommand, missing, num, obj, quiet, run } from './api';
import type { VizHost } from './host';
import { Pop } from './pop';
import { cssGradient, MAP_RAMP, MAP_UNIT, rampBytes, type RampName } from './ramps';

const NAMES = ['none', 'strain', 'stress', 'pressure', 'fit', 'contact', 'thickness', 'compare'];
const LABELS: Record<string, string> = {
  none: 'None',
  strain: 'Strain',
  stress: 'Stress',
  pressure: 'Pressure',
  fit: 'Fit',
  contact: 'Contact',
  thickness: 'Thickness',
  compare: 'Compare',
};

function fmt(v: number): string {
  if (!Number.isFinite(v)) return '-';
  const a = Math.abs(v);
  if (a !== 0 && (a < 0.01 || a >= 10000)) return v.toExponential(1);
  return String(Number(v.toPrecision(3)));
}

export class Maps {
  kind = 'none';
  private readonly items = new Map<string, HTMLButtonElement>();
  private readonly legend: HTMLDivElement;
  private readonly title: HTMLElement;
  private readonly bar: HTMLElement;
  private readonly lo: HTMLElement;
  private readonly hi: HTMLElement;
  private readonly unit: HTMLElement;
  private readonly ramps = new Map<RampName, WebGLTexture>();
  private fixed = false;
  private seen: { min: number; max: number } | null = null;
  private shownBar = '';
  private busy = false;

  constructor(private readonly host: VizHost, private readonly pop: Pop) {
    pop.setLabel('Maps');
    for (const k of MAP_KINDS) {
      const b = pop.item(LABELS[k], () => void this.choose(k));
      this.items.set(k, b);
      if (k === 'none') pop.separator();
    }
    this.legend = document.createElement('div');
    this.legend.className = 'ffz-legend';
    this.legend.hidden = true;
    this.legend.innerHTML = '<div class="ffz-lg-title"><span data-k="t"></span><span data-k="u"></span></div><div class="ffz-lg-bar" data-k="bar"></div><div class="ffz-lg-ends"><span data-k="lo"></span><span data-k="hi"></span></div>';
    this.title = this.legend.querySelector('[data-k="t"]') as HTMLElement;
    this.unit = this.legend.querySelector('[data-k="u"]') as HTMLElement;
    this.bar = this.legend.querySelector('[data-k="bar"]') as HTMLElement;
    this.lo = this.legend.querySelector('[data-k="lo"]') as HTMLElement;
    this.hi = this.legend.querySelector('[data-k="hi"]') as HTMLElement;
    host.pane.appendChild(this.legend);
    onLanguage(() => this.paintLegend());
    this.setKind('none');
  }

  available(): void {
    const ok = hasCommand('map.show');
    for (const [k, b] of this.items) {
      setDisabled(b, k !== 'none' && !ok);
    }
    setTitle(this.pop.button, ok ? t('Analysis maps') : missing('map.show'));
    this.pop.button.disabled = false;
  }

  private ramp(name: RampName): WebGLTexture | null {
    const gl = this.host.gl();
    if (!gl) return null;
    let tex = this.ramps.get(name);
    if (!tex) {
      tex = gl.createTexture() as WebGLTexture;
      gl.bindTexture(gl.TEXTURE_2D, tex);
      gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 256, 1, 0, gl.RGBA, gl.UNSIGNED_BYTE, rampBytes(name, 256));
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
      gl.bindTexture(gl.TEXTURE_2D, null);
      this.ramps.set(name, tex);
    }
    return tex;
  }

  contextRestored(): void {
    this.ramps.clear();
    if (this.kind !== 'none') this.host.fx.map.ramp = this.ramp(MAP_RAMP[this.kind] ?? 'viridis');
  }

  private setKind(kind: string): void {
    this.kind = kind;
    delete this.unit.dataset.unit;
    for (const [k, b] of this.items) setClass(b, 'is-on', k === kind);
    this.pop.setActive(kind !== 'none');
    this.pop.setLabel(kind === 'none' ? 'Maps' : LABELS[kind] ?? 'Maps');
    const fx = this.host.fx.map;
    fx.on = kind !== 'none';
    if (kind === 'none') {
      fx.ramp = null;
      this.host.clearScalars();
      this.seen = null;
      this.fixed = false;
    } else {
      fx.ramp = this.ramp(MAP_RAMP[kind] ?? 'viridis');
    }
    this.paintLegend();
    this.host.redraw();
  }

  async choose(kind: string): Promise<void> {
    if (this.busy) return;
    if (kind === this.kind) {
      if (kind !== 'none') await this.choose('none');
      return;
    }
    this.busy = true;
    try {
      this.seen = null;
      this.fixed = false;
      const r = await run<unknown>('map.show', { kind });
      if (r === undefined) return;
      this.setKind(kind);
      if (kind !== 'none') await this.stats(kind, r);
    } finally {
      this.busy = false;
    }
  }

  private async stats(kind: string, shown: unknown): Promise<void> {
    const o = obj(shown);
    let lo: number | null = null;
    let hi: number | null = null;
    if (Array.isArray(o.range) && o.range.length === 2) {
      lo = num(o.range[0]);
      hi = num(o.range[1]);
    }
    if ((lo === null || hi === null) && hasCommand('map.stats')) {
      const s = obj(await quiet<unknown>('map.stats', { kind }));
      lo = num(s.min);
      hi = num(s.max);
    }
    if (lo !== null && hi !== null && hi > lo) {
      this.host.fx.map.min = lo;
      this.host.fx.map.max = hi;
      this.fixed = true;
    }
    if (typeof o.unit === 'string') this.unit.dataset.unit = o.unit;
    this.paintLegend();
    this.host.redraw();
  }

  sync(map: string | null): void {
    if (!map || !MAP_KINDS.includes(map as never) || this.busy || map === this.kind) return;
    this.setKind(map);
  }

  onScalars(f: ScalarsFrame): void {
    const name = NAMES[f.map] ?? this.kind;
    if (name === 'none') {
      if (f.vertexCount === 0) this.sync('none');
      return;
    }
    if (this.kind === 'none' || (name !== this.kind && !this.busy)) {
      this.seen = null;
      this.fixed = false;
      delete this.unit.dataset.unit;
      this.kind = name;
      for (const [k, b] of this.items) setClass(b, 'is-on', k === name);
      this.pop.setActive(true);
      this.pop.setLabel(LABELS[name] ?? 'Maps');
      this.host.fx.map.on = true;
      this.host.fx.map.ramp = this.ramp(MAP_RAMP[name] ?? 'viridis');
    }
    if (!this.fixed && f.max > f.min) {
      const s = this.seen;
      this.seen = s ? { min: Math.min(s.min, f.min), max: Math.max(s.max, f.max) } : { min: f.min, max: f.max };
      this.host.fx.map.min = this.seen.min;
      this.host.fx.map.max = this.seen.max;
    }
    this.paintLegend();
  }

  private paintLegend(): void {
    const k = this.kind;
    setHidden(this.legend, k === 'none');
    if (k === 'none') return;
    setText(this.title, t(LABELS[k] ?? k));
    setText(this.unit, this.unit.dataset.unit ?? MAP_UNIT[k] ?? '');
    const m = this.host.fx.map;
    setText(this.lo, fmt(m.min));
    setText(this.hi, fmt(m.max));
    const g = cssGradient(MAP_RAMP[k] ?? 'viridis');
    if (g !== this.shownBar) {
      this.shownBar = g;
      this.bar.style.background = g;
    }
  }
}
