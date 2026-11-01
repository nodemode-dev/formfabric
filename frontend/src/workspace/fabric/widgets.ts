import { el, patchList, q, setClass, setText, setValue, type PatchListOptions } from '../../ui/dom';
import { enhanceSelect } from '../../ui/dropdown';
import { onLanguage, t } from '../../ui/i18n';

const repaints = new Set<() => void>();
onLanguage(() => { for (const f of repaints) f(); });

export function repaint(fn: () => void): void {
  repaints.add(fn);
  fn();
}

export function label(node: HTMLElement, key: string): void {
  repaint(() => setText(node, t(key)));
}

export function titled(node: HTMLElement, key: string): void {
  repaint(() => { const s = t(key); if (node.title !== s) node.title = s; node.setAttribute('aria-label', s); });
}

export const fmtNum = (v: number): string => String(Number(v.toPrecision(6)));

export function patchKeyed<T>(container: HTMLElement, items: T[], opts: PatchListOptions<T>): void {
  patchList(container, items, {
    key: opts.key,
    create: (item) => { const n = opts.create(item); opts.update(n, item); return n; },
    update: opts.update,
  });
}

const badTimers = new WeakMap<HTMLElement, number>();

export function flagBad(input: HTMLInputElement, message: string, restore: () => void): void {
  setClass(input, 'is-bad', true);
  input.title = message;
  window.clearTimeout(badTimers.get(input));
  badTimers.set(input, window.setTimeout(() => {
    setClass(input, 'is-bad', false);
    input.removeAttribute('title');
    restore();
  }, 2600));
}

export function later(delay = 180): (fn: () => void) => void {
  let timer = 0;
  return (fn) => {
    window.clearTimeout(timer);
    timer = window.setTimeout(fn, delay);
  };
}

export interface NumField {
  el: HTMLElement;
  input: HTMLInputElement;
  set(v: number | null): void;
  disable(on: boolean): void;
}

export function numField(labelKey: string, unit: string, step: string, commit: (v: number, input: HTMLInputElement) => void): NumField {
  const root = el(`<label class="cv-field ff-fb-row"><span></span><input class="cv-input ff-fb-num" type="number" step="${step}" disabled><span class="ff-fb-unit"></span></label>`);
  label(q(root, 'span:first-child'), labelKey);
  setText(q(root, '.ff-fb-unit'), unit);
  const input = q<HTMLInputElement>(root, 'input');
  const wait = later();
  input.addEventListener('change', () => {
    const v = Number(input.value);
    if (input.value.trim() === '' || !Number.isFinite(v)) return;
    wait(() => commit(v, input));
  });
  return {
    el: root,
    input,
    set(v) { if (!input.classList.contains('is-bad')) setValue(input, v === null ? '' : fmtNum(v)); },
    disable(on) { if (input.disabled !== on) input.disabled = on; setClass(root, 'is-off', on); },
  };
}

export interface PairField {
  el: HTMLElement;
  a: HTMLInputElement;
  b: HTMLInputElement;
  set(v: [number, number] | null): void;
  disable(on: boolean): void;
}

export function pairField(labelKey: string, unit: string, step: string, commit: (v: [number, number], input: HTMLInputElement) => void): PairField {
  const root = el(`<div class="cv-field ff-fb-row ff-fb-pair"><span></span><input class="cv-input ff-fb-num" type="number" step="${step}" disabled><input class="cv-input ff-fb-num" type="number" step="${step}" disabled><span class="ff-fb-unit"></span></div>`);
  label(q(root, 'span:first-child'), labelKey);
  setText(q(root, '.ff-fb-unit'), unit);
  const [a, b] = Array.from(root.querySelectorAll<HTMLInputElement>('input'));
  const wait = later();
  for (const input of [a, b]) {
    input.addEventListener('change', () => {
      const x = Number(a.value);
      const y = Number(b.value);
      if (!Number.isFinite(x) || !Number.isFinite(y) || a.value.trim() === '' || b.value.trim() === '') return;
      wait(() => commit([x, y], input));
    });
  }
  return {
    el: root,
    a,
    b,
    set(v) {
      for (const [input, n] of [[a, v?.[0]], [b, v?.[1]]] as const) if (!input.classList.contains('is-bad')) setValue(input, n === undefined ? '' : fmtNum(n));
    },
    disable(on) { for (const input of [a, b]) if (input.disabled !== on) input.disabled = on; setClass(root, 'is-off', on); },
  };
}

export interface UnitField {
  el: HTMLElement;
  range: HTMLInputElement;
  input: HTMLInputElement;
  set(v: number | null): void;
  disable(on: boolean): void;
}

export function unitField(labelKey: string, commit: (v: number, input: HTMLInputElement) => void): UnitField {
  const root = el('<div class="cv-field ff-fb-row ff-fb-slide"><span></span><div class="ff-fb-ctl"><input type="range" min="0" max="1" step="0.01" disabled><input class="cv-input ff-fb-num" type="number" step="0.05" disabled></div></div>');
  label(q(root, 'span:first-child'), labelKey);
  const range = q<HTMLInputElement>(root, 'input[type="range"]');
  const input = q<HTMLInputElement>(root, 'input.cv-input');
  const fill = (v: number): void => { root.style.setProperty('--pct', `${Math.min(100, Math.max(0, v * 100))}%`); };
  const wait = later();
  range.addEventListener('input', () => { const v = range.valueAsNumber; fill(v); input.value = fmtNum(v); });
  range.addEventListener('change', () => { const v = range.valueAsNumber; commit(v, input); });
  input.addEventListener('change', () => {
    const v = Number(input.value);
    if (input.value.trim() === '' || !Number.isFinite(v)) return;
    if (v >= 0 && v <= 1) { range.value = String(v); fill(v); }
    wait(() => commit(v, input));
  });
  return {
    el: root,
    range,
    input,
    set(v) {
      if (v === null) { setValue(input, ''); return; }
      if (document.activeElement !== range) { range.value = String(v); fill(v); }
      if (!input.classList.contains('is-bad')) setValue(input, fmtNum(v));
    },
    disable(on) { for (const c of [range, input]) if (c.disabled !== on) c.disabled = on; setClass(root, 'is-off', on); },
  };
}

export interface ColorField {
  el: HTMLElement;
  input: HTMLInputElement;
  name: HTMLElement;
  set(hex: string): void;
  disable(on: boolean): void;
}

export function colorField(labelKey: string, commit: (hex: string) => void): ColorField {
  const root = el('<label class="cv-field ff-fb-row ff-fb-color"><span></span><input class="cv-input cv-color" type="color" disabled><output class="ff-fb-hex"></output></label>');
  const name = q(root, 'span:first-child');
  label(name, labelKey);
  const input = q<HTMLInputElement>(root, 'input');
  const hex = q(root, 'output');
  input.addEventListener('input', () => setText(hex, input.value));
  input.addEventListener('change', () => commit(input.value));
  return {
    el: root,
    input,
    name,
    set(v) { setValue(input, v); setText(hex, v); },
    disable(on) { if (input.disabled !== on) input.disabled = on; setClass(root, 'is-off', on); },
  };
}

export interface ToggleField {
  el: HTMLElement;
  input: HTMLInputElement;
  set(on: boolean): void;
}

export function toggleField(labelKey: string, commit: (on: boolean) => void): ToggleField {
  const root = el('<label class="cv-field cv-toggle-row ff-fb-row"><span></span><span class="cv-toggle"><input type="checkbox"><span class="cv-toggle-track"><span class="cv-toggle-knob"></span></span></span></label>');
  label(q(root, 'span:first-child'), labelKey);
  const input = q<HTMLInputElement>(root, 'input');
  input.addEventListener('change', () => commit(input.checked));
  return { el: root, input, set(on) { if (input.checked !== on) input.checked = on; } };
}

export interface SelectField {
  el: HTMLElement;
  select: HTMLSelectElement;
  set(v: string): void;
  disable(on: boolean): void;
}

export function selectField(labelKey: string, options: ReadonlyArray<readonly [string, string]>, commit: (v: string) => void): SelectField {
  const root = el('<div class="cv-field ff-fb-row ff-fb-select"><span></span><select class="cv-input"></select></div>');
  label(q(root, 'span:first-child'), labelKey);
  const select = q<HTMLSelectElement>(root, 'select');
  for (const [value, text] of options) {
    const o = document.createElement('option');
    o.value = value;
    label(o, text);
    select.appendChild(o);
  }
  select.addEventListener('change', () => commit(select.value));
  enhanceSelect(select);
  return {
    el: root,
    select,
    set(v) { if (select.value !== v) select.value = v; },
    disable(on) { if (select.disabled !== on) select.disabled = on; setClass(root, 'is-off', on); },
  };
}
