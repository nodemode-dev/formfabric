import { onLanguage, t } from '../../ui/i18n';
import { icon } from '../../ui/icons';
import { q, setDisabled, setText, setValue } from '../../ui/dom';
import type { DocStore } from '../doc';
import { decimals, fmtArea, fmtLen, fromUnit, toUnit, type Selection } from '../state';

type FieldKey = 'name' | 'px' | 'py' | 'rot' | 'grain' | 'pd' | 'layer';

const LABELS: Array<[string, string]> = [
  ['title', 'Inspector'], ['name', 'Name'], ['pos', 'Position'], ['rot', 'Rotation'], ['grain', 'Grain'], ['pd', 'Particle distance'],
  ['layer', 'Layer'], ['perimeter', 'Perimeter'], ['area', 'Area'], ['mirx', 'Mirror X'], ['miry', 'Mirror Y'],
];

const num = (key: string, extra = ''): string => `<input class="cv-input ff-num" type="number" step="any" data-k="${key}" ${extra} disabled>`;

const HTML =
  '<header class="ff-ins-head"><span data-l="title"></span></header>' +
  '<div class="ff-ins-count" data-k="count"></div>' +
  '<div class="ff-ins-grid">' +
  '<span class="ff-ins-label" data-l="name"></span><input class="cv-input" type="text" spellcheck="false" data-k="name" disabled>' +
  '<span class="ff-ins-label" data-l="pos"></span><div class="ff-pair">' + num('px') + num('py') + '</div>' +
  '<span class="ff-ins-label" data-l="rot"></span>' + num('rot') +
  '<span class="ff-ins-label" data-l="grain"></span>' + num('grain') +
  '<span class="ff-ins-label" data-l="pd"></span>' + num('pd', 'min="0.1"') +
  '<span class="ff-ins-label" data-l="layer"></span>' + num('layer', 'step="1"') +
  '</div>' +
  '<div class="ff-ins-grid ff-ins-ro">' +
  '<span class="ff-ins-label" data-l="perimeter"></span><output data-k="perimeter">-</output>' +
  '<span class="ff-ins-label" data-l="area"></span><output data-k="area">-</output>' +
  '</div>' +
  `<div class="ff-ins-actions"><button class="ff-btn" type="button" data-k="mirx" disabled>${icon('layout2d', 14)}<span data-l="mirx"></span></button><button class="ff-btn" type="button" data-k="miry" disabled>${icon('layout2d', 14)}<span data-l="miry"></span></button></div>`;

export class Inspector {
  private readonly input = new Map<string, HTMLInputElement>();

  constructor(private readonly root: HTMLElement, private readonly doc: DocStore, private readonly sel: Selection, private readonly unit: () => string) {
    root.innerHTML = HTML;
    for (const el of root.querySelectorAll<HTMLInputElement>('input[data-k]')) this.input.set(el.dataset.k ?? '', el);
    const paint = (): void => {
      for (const [k, text] of LABELS) for (const el of root.querySelectorAll<HTMLElement>(`[data-l="${k}"]`)) setText(el, t(text));
      this.update();
    };
    paint();
    onLanguage(paint);
    for (const [k, el] of this.input) el.addEventListener('change', () => this.commit(k as FieldKey));
    this.input.get('name')?.addEventListener('keydown', (e) => { if (e.key === 'Enter') (e.target as HTMLElement).blur(); });
    q(root, '[data-k="mirx"]').addEventListener('click', () => this.mirror('x'));
    q(root, '[data-k="miry"]').addEventListener('click', () => this.mirror('y'));
  }

  focusName(): void {
    const n = this.input.get('name');
    if (n && !n.disabled) { n.focus(); n.select(); }
  }

  private ids(): number[] { return this.sel.list().filter((id) => this.doc.panel(id)); }

  private mirror(axis: 'x' | 'y'): void {
    const ids = this.ids();
    if (ids.length) void this.doc.run('pattern.transform', { panels: ids, mirror: axis });
  }

  private commit(k: FieldKey): void {
    const ids = this.ids();
    if (!ids.length) return;
    const u = this.unit();
    const val = (key: string): number => Number(this.input.get(key)?.value);
    if (k === 'name') {
      const name = this.input.get('name')?.value.trim() ?? '';
      if (name) void this.doc.run('pattern.rename', { panel: ids[0], name });
      return;
    }
    const args: Record<string, unknown> = { panels: ids };
    if (k === 'px' || k === 'py') {
      const x = val('px');
      const y = val('py');
      if (!Number.isFinite(x) || !Number.isFinite(y)) return;
      args.position = [fromUnit(x, u), fromUnit(y, u)];
    } else {
      const v = val(k);
      if (!Number.isFinite(v)) return;
      if (k === 'rot') args.rotation = v;
      else if (k === 'grain') args.grain = v;
      else if (k === 'pd') args.particle_distance = fromUnit(v, u);
      else args.layer = Math.round(v);
    }
    void this.doc.run('pattern.set_props', args);
  }

  update(): void {
    const ids = this.ids();
    const first = ids.length ? this.doc.panel(ids[0]) : undefined;
    const u = this.unit();
    const d = decimals(u) + 1;
    const set = (k: string, v: string): void => { const el = this.input.get(k); if (el) setValue(el, v); };
    const off = (k: string, on: boolean): void => { const el = this.input.get(k); if (el) setDisabled(el, on); };
    const single = ids.length === 1;
    setText(q(this.root, '[data-k="count"]'), !first ? t('Nothing selected') : single ? first.name : `${t('Panels')}: ${ids.length}`);
    for (const k of ['name', 'px', 'py', 'rot']) off(k, !single);
    for (const k of ['grain', 'pd', 'layer']) off(k, !first);
    for (const k of ['mirx', 'miry']) setDisabled(q(this.root, `[data-k="${k}"]`), !first);
    const out = (k: string, v: string): void => setText(q(this.root, `output[data-k="${k}"]`), v);
    if (!first) {
      for (const k of ['name', 'px', 'py', 'rot', 'grain', 'pd', 'layer']) set(k, '');
      out('perimeter', '-');
      out('area', '-');
      return;
    }
    set('name', single ? first.name : '');
    set('px', single ? toUnit(first.position[0], u).toFixed(d) : '');
    set('py', single ? toUnit(first.position[1], u).toFixed(d) : '');
    set('rot', single ? String(Math.round(first.rotation * 100) / 100) : '');
    set('grain', String(Math.round(first.grain * 100) / 100));
    set('pd', toUnit(first.particle_distance, u).toFixed(d));
    set('layer', String(first.layer));
    const m = single ? this.doc.measure(first.id) : null;
    out('perimeter', m ? fmtLen(m.perimeter, u) : '-');
    out('area', m ? fmtArea(m.area, u) : '-');
  }
}
