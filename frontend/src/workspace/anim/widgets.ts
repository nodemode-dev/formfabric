import { el, q, setClass, setText, setValue } from '../../ui/dom';
import { enhanceSelect } from '../../ui/dropdown';
import { fmtNum, label, patchKeyed, repaint, titled } from '../fabric/widgets';

export { fmtNum, label, patchKeyed, repaint, titled };

function caption(node: HTMLElement, text: string, translate: boolean): void {
  if (translate) label(node, text); else setText(node, text);
}

export interface Field<T> {
  el: HTMLElement;
  input: HTMLInputElement;
  get(): T;
  set(v: T): void;
  disable(on: boolean): void;
}

export function textRow(text: string, translate: boolean): Field<string> {
  const root = el('<label class="cv-field ff-an-row"><span></span><input class="cv-input" type="text" spellcheck="false" autocomplete="off"></label>');
  caption(q(root, 'span'), text, translate);
  const input = q<HTMLInputElement>(root, 'input');
  return {
    el: root,
    input,
    get: () => input.value,
    set: (v) => setValue(input, v),
    disable(on) { if (input.disabled !== on) input.disabled = on; setClass(root, 'is-off', on); },
  };
}

export function numRow(text: string, translate: boolean, min: number, max: number, step: number, value: number): Field<number> {
  const root = el(`<label class="cv-field ff-an-row"><span></span><input class="cv-input" type="number" min="${min}" max="${max}" step="${step}" value="${value}"></label>`);
  caption(q(root, 'span'), text, translate);
  const input = q<HTMLInputElement>(root, 'input');
  return {
    el: root,
    input,
    get: () => { const v = Number(input.value); return Number.isFinite(v) ? Math.min(max, Math.max(min, v)) : value; },
    set: (v) => setValue(input, fmtNum(v)),
    disable(on) { if (input.disabled !== on) input.disabled = on; setClass(root, 'is-off', on); },
  };
}

export function checkRow(text: string, translate: boolean, value: boolean): Field<boolean> {
  const root = el('<label class="cv-field ff-an-row ff-an-check"><span></span><input type="checkbox"></label>');
  caption(q(root, 'span'), text, translate);
  const input = q<HTMLInputElement>(root, 'input');
  input.checked = value;
  return {
    el: root,
    input,
    get: () => input.checked,
    set: (v) => { if (input.checked !== v) input.checked = v; },
    disable(on) { if (input.disabled !== on) input.disabled = on; setClass(root, 'is-off', on); },
  };
}

export interface SelRow {
  el: HTMLElement;
  select: HTMLSelectElement;
  get(): string;
  options(items: ReadonlyArray<readonly [string, string]>, translate: boolean): void;
  disable(on: boolean): void;
}

export function selRow(text: string, translate: boolean, items: ReadonlyArray<readonly [string, string]>, translateItems: boolean): SelRow {
  const root = el('<div class="cv-field ff-an-row"><span></span><select class="cv-input"></select></div>');
  caption(q(root, 'span'), text, translate);
  const select = q<HTMLSelectElement>(root, 'select');
  const row: SelRow = {
    el: root,
    select,
    get: () => select.value,
    disable(on) { if (select.disabled !== on) select.disabled = on; setClass(root, 'is-off', on); },
    options(next, tr) {
      const have = Array.from(select.options);
      const same = have.length === next.length && have.every((o, i) => o.value === next[i][0]);
      if (!same) {
        const keep = select.value;
        select.replaceChildren(...next.map(([value]) => { const o = document.createElement('option'); o.value = value; return o; }));
        if (next.some(([v]) => v === keep)) select.value = keep;
      }
      next.forEach(([, label2], i) => caption(select.options[i], label2, tr));
    },
  };
  row.options(items, translateItems);
  enhanceSelect(select);
  return row;
}
