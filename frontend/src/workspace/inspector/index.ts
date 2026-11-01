import { onLanguage, t } from '../../ui/i18n';
import { icon } from '../../ui/icons';
import { q, setDisabled, setText, setValue } from '../../ui/dom';
import type { DocStore } from '../doc';
import { mmText } from '../pattern-view/render-sew';
import { decimals, fmtArea, fmtLen, fromUnit, toUnit, type Selection } from '../state';
import type { Panel } from '../types';

type FieldKey = 'name' | 'px' | 'py' | 'rot' | 'grain' | 'pd' | 'layer';

const LABELS: Array<[string, string]> = [
  ['title', 'Inspector'], ['name', 'Name'], ['pos', 'Position'], ['rot', 'Rotation'], ['grain', 'Grain'], ['pd', 'Particle distance'],
  ['layer', 'Layer'], ['perimeter', 'Perimeter'], ['area', 'Area'], ['mirx', 'Mirror X'], ['miry', 'Mirror Y'],
  ['seam', 'Seam Allowance'], ['swidth', 'Width'], ['corners', 'Corners'], ['clear', 'Clear'], ['link', 'Link'], ['unlink', 'Unlink'],
  ['notches', 'Notches'], ['notch', 'Notch'], ['kind', 'Kind'], ['depth', 'Depth'], ['ndel', 'Delete'],
  ['inner', 'Inner Shape'], ['ishape', 'Shape'], ['role', 'Role'], ['fangle', 'Fold angle'], ['fstrength', 'Fold strength'],
  ['smh', 'Seam'], ['smfold', 'Fold'], ['smratio', 'Ratio'], ['smtension', 'Tension'], ['smsa', 'Side A'], ['smsb', 'Side B'],
  ['smla', 'Length A'], ['smlb', 'Length B'], ['smmis', 'Mismatch'], ['smflip', 'Flip'], ['smgather', 'Gather'], ['tkh', 'Tack'], ['tklen', 'Length'],
];

const FOLDS: Array<[string, string]> = [['turned', 'Turned'], ['flat', 'Flat'], ['custom', 'Custom']];

const CORNERS: Array<[string, string]> = [['miter', 'Miter'], ['square', 'Square'], ['round', 'Round'], ['mirror', 'Mirror']];
const KINDS: Array<[string, string]> = [['slit', 'Slit'], ['v', 'V'], ['t', 'T'], ['u', 'U']];
const ROLES: Array<[string, string]> = [['shape', 'Shape'], ['fold', 'Fold'], ['dart', 'Dart'], ['reference', 'Reference']];

const num = (key: string, extra = ''): string => `<input class="cv-input ff-num" type="number" step="any" data-k="${key}" ${extra} disabled>`;
const pick = (key: string): string => `<div class="ff-sel"><select class="cv-input" data-k="${key}" disabled></select></div>`;
const row = (label: string, control: string): string => `<span class="ff-ins-label" data-l="${label}"></span>${control}`;
const btn = (key: string, label: string): string => `<span></span><button class="ff-btn" type="button" data-k="${key}" disabled><span data-l="${label}"></span></button>`;
const sub = (label: string): string => `<header class="ff-ins-sub"><span data-l="${label}"></span></header>`;

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
  `<div class="ff-ins-actions"><button class="ff-btn" type="button" data-k="mirx" disabled>${icon('layout2d', 14)}<span data-l="mirx"></span></button><button class="ff-btn" type="button" data-k="miry" disabled>${icon('layout2d', 14)}<span data-l="miry"></span></button></div>` +
  sub('seam') +
  '<div class="ff-ins-grid">' + row('swidth', num('sa_w', 'min="0" max="500"')) + row('corners', pick('sa_c')) + btn('sa_clear', 'clear') + '</div>' +
  sub('link') +
  '<div class="ff-ins-grid"><span class="ff-ins-label" data-l="link"></span><output class="ff-ins-out" data-k="linkinfo">-</output>' + btn('unlink', 'unlink') + '</div>' +
  sub('notches') +
  '<div class="ff-ins-grid">' + row('notch', pick('nt_sel')) + row('kind', pick('nt_k')) + row('depth', num('nt_d', 'min="0.5"')) + row('swidth', num('nt_w', 'min="0.1"')) + btn('nt_del', 'ndel') + '</div>' +
  sub('inner') +
  '<div class="ff-ins-grid">' + row('ishape', pick('in_sel')) + row('role', pick('in_r')) + row('fangle', num('in_a', 'min="0" max="360"')) + row('fstrength', num('in_s', 'min="0" max="20"')) + '</div>' +
  sub('smh') +
  '<div class="ff-ins-grid">' + row('smh', pick('sm_sel')) + row('smfold', pick('sm_f')) + row('fangle', num('sm_a', 'min="0" max="360"')) + row('fstrength', num('sm_s', 'min="0" max="20"')) +
  row('smratio', num('sm_r', 'min="0.05" max="20" step="0.05"')) + row('smtension', num('sm_t', 'min="0" max="20" step="0.1"')) +
  row('smsa', pick('sm_sa')) + '<span class="ff-ins-label" data-l="smsb"></span><output class="ff-ins-out" data-k="sm_sb">-</output>' +
  '<span class="ff-ins-label" data-l="smla"></span><output class="ff-ins-out" data-k="sm_la">-</output>' +
  '<span class="ff-ins-label" data-l="smlb"></span><output class="ff-ins-out" data-k="sm_lb">-</output>' +
  '<span class="ff-ins-label" data-l="smmis"></span><output class="ff-ins-out" data-k="sm_mis">-</output></div>' +
  '<div class="ff-ins-actions ff-ins-actions3"><button class="ff-btn" type="button" data-k="sm_flip" disabled><span data-l="smflip"></span></button><button class="ff-btn" type="button" data-k="sm_gather" disabled><span data-l="smgather"></span></button><button class="ff-btn" type="button" data-k="sm_del" disabled><span data-l="ndel"></span></button></div>' +
  sub('tkh') +
  '<div class="ff-ins-grid">' + row('tkh', pick('tk_sel')) + '<span class="ff-ins-label" data-l="tklen"></span><output class="ff-ins-out" data-k="tk_len">-</output>' + btn('tk_del', 'ndel') + '</div>';

export class Inspector {
  private readonly input = new Map<string, HTMLInputElement>();
  private readonly choice = new Map<string, HTMLSelectElement>();
  private notchId = 0;

  constructor(private readonly root: HTMLElement, private readonly doc: DocStore, private readonly sel: Selection, private readonly unit: () => string, private readonly onInner: () => void = () => undefined) {
    root.innerHTML = HTML;
    for (const el of root.querySelectorAll<HTMLInputElement>('input[data-k]')) this.input.set(el.dataset.k ?? '', el);
    for (const el of root.querySelectorAll<HTMLSelectElement>('select[data-k]')) this.choice.set(el.dataset.k ?? '', el);
    const paint = (): void => {
      for (const [k, text] of LABELS) for (const el of root.querySelectorAll<HTMLElement>(`[data-l="${k}"]`)) setText(el, t(text));
      this.update();
    };
    paint();
    onLanguage(paint);
    for (const [k, el] of this.input) el.addEventListener('change', () => this.commit(k as FieldKey));
    for (const [k, el] of this.choice) el.addEventListener('change', () => this.extra(k));
    for (const k of ['sa_clear', 'unlink', 'nt_del', 'sm_flip', 'sm_gather', 'sm_del', 'tk_del']) q(root, `[data-k="${k}"]`).addEventListener('click', () => this.extra(k));
    this.input.get('name')?.addEventListener('keydown', (e) => { if (e.key === 'Enter') (e.target as HTMLElement).blur(); });
    q(root, '[data-k="mirx"]').addEventListener('click', () => this.mirror('x'));
    q(root, '[data-k="miry"]').addEventListener('click', () => this.mirror('y'));
  }

  private sewAct(k: string): boolean {
    const text = (key: string): string => this.choice.get(key)?.value ?? '';
    const num = (key: string): number => Number(this.input.get(key)?.value);
    if (k === 'sm_sel') {
      const v = Number(text('sm_sel'));
      this.sel.seam = v > 0 ? v : null;
      if (v > 0) this.sel.tack = null;
      this.update();
      this.onInner();
      return true;
    }
    if (k === 'tk_sel') {
      const v = Number(text('tk_sel'));
      this.sel.tack = v > 0 ? v : null;
      if (v > 0) this.sel.seam = null;
      this.update();
      this.onInner();
      return true;
    }
    if (k === 'tk_del') {
      if (this.sel.tack !== null) void this.doc.run('tack.delete', { tack: this.sel.tack });
      return true;
    }
    const seam = this.sel.seam;
    if (seam === null) return true;
    if (k === 'sm_flip') void this.doc.run('sew.flip', { seam });
    else if (k === 'sm_del') void this.doc.run('sew.delete', { seam });
    else if (k === 'sm_gather') void this.doc.run('sew.gather', { seam, ratio: Math.min(20, Math.max(1, num('sm_r') || 1.5)) });
    else if (k === 'sm_f') void this.doc.run('sew.edit', { seam, fold: text('sm_f') });
    else if (k === 'sm_sa') void this.doc.run('sew.edit', { seam, side: text('sm_sa') });
    else {
      const v = num(k);
      if (!Number.isFinite(v)) return true;
      if (k === 'sm_a') void this.doc.run('sew.edit', { seam, fold: 'custom', fold_angle: Math.min(360, Math.max(0, v)) });
      else if (k === 'sm_s') void this.doc.run('sew.edit', { seam, fold: 'custom', fold_strength: Math.min(20, Math.max(0, v)) });
      else if (k === 'sm_r') void this.doc.run('sew.edit', { seam, ratio: Math.min(20, Math.max(0.05, v)) });
      else if (k === 'sm_t') void this.doc.run('sew.edit', { seam, tension: Math.min(20, Math.max(0, v)) });
    }
    return true;
  }

  private refreshSew(): void {
    const u = this.unit();
    const { seams, tacks, info } = this.doc.sew;
    const nm = (id: number | undefined): string => (id === undefined ? '-' : this.doc.panel(id)?.name ?? String(id));
    const cur = seams.find((s) => s.id === this.sel.seam);
    const inf = cur ? info.get(cur.id) : undefined;
    const set = (k: string, v: string): void => { const el = this.input.get(k); if (el) setValue(el, v); };
    const off = (k: string, on: boolean): void => {
      const a = this.input.get(k);
      const b = this.choice.get(k);
      if (a) setDisabled(a, on);
      if (b) setDisabled(b, on);
    };
    const out = (k: string, v: string): void => setText(q(this.root, `output[data-k="${k}"]`), v);
    const list: Array<[string, string]> = [['', '-']];
    for (const s of seams) list.push([String(s.id), `#${s.id} ${nm(s.a[0]?.panel)} / ${nm(s.b[0]?.panel)}`]);
    this.fill('sm_sel', list, cur ? String(cur.id) : '');
    off('sm_sel', seams.length === 0);
    this.fill('sm_f', FOLDS.map(([v, l]) => [v, t(l)]), cur?.fold ?? 'turned');
    const auto = cur && inf ? `${t('Auto')} · ${t(inf.side_resolved === 'back' ? 'Back' : 'Front')}` : t('Auto');
    this.fill('sm_sa', [['auto', auto], ['front', t('Front face of this panel')], ['back', t('Back face of this panel')]], cur?.side ?? 'auto');
    const custom = cur?.fold === 'custom';
    set('sm_a', cur ? String(Math.round(cur.fold_angle * 100) / 100) : '');
    set('sm_s', cur ? String(Math.round(cur.fold_strength * 100) / 100) : '');
    set('sm_r', cur ? String(Math.round(cur.ratio * 1000) / 1000) : '');
    set('sm_t', cur ? String(Math.round(cur.tension * 1000) / 1000) : '');
    for (const k of ['sm_f', 'sm_r', 'sm_t', 'sm_sa']) off(k, !cur);
    for (const k of ['sm_a', 'sm_s']) off(k, !custom);
    out('sm_sb', cur ? t('Derived from placement') : '-');
    out('sm_la', inf ? fmtLen(inf.length_a, u) : '-');
    out('sm_lb', inf ? fmtLen(inf.length_b, u) : '-');
    out('sm_mis', inf ? mmText(inf.mismatch) : '-');
    for (const k of ['sm_flip', 'sm_gather', 'sm_del']) setDisabled(q(this.root, `[data-k="${k}"]`), !cur);
    const tack = tacks.find((k) => k.id === this.sel.tack);
    this.fill('tk_sel', [['', '-'], ...tacks.map((k): [string, string] => [String(k.id), `#${k.id} ${nm(k.a.panel ?? undefined)} / ${nm(k.b.panel ?? undefined)}`])], tack ? String(tack.id) : '');
    off('tk_sel', tacks.length === 0);
    out('tk_len', tack ? fmtLen(tack.length, u) : '-');
    setDisabled(q(this.root, '[data-k="tk_del"]'), !tack);
  }

  private extra(k: string): boolean {
    if (k.startsWith('sm_') || k.startsWith('tk_')) return this.sewAct(k);
    if (!['sa_w', 'sa_c', 'sa_clear', 'unlink', 'nt_sel', 'nt_k', 'nt_d', 'nt_w', 'nt_del', 'in_sel', 'in_r', 'in_a', 'in_s'].includes(k)) return false;
    const ids = this.ids();
    const panel = ids.length === 1 ? this.doc.panel(ids[0]) : undefined;
    if (!panel) return true;
    const u = this.unit();
    const num = (key: string): number => Number(this.input.get(key)?.value);
    const text = (key: string): string => this.choice.get(key)?.value ?? '';
    const notch = Number(text('nt_sel')) || 0;
    const inner = Number(text('in_sel')) || 0;
    if (k === 'sa_w' || k === 'sa_c') {
      const w = k === 'sa_w' ? fromUnit(num('sa_w'), u) : (panel.seam_allowance?.width ?? 0);
      if (!Number.isFinite(w)) return true;
      if (w <= 0) { if (panel.seam_allowance) void this.doc.run('pattern.seam_allowance', { panels: [panel.id], clear: true }); return true; }
      void this.doc.run('pattern.seam_allowance', { panels: [panel.id], width: Math.min(500, w), corners: text('sa_c') || 'miter' });
    } else if (k === 'sa_clear') void this.doc.run('pattern.seam_allowance', { panels: [panel.id], clear: true });
    else if (k === 'unlink') void this.doc.run('pattern.unlink', { panel: panel.id });
    else if (k === 'nt_sel') { this.notchId = notch; this.update(); }
    else if (k === 'nt_k') { if (notch) void this.doc.run('pattern.notch_set', { panel: panel.id, notch, kind: text('nt_k') }); }
    else if (k === 'nt_d' || k === 'nt_w') {
      const v = fromUnit(num(k), u);
      if (notch && Number.isFinite(v)) void this.doc.run('pattern.notch_set', k === 'nt_d' ? { panel: panel.id, notch, depth: v } : { panel: panel.id, notch, width: v });
    } else if (k === 'nt_del') { if (notch) void this.doc.run('pattern.notch_delete', { panel: panel.id, notch }); }
    else if (k === 'in_sel') { this.sel.inner = inner ? { panel: panel.id, id: inner } : null; this.update(); this.onInner(); }
    else if (k === 'in_r') { if (inner) void this.doc.run('pattern.set_inner', { panel: panel.id, inner, role: text('in_r') }); }
    else if (k === 'in_a' || k === 'in_s') {
      const v = num(k);
      if (inner && Number.isFinite(v)) void this.doc.run('pattern.set_inner', k === 'in_a' ? { panel: panel.id, inner, fold_angle: Math.min(360, Math.max(0, v)) } : { panel: panel.id, inner, fold_strength: Math.min(20, Math.max(0, v)) });
    }
    return true;
  }

  private fill(key: string, items: Array<[string, string]>, current: string): void {
    const el = this.choice.get(key);
    if (!el) return;
    while (el.options.length > items.length) el.remove(el.options.length - 1);
    items.forEach(([value, label], i) => {
      let o = el.options[i];
      if (!o) { o = new Option(label, value); el.add(o); return; }
      if (o.value !== value) o.value = value;
      if (o.text !== label) o.text = label;
    });
    if (el.value !== current && document.activeElement !== el) el.value = current;
  }

  private refreshExtra(first: Panel | undefined, single: boolean): void {
    const u = this.unit();
    const d = decimals(u) + 1;
    const lock = !!first && !!first.link;
    const ok = !!first && single;
    const set = (k: string, v: string): void => { const el = this.input.get(k); if (el) setValue(el, v); };
    const off = (k: string, on: boolean): void => {
      const a = this.input.get(k);
      const b = this.choice.get(k);
      if (a) setDisabled(a, on);
      if (b) setDisabled(b, on);
    };
    const sa = first?.seam_allowance ?? null;
    this.fill('sa_c', CORNERS.map(([v, l]) => [v, t(l)]), sa?.corners ?? 'miter');
    set('sa_w', ok && sa ? toUnit(sa.width, u).toFixed(d) : '');
    off('sa_w', !ok || lock);
    off('sa_c', !ok || lock);
    setDisabled(q(this.root, '[data-k="sa_clear"]'), !ok || lock || !sa);

    const link = first?.link ?? null;
    const copies = first ? this.doc.garment.panels.filter((p) => p.link && p.link.source === first.id).length : 0;
    let info = '-';
    if (ok && link) info = `${t(link.kind === 'mirror' ? 'Mirror' : 'Instance')} · ${this.doc.panel(link.source)?.name ?? link.source}`;
    else if (ok && copies > 0) info = `${t('Source')} · ${copies}`;
    setText(q(this.root, 'output[data-k="linkinfo"]'), info);
    setDisabled(q(this.root, '[data-k="unlink"]'), !ok || (!link && copies === 0));

    const notches = ok && first ? first.notches ?? [] : [];
    const kindLabel = (k: string): string => t(KINDS.find((x) => x[0] === k)?.[1] ?? k);
    const items = notches.map((n, i): [string, string] => [String(n.id), `${i + 1} · ${kindLabel(n.kind)}`]);
    if (!notches.some((n) => n.id === this.notchId)) this.notchId = notches[0]?.id ?? 0;
    const cur = notches.find((n) => n.id === this.notchId);
    this.fill('nt_sel', items, cur ? String(cur.id) : '');
    this.fill('nt_k', KINDS.map(([v, l]) => [v, t(l)]), cur?.kind ?? 'slit');
    set('nt_d', cur ? toUnit(cur.depth, u).toFixed(d) : '');
    set('nt_w', cur ? toUnit(cur.width, u).toFixed(d) : '');
    off('nt_sel', !cur);
    off('nt_k', !cur || lock);
    off('nt_d', !cur || lock);
    off('nt_w', !cur || lock);
    setDisabled(q(this.root, '[data-k="nt_del"]'), !cur || lock);

    const inners = ok && first ? first.inner : [];
    const wanted = this.sel.inner && first && this.sel.inner.panel === first.id ? this.sel.inner.id : 0;
    const pickInner = inners.find((i) => i.id === wanted) ?? inners[0];
    this.fill('in_sel', inners.map((i): [string, string] => [String(i.id), `#${i.id} ${i.kind}`]), pickInner ? String(pickInner.id) : '');
    this.fill('in_r', ROLES.map(([v, l]) => [v, t(l)]), pickInner?.role ?? 'shape');
    const fold = pickInner?.role === 'fold';
    set('in_a', pickInner && fold ? String(Math.round(pickInner.fold_angle * 100) / 100) : '');
    set('in_s', pickInner && fold ? String(Math.round(pickInner.fold_strength * 100) / 100) : '');
    off('in_sel', !pickInner);
    off('in_r', !pickInner || lock);
    off('in_a', !fold || lock);
    off('in_s', !fold || lock);
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
    if (this.extra(k)) return;
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
    const locked = !!first && !!first.link;
    off('name', !single);
    for (const k of ['px', 'py', 'rot']) off(k, !single || locked);
    off('grain', !first || locked);
    for (const k of ['pd', 'layer']) off(k, !first);
    for (const k of ['mirx', 'miry']) setDisabled(q(this.root, `[data-k="${k}"]`), !first || locked);
    this.refreshExtra(first, single);
    this.refreshSew();
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
