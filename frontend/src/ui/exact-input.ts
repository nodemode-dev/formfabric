import type { JsonSchema } from '../engine/service';
import { Dialog } from './dialogs';
import { el, q, setText } from './dom';
import { t } from './i18n';
import { evalExpr } from './number-fields';
import { loadPrefs } from './prefs';

const UNIT_MM: Record<string, number> = { mm: 1, cm: 10, in: 25.4 };

type Values = Record<string, unknown>;
type Read = () => unknown;

const typeOf = (s: JsonSchema): string => (Array.isArray(s.type) ? s.type[0] : s.type) ?? 'string';
const label = (name: string): string => name.replace(/_/g, ' ').replace(/^./, (c) => c.toUpperCase());

export class ExactInputDialog extends Dialog {
  private readonly form = el('<div class="cv-exact"></div>');
  private readers: Array<{ name: string; required: boolean; read: Read; box: HTMLElement }> = [];
  private resolve: ((v: Values | null) => void) | null = null;
  private readonly error = el('<p class="cv-note cv-exact-error" role="alert" hidden></p>');
  private readonly ok: HTMLButtonElement;

  constructor() {
    super('Exact Input', 460);
    this.body.append(this.form, this.error);
    this.button('Cancel', 'ghost', () => this.finish(null));
    this.ok = this.button('OK', 'primary', () => this.submit());
    this.onClose = () => this.finish(null);
    this.scrim.addEventListener('keydown', (e) => {
      if (e.key === 'Enter' && !(e.target instanceof HTMLTextAreaElement) && !(e.target instanceof HTMLButtonElement)) { e.preventDefault(); this.submit(); }
    });
  }

  ask(title: string, schema: JsonSchema, initial: Values = {}): Promise<Values | null> {
    const pending = this.resolve;
    this.resolve = null;
    pending?.(null);
    this.setTitle(title);
    this.form.replaceChildren();
    this.readers = [];
    this.error.hidden = true;
    const need = new Set(schema.required ?? []);
    let first: HTMLElement | null = null;
    for (const [name, s] of Object.entries(schema.properties ?? {})) {
      const row = this.row(name, s, need.has(name), initial[name] ?? s.default);
      this.form.appendChild(row.node);
      first ??= row.focus;
    }
    this.open(first ?? this.ok);
    return new Promise((resolve) => { this.resolve = resolve; });
  }

  private row(name: string, s: JsonSchema, required: boolean, value: unknown): { node: HTMLElement; focus: HTMLElement } {
    const type = typeOf(s);
    const node = el('<label class="cv-field"><span></span></label>');
    const unit = loadPrefs().units;
    setText(q(node, 'span'), s.unit === 'mm' ? `${t(label(name))} (${type === 'number' ? unit : 'mm'})` : label(name));
    if (s.description) node.title = s.description;
    let control: HTMLElement;
    let read: Read;
    if (s.enum) {
      const sel = el<HTMLSelectElement>('<select class="cv-input"></select>');
      for (const v of s.enum) sel.append(new Option(String(v), String(v)));
      if (value !== undefined) sel.value = String(value);
      control = sel;
      read = () => (typeof s.enum?.[0] === 'number' ? Number(sel.value) : sel.value);
    } else if (type === 'boolean') {
      const box = el<HTMLInputElement>('<input type="checkbox">');
      box.checked = value === true;
      control = box;
      read = () => box.checked;
    } else if (type === 'number' || type === 'integer') {
      const inp = el<HTMLInputElement>('<input class="cv-input" type="text" inputmode="decimal" data-numtext spellcheck="false">');
      const f = type === 'number' && s.unit === 'mm' ? UNIT_MM[unit] ?? 1 : 1;
      if (typeof value === 'number') inp.value = String(Number((value / f).toFixed(4)));
      control = inp;
      read = () => {
        if (inp.value.trim() === '') return undefined;
        const v = evalExpr(inp.value);
        if (v === null) return Number.NaN;
        const n = type === 'integer' ? Math.round(v) : v * f;
        return Math.min(s.maximum ?? Infinity, Math.max(s.minimum ?? -Infinity, n));
      };
    } else if (type === 'array' || type === 'object') {
      const area = el<HTMLTextAreaElement>('<textarea class="cv-input cv-exact-json" spellcheck="false" rows="3"></textarea>');
      area.value = value === undefined ? '' : JSON.stringify(value);
      control = area;
      read = () => (area.value.trim() === '' ? undefined : JSON.parse(area.value));
    } else {
      const inp = el<HTMLInputElement>('<input class="cv-input" type="text" spellcheck="false">');
      if (typeof value === 'string') inp.value = value;
      control = inp;
      read = () => (inp.value.trim() === '' ? undefined : inp.value);
    }
    node.appendChild(control);
    this.readers.push({ name, required, read, box: node });
    return { node, focus: control };
  }

  private submit(): void {
    const out: Values = {};
    for (const r of this.readers) {
      let v: unknown;
      try { v = r.read(); } catch { return this.fail(`${label(r.name)}: ${t('is not valid JSON')}`); }
      if (typeof v === 'number' && !Number.isFinite(v)) return this.fail(`${label(r.name)}: ${t('is not a number')}`);
      if (v === undefined) {
        if (r.required) return this.fail(`${label(r.name)}: ${t('is required')}`);
        continue;
      }
      out[r.name] = v;
    }
    this.finish(out);
  }

  private fail(message: string): void {
    setText(this.error, message);
    this.error.hidden = false;
  }

  private finish(v: Values | null): void {
    const done = this.resolve;
    this.resolve = null;
    if (this.isOpen) this.close();
    done?.(v);
  }
}

let dialog: ExactInputDialog | null = null;

export function exactInput(title: string, schema: JsonSchema, initial: Values = {}): Promise<Values | null> {
  dialog ??= new ExactInputDialog();
  return dialog.ask(title, schema, initial);
}
