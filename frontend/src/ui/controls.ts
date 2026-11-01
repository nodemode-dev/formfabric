import { el, q, setChecked, setText, setValue } from './dom';
import { enhanceSelect } from './dropdown';

export interface Row<T> { el: HTMLElement; set(v: T): void; disable(on: boolean): void }

export const fmt = (v: number): string => String(Number(v.toFixed(3)));
const clamp = (v: number, lo: number, hi: number): number => Math.min(hi, Math.max(lo, v));

function lock(root: HTMLElement, on: boolean): void {
  root.classList.toggle('is-off', on);
  for (const c of Array.from(root.querySelectorAll<HTMLInputElement | HTMLSelectElement | HTMLButtonElement>('input, select, button'))) c.disabled = on;
}

export function sliderRow(label: string, min: number, max: number, step: number, onInput: (v: number) => void, def?: number): Row<number> {
  const root = el(`<div class="cv-field cv-slider"><span></span><div class="cv-slider-ctl"><input type="range" min="${min}" max="${max}" step="${step}"><input class="cv-num" type="number" min="${min}" max="${max}" step="${step}"></div></div>`);
  setText(q(root, 'span'), label);
  const range = q<HTMLInputElement>(root, 'input[type="range"]');
  const box = q<HTMLInputElement>(root, 'input.cv-num');
  range.setAttribute('aria-label', label);
  box.setAttribute('aria-label', `${label} value`);
  const fill = (): void => { const span = max - min; range.style.setProperty('--pct', `${span > 0 ? clamp(((range.valueAsNumber - min) / span) * 100, 0, 100) : 0}%`); };
  range.addEventListener('input', () => { fill(); const v = range.valueAsNumber; box.value = fmt(v); onInput(v); });
  box.addEventListener('change', () => {
    const raw = box.valueAsNumber;
    if (!Number.isFinite(raw)) return;
    const v = clamp(raw, min, max);
    box.value = fmt(v);
    range.value = String(v);
    fill();
    onInput(v);
  });
  box.addEventListener('keydown', (e) => { if (e.key === 'Enter') { e.preventDefault(); box.blur(); } });
  if (def !== undefined) {
    const reset = (): void => { range.value = String(def); fill(); box.value = fmt(def); onInput(def); };
    q(root, 'span').addEventListener('dblclick', reset);
    range.addEventListener('dblclick', reset);
    q(root, 'span').title = 'Double-click to reset';
  }
  fill();
  return { el: root, set(v) { range.value = String(v); fill(); setValue(box, fmt(v)); }, disable(on) { lock(root, on); } };
}

export function selectRow(label: string, options: ReadonlyArray<readonly [string, string]>, onChange: (v: string) => void): Row<string> {
  const root = el(`<label class="cv-field"><span></span><select class="cv-input">${options.map(([v, t]) => `<option value="${v}">${t}</option>`).join('')}</select></label>`);
  setText(q(root, 'span'), label);
  const sel = q<HTMLSelectElement>(root, 'select');
  sel.setAttribute('aria-label', label);
  sel.addEventListener('change', () => onChange(sel.value));
  enhanceSelect(sel);
  return { el: root, set(v) { if (sel.value !== v) sel.value = v; }, disable(on) { lock(root, on); } };
}

const TOGGLE_TRACK = '<span class="cv-toggle-track"><span class="cv-toggle-knob"></span></span>';

export function toggleRow(label: string, onChange: (v: boolean) => void): Row<boolean> {
  const root = el(`<label class="cv-field cv-toggle-row"><span></span><span class="cv-toggle"><input type="checkbox">${TOGGLE_TRACK}</span></label>`);
  setText(q(root, 'span'), label);
  const box = q<HTMLInputElement>(root, 'input');
  box.setAttribute('aria-label', label);
  box.addEventListener('change', () => onChange(box.checked));
  return { el: root, set(v) { setChecked(box, v); }, disable(on) { lock(root, on); } };
}

export function numberRow(label: string, min: number, max: number, step: number, onChange: (v: number) => void): Row<number> {
  const root = el(`<label class="cv-field"><span></span><input class="cv-input" type="number" min="${min}" max="${max}" step="${step}"></label>`);
  setText(q(root, 'span'), label);
  const input = q<HTMLInputElement>(root, 'input');
  input.setAttribute('aria-label', label);
  input.addEventListener('change', () => { const v = input.valueAsNumber; if (Number.isFinite(v)) onChange(clamp(v, min, max)); });
  return { el: root, set(v) { setValue(input, fmt(v)); }, disable(on) { lock(root, on); } };
}

export function colorRow(label: string, onChange: (hex: string) => void): Row<string> {
  const root = el('<label class="cv-field"><span></span><input class="cv-input cv-color" type="color"></label>');
  setText(q(root, 'span'), label);
  const inp = q<HTMLInputElement>(root, 'input');
  inp.setAttribute('aria-label', label);
  inp.addEventListener('input', () => onChange(inp.value));
  return { el: root, set(v) { setValue(inp, v); }, disable(on) { lock(root, on); } };
}

export function checkOf(label: string, get: () => boolean, onChange: (v: boolean) => void): Row<boolean> {
  const row = toggleRow(label, onChange);
  row.set(get());
  return row;
}
