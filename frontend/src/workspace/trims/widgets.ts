import { el, q, setClass, setHidden, setText, setValue } from '../../ui/dom';
import { enhanceSelect } from '../../ui/dropdown';
import { icon } from '../../ui/icons';
import { fmtNum, label } from '../fabric/widgets';

export { label, patchKeyed, repaint, titled } from '../fabric/widgets';

export interface Row { el: HTMLElement; show(on: boolean): void }

export interface NumRow extends Row {
  input: HTMLInputElement;
  set(v: number | null): void;
  disable(on: boolean): void;
  unit(text: string): void;
}

export interface PairRow extends Row {
  a: HTMLInputElement;
  b: HTMLInputElement;
  set(v: [number, number] | null): void;
  disable(on: boolean): void;
  unit(text: string): void;
}

export interface TextRow extends Row {
  input: HTMLInputElement;
  set(v: string): void;
  disable(on: boolean): void;
}

export interface SelRow extends Row {
  select: HTMLSelectElement;
  set(v: string): void;
  disable(on: boolean): void;
  options(items: ReadonlyArray<readonly [string, string]>, translate: boolean): void;
}

export interface ToggleRow extends Row {
  input: HTMLInputElement;
  set(on: boolean): void;
  disable(on: boolean): void;
}

export interface OutRow extends Row { set(text: string): void }

function shell(html: string, labelKey: string): HTMLElement {
  const root = el(html);
  label(q(root, 'span:first-child'), labelKey);
  return root;
}

const showFn = (root: HTMLElement) => (on: boolean): void => setHidden(root, !on);

export function numRow(labelKey: string, unitText: string, attrs: { min?: number; max?: number; step?: number }, commit: (v: number) => void): NumRow {
  const a = `${attrs.min !== undefined ? ` min="${attrs.min}"` : ''}${attrs.max !== undefined ? ` max="${attrs.max}"` : ''} step="${attrs.step ?? 'any'}"`;
  const root = shell(`<label class="cv-field ff-tr-row"><span class="ff-tr-label"></span><input class="cv-input ff-tr-num" type="number"${a} disabled><span class="ff-tr-unit"></span></label>`, labelKey);
  const input = q<HTMLInputElement>(root, 'input');
  const u = q(root, '.ff-tr-unit');
  setText(u, unitText);
  input.addEventListener('change', () => {
    const v = Number(input.value);
    if (input.value.trim() === '' || !Number.isFinite(v)) return;
    commit(v);
  });
  return {
    el: root, input, show: showFn(root),
    set(v) { setValue(input, v === null ? '' : fmtNum(v)); },
    disable(on) { if (input.disabled !== on) input.disabled = on; setClass(root, 'is-off', on); },
    unit(text) { setText(u, text); },
  };
}

export function pairRow(labelKey: string, unitText: string, commit: (a: number, b: number) => void): PairRow {
  const root = shell(`<div class="cv-field ff-tr-row ff-tr-pair"><span class="ff-tr-label"></span><input class="cv-input ff-tr-num" type="number" step="any" disabled><input class="cv-input ff-tr-num" type="number" step="any" disabled><span class="ff-tr-unit"></span></div>`, labelKey);
  const [a, b] = Array.from(root.querySelectorAll<HTMLInputElement>('input'));
  const u = q(root, '.ff-tr-unit');
  setText(u, unitText);
  const go = (): void => {
    const x = Number(a.value);
    const y = Number(b.value);
    if (a.value.trim() === '' || b.value.trim() === '' || !Number.isFinite(x) || !Number.isFinite(y)) return;
    commit(x, y);
  };
  a.addEventListener('change', go);
  b.addEventListener('change', go);
  return {
    el: root, a, b, show: showFn(root),
    set(v) { setValue(a, v ? fmtNum(v[0]) : ''); setValue(b, v ? fmtNum(v[1]) : ''); },
    disable(on) { for (const i of [a, b]) if (i.disabled !== on) i.disabled = on; setClass(root, 'is-off', on); },
    unit(text) { setText(u, text); },
  };
}

export function textRow(labelKey: string, commit: (v: string) => void): TextRow {
  const root = shell('<label class="cv-field ff-tr-row ff-tr-wide"><span class="ff-tr-label"></span><input class="cv-input" type="text" spellcheck="false" autocomplete="off" disabled></label>', labelKey);
  const input = q<HTMLInputElement>(root, 'input');
  input.addEventListener('change', () => commit(input.value.trim()));
  input.addEventListener('keydown', (e) => { if (e.key === 'Enter') input.blur(); });
  return {
    el: root, input, show: showFn(root),
    set(v) { setValue(input, v); },
    disable(on) { if (input.disabled !== on) input.disabled = on; setClass(root, 'is-off', on); },
  };
}

export function selRow(labelKey: string, items: ReadonlyArray<readonly [string, string]>, translate: boolean, commit: (v: string) => void): SelRow {
  const root = shell('<div class="cv-field ff-tr-row ff-tr-wide"><span class="ff-tr-label"></span><select class="cv-input" disabled></select></div>', labelKey);
  const select = q<HTMLSelectElement>(root, 'select');
  const row: SelRow = {
    el: root, select, show: showFn(root),
    set(v) { if (select.value !== v && document.activeElement !== select) select.value = v; },
    disable(on) { if (select.disabled !== on) select.disabled = on; setClass(root, 'is-off', on); },
    options(next, tr) {
      const keep = select.value;
      next.forEach(([value, text], i) => {
        let o = select.options[i];
        if (!o) { o = new Option('', value); select.add(o); }
        if (tr && o.dataset.key !== text) { o.dataset.key = text; label(o, text); }
        if (o.value !== value) o.value = value;
        if (!tr && o.textContent !== text) o.textContent = text;
      });
      while (select.options.length > next.length) select.remove(select.options.length - 1);
      if (next.some(([v]) => v === keep) && select.value !== keep && document.activeElement !== select) select.value = keep;
    },
  };
  row.options(items, translate);
  select.addEventListener('change', () => commit(select.value));
  enhanceSelect(select);
  return row;
}

export function toggleRow(labelKey: string, commit: (on: boolean) => void): ToggleRow {
  const root = shell('<label class="cv-field cv-toggle-row ff-tr-row ff-tr-sw"><span class="ff-tr-label"></span><span class="cv-toggle"><input type="checkbox" disabled><span class="cv-toggle-track"><span class="cv-toggle-knob"></span></span></span></label>', labelKey);
  const input = q<HTMLInputElement>(root, 'input');
  input.addEventListener('change', () => commit(input.checked));
  return {
    el: root, input, show: showFn(root),
    set(on) { if (input.checked !== on) input.checked = on; },
    disable(on) { if (input.disabled !== on) input.disabled = on; setClass(root, 'is-off', on); },
  };
}

export function outRow(labelKey: string): OutRow {
  const root = shell('<div class="cv-field ff-tr-row ff-tr-wide"><span class="ff-tr-label"></span><output class="ff-tr-out"></output></div>', labelKey);
  const out = q(root, 'output');
  return { el: root, show: showFn(root), set(text) { setText(out, text); } };
}

export interface Action { el: HTMLButtonElement; set(disabled: boolean): void; show(on: boolean): void }

export function action(labelKey: string, iconName: string | null, onClick: () => void): Action {
  const b = el<HTMLButtonElement>(`<button class="ff-btn ff-tr-btn" type="button">${iconName ? icon(iconName, 14) : ''}<span></span></button>`);
  label(q(b, 'span:last-child'), labelKey);
  b.addEventListener('click', () => { onClick(); b.blur(); });
  return {
    el: b,
    set(disabled) { if (b.disabled !== disabled) b.disabled = disabled; },
    show(on) { setHidden(b, !on); },
  };
}
