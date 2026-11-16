import { setChecked, setDisabled, setText, setValue } from '../../ui/dom';
import { onLanguage, t } from '../../ui/i18n';

const CSS = `
.ffd-opts{display:flex;align-items:center;gap:6px;height:24px;box-sizing:border-box;padding:0 6px;pointer-events:auto;border:1px solid var(--border-color);border-radius:var(--radius-sm);background:color-mix(in srgb,var(--bg-secondary) 88%,transparent);color:var(--text-secondary);font:500 var(--fs-11)/1 var(--font-ui)}
.ffd-opts[hidden],.ffd-pop[hidden],.ffd-strip[hidden],.ffd-sec-row[hidden]{display:none}
.ffd-f{display:inline-flex;align-items:center;gap:4px}
.ffd-f input[type=number],.ffd-f select,.ffd-pop input[type=number],.ffd-pop select,.ffd-strip input[type=number]{height:18px;box-sizing:border-box;min-width:0;padding:0 3px;border:1px solid var(--border-color);border-radius:var(--radius-sm);background:var(--bg-primary);color:var(--text-primary);font:500 var(--fs-11)/1 var(--font-ui)}
.ffd-f input[type=number]{width:46px}
.ffd-pop{position:absolute;left:8px;top:40px;z-index:var(--z-raised);width:264px;max-height:calc(100% - 56px);overflow:auto;box-sizing:border-box;padding:6px;border:1px solid var(--border-color);border-radius:var(--radius-sm);background:color-mix(in srgb,var(--bg-secondary) 96%,transparent);color:var(--text-secondary);font:500 var(--fs-11)/1.3 var(--font-ui)}
.ffd-pop details{border-bottom:1px solid var(--border-color)}
.ffd-pop details:last-child{border-bottom:0}
.ffd-pop summary{cursor:pointer;padding:5px 2px;color:var(--text-primary);font-weight:600}
.ffd-sec{display:grid;gap:5px;padding:2px 2px 8px}
.ffd-sec-row{display:grid;grid-template-columns:78px minmax(0,1fr) 44px;align-items:center;gap:6px}
.ffd-sec-row.is-2{grid-template-columns:78px minmax(0,1fr)}
.ffd-sec>.ffd-f{justify-content:space-between}
.ffd-sec>.ffd-f select{flex:1;max-width:150px}
.ffd-xyz input{width:100%}
.ffd-sec-row .ffd-xyz{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:3px}
.ffd-sec-row input[type=range],.ffd-strip input[type=range]{width:100%;min-width:0;margin:0;accent-color:var(--accent-color)}
.ffd-val{text-align:right;color:var(--text-primary);font-variant-numeric:tabular-nums;white-space:nowrap}
.ffd-act{height:22px;padding:0 8px;border:1px solid var(--border-color);border-radius:var(--radius-sm);background:transparent;color:var(--text-secondary);font:500 var(--fs-11)/1 var(--font-ui);cursor:pointer}
.ffd-act:hover{background:var(--bg-hover);color:var(--text-primary)}
.ffd-act.is-on{background:var(--accent-color-dim);color:var(--accent-color)}
.ffd-act:disabled{opacity:.4;cursor:default;pointer-events:none}
.ffd-meter{position:relative;height:4px;overflow:hidden;border-radius:2px;background:color-mix(in srgb,var(--accent-color) 22%,transparent)}
.ffd-meter i{position:absolute;left:0;top:0;bottom:0;width:calc(var(--p,0) * 100%);background:var(--accent-color)}
.ffd-meter.is-indet i{width:30%;animation:ffd-slide 1.1s linear infinite}
@keyframes ffd-slide{from{transform:translateX(-100%)}to{transform:translateX(340%)}}
.ffd-strip{position:absolute;right:8px;bottom:calc(34px + var(--ff-an-lift, 0px));z-index:var(--z-raised);display:flex;flex-wrap:wrap;align-items:center;gap:6px;max-width:calc(100% - 200px);box-sizing:border-box;padding:4px 6px;border:1px solid var(--border-color);border-radius:var(--radius-sm);background:color-mix(in srgb,var(--bg-secondary) 92%,transparent);color:var(--text-secondary);font:500 var(--fs-11)/1 var(--font-ui)}
.ffd-strip .ffd-f input[type=range]{width:84px}
.ffd-strip .ffd-val{min-width:34px}
`;

export function ensureStyle(): void {
  if (document.getElementById('ffd-ui-style')) return;
  const style = document.createElement('style');
  style.id = 'ffd-ui-style';
  style.textContent = CSS;
  document.head.appendChild(style);
}

export function labelled(node: HTMLElement, english: string): void {
  setText(node, t(english));
  onLanguage(() => setText(node, t(english)));
}

export function button(english: string, onClick: () => void, cls = 'ffd-act'): HTMLButtonElement {
  const b = document.createElement('button');
  b.type = 'button';
  b.className = cls;
  labelled(b, english);
  b.addEventListener('click', (e) => { onClick(); if (e.detail > 0) b.blur(); });
  return b;
}

export interface Field<T> {
  root: HTMLElement;
  set(v: T): void;
  focused(): boolean;
  disable(on: boolean): void;
}

export function numberField(english: string, min: number, max: number, step: number, value: number, onChange: (v: number) => void): Field<number> {
  const root = document.createElement('label');
  root.className = 'ffd-f';
  const name = document.createElement('span');
  labelled(name, english);
  const input = document.createElement('input');
  input.type = 'number';
  input.min = String(min);
  input.max = String(max);
  input.step = String(step);
  input.value = String(value);
  input.addEventListener('change', () => {
    const v = Number(input.value);
    if (!Number.isFinite(v)) { input.value = String(value); return; }
    const c = Math.min(max, Math.max(min, v));
    value = c;
    input.value = String(c);
    onChange(c);
  });
  root.append(name, input);
  return {
    root,
    set: (v) => { value = v; setValue(input, String(v)); },
    focused: () => document.activeElement === input,
    disable: (on) => setDisabled(input, on),
  };
}

export function selectField(english: string, options: Array<[string, string]>, value: string, onChange: (v: string) => void): Field<string> & { select: HTMLSelectElement; option(v: string): HTMLOptionElement | undefined } {
  const root = document.createElement('label');
  root.className = 'ffd-f';
  const name = document.createElement('span');
  labelled(name, english);
  const select = document.createElement('select');
  for (const [v, label] of options) {
    const o = document.createElement('option');
    o.value = v;
    labelled(o, label);
    select.appendChild(o);
  }
  select.value = value;
  select.addEventListener('change', () => { onChange(select.value); select.blur(); });
  root.append(name, select);
  return {
    root,
    select,
    option: (v) => Array.from(select.options).find((o) => o.value === v),
    set: (v) => setValue(select, v),
    focused: () => document.activeElement === select,
    disable: (on) => setDisabled(select, on),
  };
}

export interface SliderOpts {
  english: string;
  min: number;
  max: number;
  step: number;
  value: number;
  fmt(v: number): string;
  commit(v: number): void;
  toValue?(raw: number): number;
  toRaw?(v: number): number;
  row?: boolean;
}

export function sliderField(o: SliderOpts): Field<number> {
  const to = o.toValue ?? ((r: number): number => r);
  const from = o.toRaw ?? ((v: number): number => v);
  const root = document.createElement(o.row ? 'div' : 'label');
  root.className = o.row ? 'ffd-sec-row' : 'ffd-f';
  const name = document.createElement('span');
  labelled(name, o.english);
  const input = document.createElement('input');
  input.type = 'range';
  input.min = String(o.min);
  input.max = String(o.max);
  input.step = String(o.step);
  input.value = String(from(o.value));
  const val = document.createElement('span');
  val.className = 'ffd-val';
  setText(val, o.fmt(o.value));
  input.addEventListener('input', () => setText(val, o.fmt(to(Number(input.value)))));
  input.addEventListener('change', () => { o.commit(to(Number(input.value))); input.blur(); });
  root.append(name, input, val);
  return {
    root,
    set: (v) => {
      setText(val, o.fmt(v));
      if (document.activeElement !== input) setValue(input, String(from(v)));
    },
    focused: () => document.activeElement === input,
    disable: (on) => setDisabled(input, on),
  };
}

export function toggleField(english: string, onChange: (on: boolean) => void): Field<boolean> & { input: HTMLInputElement } {
  const root = document.createElement('label');
  root.className = 'ffd-f';
  const input = document.createElement('input');
  input.type = 'checkbox';
  const name = document.createElement('span');
  labelled(name, english);
  input.addEventListener('change', () => { onChange(input.checked); input.blur(); });
  root.append(input, name);
  return {
    root,
    input,
    set: (v) => setChecked(input, v),
    focused: () => document.activeElement === input,
    disable: (on) => setDisabled(input, on),
  };
}

export function row(english: string, ...nodes: HTMLElement[]): HTMLElement {
  const r = document.createElement('div');
  r.className = 'ffd-sec-row is-2';
  const name = document.createElement('span');
  labelled(name, english);
  r.append(name, ...nodes);
  return r;
}
