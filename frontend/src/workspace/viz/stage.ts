import { nativeDialogs, pickPath } from '../../shell/file-dialog';
import { el, patchList, q, setClass, setDisabled, setHidden, setText, setValue } from '../../ui/dom';
import { icon } from '../../ui/icons';
import { onLanguage, t } from '../../ui/i18n';
import '../fabric/fabric.css';
import { colorField, label, later, numField, selectField, titled, toggleField } from '../fabric/widgets';
import { hasCommand, missing, run } from './api';
import type { VizHost } from './host';
import type { LightJson, LightKind } from './types';
import './viz.css';

const OPEN_KEY = 'formfabric.stage.open';
const KINDS: ReadonlyArray<readonly [string, string]> = [
  ['directional', 'Directional'],
  ['point', 'Point'],
  ['spot', 'Spot'],
  ['area', 'Area'],
];

const HTML =
  '<header class="ff-fb-head"><span class="ff-fb-title" data-k="title"></span><button class="cv-icon-btn" type="button" data-k="close"></button></header>' +
  '<div class="ff-fb-scroll">' +
  '<details class="ff-fb-sec" open data-s="lights"><summary><span class="ff-fb-chev"></span><span data-k="h-lights"></span></summary><div class="ff-fb-body" data-k="lights"></div></details>' +
  '<details class="ff-fb-sec" open data-s="props"><summary><span class="ff-fb-chev"></span><span data-k="h-props"></span></summary><div class="ff-fb-body" data-k="props"></div></details>' +
  '<details class="ff-fb-sec" open data-s="env"><summary><span class="ff-fb-chev"></span><span data-k="h-env"></span></summary><div class="ff-fb-body" data-k="env"></div></details>' +
  '</div>';

interface VecField {
  el: HTMLElement;
  inputs: HTMLInputElement[];
  set(v: [number, number, number] | null): void;
}

function vecField(labelKey: string, commit: (v: [number, number, number]) => void): VecField {
  const root = el('<div class="ff-sg-vec"><span></span><input class="cv-input" type="number" step="10" disabled><input class="cv-input" type="number" step="10" disabled><input class="cv-input" type="number" step="10" disabled></div>');
  label(q(root, 'span'), labelKey);
  const inputs = Array.from(root.querySelectorAll('input'));
  const wait = later();
  for (const i of inputs) {
    i.addEventListener('change', () => {
      const v = inputs.map((x) => Number(x.value));
      if (v.some((x) => !Number.isFinite(x)) || inputs.some((x) => x.value.trim() === '')) return;
      wait(() => commit([v[0], v[1], v[2]]));
    });
  }
  return {
    el: root,
    inputs,
    set(v) {
      setClass(root, 'is-off', v === null);
      inputs.forEach((x, k) => {
        setDisabled(x, v === null);
        setValue(x, v === null ? '' : String(Number(v[k].toFixed(3))));
      });
    },
  };
}

function hex(rgb: [number, number, number]): string {
  return `#${rgb.map((c) => Math.round(Math.pow(Math.max(0, Math.min(1, c)), 1 / 2.2) * 255).toString(16).padStart(2, '0')).join('')}`;
}

function rgbOf(h: string): [number, number, number] {
  const n = parseInt(h.slice(1), 16);
  const lin = (v: number): number => Math.pow(v / 255, 2.2);
  return [lin((n >> 16) & 255), lin((n >> 8) & 255), lin(n & 255)];
}

export class StageDrawer {
  private readonly drawer: HTMLElement;
  readonly toggle: HTMLButtonElement;
  private readonly list: HTMLElement;
  private readonly none: HTMLElement;
  private readonly kind;
  private readonly pos: VecField;
  private readonly dir: VecField;
  private readonly color;
  private readonly intensity;
  private readonly shadows;
  private readonly hdri: HTMLInputElement;
  private readonly status: HTMLElement;
  private readonly rotation;
  private readonly exposure;
  private readonly ground;
  private readonly bg;
  private readonly image: HTMLInputElement;
  private readonly fit;
  private readonly addKind;
  private readonly addBtn: HTMLButtonElement;
  private sel: number | null = null;
  private isOpen = false;
  private lights: LightJson[] = [];

  constructor(private readonly host: VizHost, toggle: HTMLButtonElement) {
    this.toggle = toggle;
    this.drawer = el('<aside class="ff-stage" data-k="stage" hidden></aside>');
    this.drawer.innerHTML = HTML;
    (host.pane.closest('.ff-body') ?? host.pane).appendChild(this.drawer);
    label(q(this.drawer, '[data-k="title"]'), 'Stage');
    label(q(this.drawer, '[data-k="h-lights"]'), 'Lights');
    label(q(this.drawer, '[data-k="h-props"]'), 'Light properties');
    label(q(this.drawer, '[data-k="h-env"]'), 'Environment');
    for (const c of this.drawer.querySelectorAll<HTMLElement>('.ff-fb-chev')) c.innerHTML = icon('chevronDown', 12);
    const close = q<HTMLButtonElement>(this.drawer, '[data-k="close"]');
    close.innerHTML = icon('close', 16);
    titled(close, 'Close');
    close.addEventListener('click', () => this.setOpen(false));
    toggle.addEventListener('click', (e) => { this.setOpen(!this.isOpen); if (e.detail > 0) toggle.blur(); });

    const lights = q(this.drawer, '[data-k="lights"]');
    const props = q(this.drawer, '[data-k="props"]');
    const env = q(this.drawer, '[data-k="env"]');

    this.list = el('<div class="ff-sg-list"></div>');
    this.none = el('<p class="ff-sg-empty"></p>');
    label(this.none, 'No lights, the default studio light is used');
    this.addKind = selectField('Kind', KINDS, () => undefined);
    this.addBtn = el<HTMLButtonElement>('<button class="cv-btn" type="button"></button>');
    label(this.addBtn, 'Add light');
    this.addBtn.addEventListener('click', () => void this.add());
    lights.append(this.list, this.none, this.addKind.el, this.addBtn);

    this.kind = selectField('Kind', KINDS, (v) => this.set({ kind: v }));
    this.pos = vecField('Position (mm)', (v) => this.set({ position: v }));
    this.dir = vecField('Direction', (v) => this.set({ direction: v }));
    this.color = colorField('Color', (h) => this.set({ color: rgbOf(h) }));
    this.intensity = numField('Intensity', '', '0.1', (v) => this.set({ intensity: Math.max(0, v) }));
    this.shadows = toggleField('Casts shadow', (on) => this.set({ shadows: on }));
    props.append(this.kind.el, this.pos.el, this.dir.el, this.color.el, this.intensity.el, this.shadows.el);

    const hrow = el('<div class="ff-sg-hdri"><input class="cv-input" type="text" spellcheck="false"><button class="cv-btn" type="button"></button></div>');
    this.hdri = q<HTMLInputElement>(hrow, 'input');
    this.hdri.placeholder = t('Path of an equirectangular image');
    const apply = q<HTMLButtonElement>(hrow, 'button');
    label(apply, 'Apply');
    this.status = el('<p class="ff-sg-status"></p>');
    const commitHdri = (): void => void this.stage({ hdri: this.hdri.value.trim() || null });
    apply.addEventListener('click', commitHdri);
    this.hdri.addEventListener('keydown', (e) => { if (e.key === 'Enter') commitHdri(); });
    this.rotation = numField('Rotation', 'deg', '5', (v) => void this.stage({ hdri_rotation: v }));
    this.exposure = numField('Exposure', 'EV', '0.25', (v) => void this.stage({ exposure: Math.max(-10, Math.min(10, v)) }));
    this.ground = toggleField('Ground and shadow catcher', (on) => void this.stage({ ground: on }));
    this.bg = colorField('Background', (h) => this.backdrop({ color: h }));
    const brow = el('<div class="ff-sg-hdri"><input class="cv-input" type="text" spellcheck="false"><span class="ff-sg-btns"><button class="cv-btn" type="button" data-k="pick"></button><button class="cv-btn" type="button" data-k="apply"></button><button class="cv-btn cv-btn-ghost" type="button" data-k="clear"></button></span></div>');
    this.image = q<HTMLInputElement>(brow, 'input');
    const pick = q<HTMLButtonElement>(brow, '[data-k="pick"]');
    const applyImage = q<HTMLButtonElement>(brow, '[data-k="apply"]');
    const clearImage = q<HTMLButtonElement>(brow, '[data-k="clear"]');
    label(pick, 'Choose...');
    label(applyImage, 'Apply');
    label(clearImage, 'Clear');
    setHidden(pick, !nativeDialogs());
    pick.addEventListener('click', () => void pickPath('open', null, 'image').then((p) => { if (p) { setValue(this.image, p); this.backdrop({ image: p }); } }).catch(() => undefined));
    applyImage.addEventListener('click', () => this.backdrop({ image: this.image.value.trim() || null }));
    clearImage.addEventListener('click', () => this.backdrop({ image: null }));
    this.image.addEventListener('keydown', (e) => { if (e.key === 'Enter') this.backdrop({ image: this.image.value.trim() || null }); });
    this.fit = selectField('Fit', [['cover', 'Cover'], ['contain', 'Contain']], (v) => this.backdrop({ fit: v }));
    env.append(hrow, this.status, this.rotation.el, this.exposure.el, this.ground.el, this.bg.el, brow, this.fit.el);
    onLanguage(() => { this.hdri.placeholder = t('Path of an equirectangular image'); this.image.placeholder = t('Path of a background picture'); this.refresh(); });
    this.image.placeholder = t('Path of a background picture');

    if (localStorage.getItem(OPEN_KEY) === '1') this.setOpen(true);
    this.available();
  }

  available(): void {
    const ok = hasCommand('light.add');
    setDisabled(this.addBtn, !ok);
    this.addBtn.title = ok ? '' : missing('light.add');
    this.addKind.disable(!ok);
    const st = hasCommand('stage.set');
    for (const f of [this.rotation, this.exposure]) f.disable(!st);
    setDisabled(this.hdri, !st);
    this.ground.input.disabled = !st;
    this.bg.disable(!st);
    setDisabled(this.image, !st);
    this.fit.disable(!st);
  }

  setOpen(on: boolean): void {
    if (on === this.isOpen) return;
    this.isOpen = on;
    localStorage.setItem(OPEN_KEY, on ? '1' : '0');
    setClass(this.toggle, 'is-on', on);
    if (on) {
      for (const other of document.querySelectorAll<HTMLElement>('.ff-body > aside[data-k]:not([hidden])')) {
        if (other !== this.drawer) other.querySelector<HTMLButtonElement>('[data-k="close"]')?.click();
      }
    }
    setHidden(this.drawer, !on);
    if (on) this.refresh();
  }

  private async add(): Promise<void> {
    const kind = this.addKind.select.value as LightKind;
    const first = this.lights.length === 0;
    const light = {
      kind,
      position: [1200, 2200, 1800],
      direction: [-0.4, -0.7, -0.6],
      color: [1, 0.97, 0.92],
      intensity: 2,
      shadows: first,
    };
    const r = await run<unknown>('light.add', light);
    const o = r as { id?: number; light?: { id?: number } } | undefined;
    const id = o?.id ?? o?.light?.id;
    if (typeof id === 'number') this.sel = id;
  }

  private set(patch: Record<string, unknown>): void {
    if (this.sel === null) return;
    void run('light.set', { id: this.sel, ...patch });
  }

  private async stage(patch: Record<string, unknown>): Promise<void> {
    const cur = this.host.viz().stage;
    await run('stage.set', { hdri: cur.hdri, hdri_rotation: cur.hdri_rotation, exposure: cur.exposure, ground: cur.ground, background: cur.background, ...patch });
  }

  refresh(): void {
    if (!this.isOpen) return;
    const s = this.host.viz().stage;
    this.lights = s.lights;
    if (this.sel === null || !s.lights.some((l) => l.id === this.sel)) this.sel = s.lights[0]?.id ?? null;
    setHidden(this.none, s.lights.length > 0);
    patchList(this.list, s.lights, {
      key: (l) => String(l.id),
      create: (l) => {
        const row = el(`<div class="ff-fb-item" tabindex="0" data-id="${l.id}"><span class="ff-fb-sw"></span><span class="ff-fb-name"></span><button class="cv-icon-btn" type="button">${icon('delete', 12)}</button></div>`);
        const pick = (): void => { this.sel = Number(row.dataset.id); this.refresh(); };
        row.addEventListener('click', (e) => {
          if ((e.target as HTMLElement).closest('button')) { void run('light.remove', { id: Number(row.dataset.id) }); return; }
          pick();
        });
        row.addEventListener('keydown', (e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); pick(); } });
        return row;
      },
      update: (node, l) => {
        node.dataset.id = String(l.id);
        setClass(node, 'is-sel', l.id === this.sel);
        const kindName = KINDS.find((k) => k[0] === l.kind)?.[1] ?? l.kind;
        setText(q(node, '.ff-fb-name'), `${t(kindName)} ${l.id}`);
        (q(node, '.ff-fb-sw') as HTMLElement).style.setProperty('--sw', hex(l.color));
      },
    });
    const l = s.lights.find((x) => x.id === this.sel);
    const live = hasCommand('light.set');
    this.kind.set(l?.kind ?? 'directional');
    this.kind.disable(!l || !live);
    this.pos.set(l ? l.position : null);
    this.dir.set(l ? l.direction : null);
    this.color.set(l ? hex(l.color) : '#ffffff');
    this.color.disable(!l || !live);
    this.intensity.set(l ? l.intensity : null);
    this.intensity.disable(!l || !live);
    if (l) this.shadows.set(l.shadows);
    this.shadows.input.disabled = !l || !live;
    setValue(this.hdri, s.hdri ?? '');
    this.rotation.set(s.hdri_rotation);
    this.exposure.set(s.exposure);
    this.ground.set(s.ground);
    const bo = s.background && typeof s.background === 'object' && !Array.isArray(s.background) ? (s.background as Record<string, unknown>) : {};
    const bgText = typeof s.background === 'string' ? s.background : typeof bo.color === 'string' ? bo.color : '';
    this.bg.set(/^#[0-9a-f]{6}$/i.test(bgText) ? bgText : '#1c1b1d');
    setValue(this.image, typeof bo.image === 'string' ? bo.image : '');
    this.fit.set(bo.fit === 'contain' ? 'contain' : 'cover');
  }

  private backdrop(patch: { image?: string | null; fit?: string; color?: string }): void {
    const cur = this.host.viz().stage.background;
    const o = cur && typeof cur === 'object' && !Array.isArray(cur) ? (cur as Record<string, unknown>) : {};
    const color = patch.color ?? (typeof cur === 'string' ? cur : typeof o.color === 'string' ? o.color : undefined);
    const image = patch.image !== undefined ? patch.image : typeof o.image === 'string' ? o.image : null;
    const fit = patch.fit ?? (o.fit === 'contain' ? 'contain' : 'cover');
    void this.stage({ background: image ? { image, fit, ...(color ? { color } : {}) } : color ?? null });
  }

  setHdriStatus(state: string): void {
    const text = state === 'loading' ? 'Loading the image' : state === 'ready' ? 'Image loaded' : state === 'failed' ? 'The image could not be loaded' : 'No environment image';
    setText(this.status, t(text));
  }
}
