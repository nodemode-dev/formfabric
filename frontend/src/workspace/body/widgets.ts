import { el, q, setClass, setValue } from '../../ui/dom';
import { enhanceSelect } from '../../ui/dropdown';
import { fmtNum, label } from '../fabric/widgets';

export { flagBad, fmtNum, label, patchKeyed, repaint, titled, toggleField, type ToggleField } from '../fabric/widgets';

export interface SlideRow {
  el: HTMLElement;
  range: HTMLInputElement;
  input: HTMLInputElement;
  set(v: number | null): void;
  disable(on: boolean): void;
}

export function slideRow(labelKey: string, min: number, max: number, step: number, commit: (v: number) => void): SlideRow {
  const root = el(`<div class="cv-field ff-bd-row"><span></span><div class="ff-bd-ctl"><input type="range" min="${min}" max="${max}" step="${step}" disabled><input class="cv-input ff-bd-num" type="number" min="${min}" max="${max}" step="${step}" disabled></div></div>`);
  label(q(root, 'span:first-child'), labelKey);
  const range = q<HTMLInputElement>(root, 'input[type="range"]');
  const input = q<HTMLInputElement>(root, 'input.cv-input');
  range.addEventListener('input', () => { input.value = fmtNum(range.valueAsNumber); });
  range.addEventListener('change', () => { commit(range.valueAsNumber); });
  input.addEventListener('change', () => {
    const v = Number(input.value);
    if (input.value.trim() === '' || !Number.isFinite(v)) return;
    range.value = String(v);
    commit(v);
  });
  return {
    el: root,
    range,
    input,
    set(v) {
      if (v === null) { setValue(input, ''); return; }
      if (document.activeElement !== range && range.valueAsNumber !== v) range.value = String(v);
      setValue(input, fmtNum(v));
    },
    disable(on) { for (const c of [range, input]) if (c.disabled !== on) c.disabled = on; setClass(root, 'is-off', on); },
  };
}

export interface SelectRow {
  el: HTMLElement;
  select: HTMLSelectElement;
  set(v: string): void;
  disable(on: boolean): void;
  options(items: ReadonlyArray<readonly [string, string]>, translate: boolean): void;
}

export function selectRow(labelKey: string, items: ReadonlyArray<readonly [string, string]>, translate: boolean, commit: (v: string) => void): SelectRow {
  const root = el('<div class="cv-field ff-bd-row ff-bd-sel"><span></span><select class="cv-input"></select></div>');
  label(q(root, 'span:first-child'), labelKey);
  const select = q<HTMLSelectElement>(root, 'select');
  const row: SelectRow = {
    el: root,
    select,
    set(v) { if (select.value !== v) select.value = v; },
    disable(on) { if (select.disabled !== on) select.disabled = on; setClass(root, 'is-off', on); },
    options(next, tr) {
      const have = Array.from(select.options);
      const same = have.length === next.length && have.every((o, i) => o.value === next[i][0]);
      if (!same) {
        const keep = select.value;
        select.replaceChildren(...next.map(([value]) => { const o = document.createElement('option'); o.value = value; return o; }));
        if (next.some(([v]) => v === keep)) select.value = keep;
      }
      next.forEach(([, text], i) => {
        const o = select.options[i];
        if (tr) label(o, text); else if (o.textContent !== text) o.textContent = text;
      });
    },
  };
  row.options(items, translate);
  select.addEventListener('change', () => commit(select.value));
  enhanceSelect(select);
  return row;
}
