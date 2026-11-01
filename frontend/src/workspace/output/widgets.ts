import { el, q, setClass, setStyleVar, setText, setValue } from '../../ui/dom';
import { enhanceSelect } from '../../ui/dropdown';
import { fmtNum, label, later, titled } from '../fabric/widgets';

export { fmtNum, label, later, titled };

export interface Row<T> {
  el: HTMLElement;
  set(v: T): void;
  get(): T;
  disable(on: boolean): void;
  onInput(cb: () => void): void;
}

export function numRow(labelKey: string, unit: string, step: string, min: number | null, max: number | null): Row<number> & { input: HTMLInputElement } {
  const root = el('<label class="cv-field ou-row"><span></span><input class="cv-input ou-num" type="number"><span class="ou-unit"></span></label>');
  label(q(root, 'span:first-child'), labelKey);
  setText(q(root, '.ou-unit'), unit);
  if (unit) root.classList.add('ou-unitrow');
  const input = q<HTMLInputElement>(root, 'input');
  input.step = step;
  if (min !== null) input.min = String(min);
  if (max !== null) input.max = String(max);
  return {
    el: root,
    input,
    set(v) { setValue(input, fmtNum(v)); },
    get() { const v = Number(input.value); return input.value.trim() === '' || !Number.isFinite(v) ? 0 : v; },
    disable(on) { if (input.disabled !== on) input.disabled = on; setClass(root, 'is-off', on); },
    onInput(cb) { input.addEventListener('change', cb); },
  };
}

export function pairRow(labelKey: string, step: string): Row<[number, number]> {
  const root = el('<div class="cv-field ou-row ou-pair"><span></span><input class="cv-input ou-num" type="number"><input class="cv-input ou-num" type="number"></div>');
  label(q(root, 'span:first-child'), labelKey);
  const [a, b] = Array.from(root.querySelectorAll<HTMLInputElement>('input'));
  a.step = step;
  b.step = step;
  return {
    el: root,
    set(v) { setValue(a, fmtNum(v[0])); setValue(b, fmtNum(v[1])); },
    get() { return [Number(a.value) || 0, Number(b.value) || 0]; },
    disable(on) { for (const i of [a, b]) if (i.disabled !== on) i.disabled = on; setClass(root, 'is-off', on); },
    onInput(cb) { a.addEventListener('change', cb); b.addEventListener('change', cb); },
  };
}

export function toggleRow(labelKey: string): Row<boolean> & { input: HTMLInputElement } {
  const root = el('<label class="cv-field cv-toggle-row ou-row ou-toggle"><span></span><span class="cv-toggle"><input type="checkbox"><span class="cv-toggle-track"><span class="cv-toggle-knob"></span></span></span></label>');
  label(q(root, 'span:first-child'), labelKey);
  const input = q<HTMLInputElement>(root, 'input');
  return {
    el: root,
    input,
    set(v) { if (input.checked !== v) input.checked = v; },
    get() { return input.checked; },
    disable(on) { if (input.disabled !== on) input.disabled = on; setClass(root, 'is-off', on); },
    onInput(cb) { input.addEventListener('change', cb); },
  };
}

export function selectRow(labelKey: string, options: ReadonlyArray<readonly [string, string]>): Row<string> & { select: HTMLSelectElement } {
  const root = el('<div class="cv-field ou-row ou-select"><span></span><select class="cv-input"></select></div>');
  label(q(root, 'span:first-child'), labelKey);
  const select = q<HTMLSelectElement>(root, 'select');
  for (const [value, text] of options) {
    const o = document.createElement('option');
    o.value = value;
    label(o, text);
    select.appendChild(o);
  }
  enhanceSelect(select);
  return {
    el: root,
    select,
    set(v) { if (select.value !== v) select.value = v; },
    get() { return select.value; },
    disable(on) { if (select.disabled !== on) select.disabled = on; setClass(root, 'is-off', on); },
    onInput(cb) { select.addEventListener('change', cb); },
  };
}

export function textRow(labelKey: string): Row<string> & { input: HTMLInputElement } {
  const root = el('<label class="cv-field ou-row ou-text"><span></span><input class="cv-input" type="text" spellcheck="false" autocomplete="off"></label>');
  label(q(root, 'span:first-child'), labelKey);
  const input = q<HTMLInputElement>(root, 'input');
  return {
    el: root,
    input,
    set(v) { setValue(input, v); },
    get() { return input.value; },
    disable(on) { if (input.disabled !== on) input.disabled = on; setClass(root, 'is-off', on); },
    onInput(cb) { input.addEventListener('change', cb); },
  };
}

export interface SlideRow {
  el: HTMLElement;
  range: HTMLInputElement;
  input: HTMLInputElement;
  set(v: number): void;
  disable(on: boolean): void;
}

export function slideRow(labelKey: string, min: number, max: number, step: number, onChange: (v: number) => void): SlideRow {
  const root = el('<div class="cv-field ou-row ou-slide"><span></span><div class="ou-ctl"><input type="range" disabled><input class="cv-input ou-num" type="number" disabled></div></div>');
  label(q(root, 'span:first-child'), labelKey);
  const range = q<HTMLInputElement>(root, 'input[type="range"]');
  const input = q<HTMLInputElement>(root, 'input.cv-input');
  for (const i of [range, input]) { i.min = String(min); i.max = String(max); i.step = String(step); }
  const fill = (v: number): void => setStyleVar(root, '--pct', `${Math.min(100, Math.max(0, ((v - min) / (max - min)) * 100))}%`);
  range.addEventListener('input', () => { const v = range.valueAsNumber; fill(v); input.value = fmtNum(v); });
  range.addEventListener('change', () => onChange(range.valueAsNumber));
  const wait = later();
  input.addEventListener('change', () => {
    const v = Number(input.value);
    if (input.value.trim() === '' || !Number.isFinite(v)) return;
    const c = Math.min(max, Math.max(min, v));
    range.value = String(c);
    fill(c);
    wait(() => onChange(c));
  });
  return {
    el: root,
    range,
    input,
    set(v) {
      if (document.activeElement !== range) { range.value = String(v); fill(v); }
      setValue(input, fmtNum(v));
    },
    disable(on) { for (const c of [range, input]) if (c.disabled !== on) c.disabled = on; setClass(root, 'is-off', on); },
  };
}

export function button(labelKey: string, kind: '' | 'primary' | 'ghost' = ''): HTMLButtonElement {
  const b = el<HTMLButtonElement>(`<button class="cv-btn${kind ? ` cv-btn-${kind}` : ''} ou-btn" type="button"></button>`);
  label(b, labelKey);
  return b;
}

export function bar(): { el: HTMLElement; set(p: number | null): void; show(on: boolean): void } {
  const root = el('<div class="ou-bar" hidden><i></i></div>');
  return {
    el: root,
    set(p) {
      setClass(root, 'is-indeterminate', p === null);
      setStyleVar(root, '--p', `${Math.round((p ?? 0) * 100)}%`);
    },
    show(on) { if (root.hidden === on) root.hidden = !on; },
  };
}
