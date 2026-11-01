import './trims.css';
import type { ShellDeps } from '../../shell/deps';
import { el, q, setClass, setHidden, setText } from '../../ui/dom';
import { onLanguage, t } from '../../ui/i18n';
import { icon } from '../../ui/icons';
import { toast } from '../../ui/toast';
import type { DocStore } from '../doc';
import { canPickImage, pickImage } from '../fabric/files';
import { nativeDialogs, pickPath } from '../../shell/file-dialog';
import { decimals, fromUnit, toUnit, type Selection } from '../state';
import { itemLabel, removeTrim } from './actions';
import './icons';
import { KIND_ICON } from './icons';
import { trimState } from './state';
import {
  FINISH_KINDS, STYLE_KINDS, findPrint, str, flag, styleOf, trimDoc,
  type StyleKind, type TStyle, type TrimItem,
} from './types';
import {
  action, label, numRow, pairRow, patchKeyed, selRow, textRow, titled, toggleRow,
  type Row,
} from './widgets';

const OPEN_KEY = 'formfabric.trims.open';

interface Def {
  key: string;
  label: string;
  type: 'num' | 'sel' | 'bool';
  mm?: boolean;
  unit?: string;
  step?: number;
  options?: string[];
  def: number | string | boolean;
}

const COMMON: Def = { key: 'stiffness', label: 'Stiffness', type: 'num', step: 0.1, def: 1 };

const DEFS: Record<StyleKind, Def[]> = {
  button: [
    { key: 'width', label: 'Width', type: 'num', mm: true, step: 0.5, def: 12 },
    { key: 'thickness', label: 'Thickness', type: 'num', mm: true, step: 0.5, def: 3 },
    { key: 'weight', label: 'Weight', type: 'num', unit: 'g', step: 0.1, def: 1.5 },
    { key: 'thread', label: 'Thread', type: 'sel', options: ['cross', 'parallel', 'shank'], def: 'cross' },
    { key: 'collision', label: 'Collision', type: 'bool', def: true },
    COMMON,
  ],
  buttonhole: [
    { key: 'length', label: 'Length', type: 'num', mm: true, step: 0.5, def: 14 },
    { key: 'z_offset', label: 'Offset', type: 'num', mm: true, step: 0.05, def: 0.13 },
    COMMON,
  ],
  zipper: [
    { key: 'preset', label: 'Preset', type: 'sel', options: ['nylon', 'metal', 'plastic', 'invisible'], def: 'nylon' },
    { key: 'width', label: 'Width', type: 'num', mm: true, step: 0.5, def: 10 },
    COMMON,
  ],
  topstitch: [
    { key: 'count', label: 'Rows', type: 'num', step: 1, def: 1 },
    { key: 'shape', label: 'Shape', type: 'sel', options: ['single', 'double', 'zigzag'], def: 'single' },
    { key: 'spi', label: 'Stitches per inch', type: 'num', step: 1, def: 10 },
    { key: 'thickness', label: 'Thickness', type: 'num', mm: true, step: 0.05, def: 0.6 },
    { key: 'offset', label: 'Offset', type: 'num', mm: true, step: 0.5, def: 0 },
    { key: 'stiffen', label: 'Stiffen', type: 'num', step: 0.05, def: 1.15 },
  ],
  puckering: [
    { key: 'density', label: 'Density', type: 'num', unit: '%', step: 5, def: 50 },
    { key: 'width', label: 'Width', type: 'num', mm: true, step: 0.5, def: 3 },
  ],
  trim: [
    { key: 'collision', label: 'Collision', type: 'bool', def: true },
  ],
};

const KIND_NAMES: Record<string, string> = {
  button: 'Button', buttonhole: 'Buttonhole', zipper: 'Zipper', topstitch: 'Topstitch', puckering: 'Puckering', trim: 'Trim',
};

const FINISH_NAMES: Record<string, string> = {
  piping: 'Piping', binding: 'Binding', elastic: 'Elastic', tape: 'Tape', bond: 'Bond', press: 'Press',
};

const SIDES: Array<[string, string]> = [['auto', 'Auto'], ['front', 'Front'], ['back', 'Back']];

const SECTIONS: Array<[string, string, string]> = [
  ['styles', 'Styles', 'trims'],
  ['tools', 'Tool options', 'tTrimSelect'],
  ['items', 'Items', 'layers'],
  ['seams', 'Seams', 'seamCheck'],
];

const HTML =
  '<header class="ff-tr-head"><span data-k="title"></span><button class="cv-icon-btn" type="button" data-k="close"></button></header>' +
  '<div class="ff-tr-scroll">' +
  SECTIONS.map(([k, , ico]) => `<details class="ff-tr-sec" open data-s="${k}"><summary><span class="ff-tr-chev"></span>${icon(ico, 14)}<span data-k="h-${k}"></span></summary><div class="ff-tr-body" data-k="${k}"></div></details>`).join('') +
  '</div>';

interface ParamRow { def: Def; row: Row; set(style: TStyle): void }

export class TrimsPanel {
  private readonly drawer: HTMLElement;
  private readonly toggle: HTMLButtonElement;
  private readonly list: HTMLElement;
  private readonly listEmpty: HTMLElement;
  private readonly items: HTMLElement;
  private readonly itemsEmpty: HTMLElement;
  private readonly newKind;
  private readonly name;
  private readonly dup;
  private readonly del;
  private readonly groups = new Map<StyleKind, HTMLElement>();
  private readonly params: Array<[StyleKind, ParamRow]> = [];
  private readonly toolRows: Array<() => void> = [];
  private readonly seamPick;
  private readonly ripBtn;
  private readonly mendBtn;
  private readonly frayAmount;
  private readonly frayBtn;
  private readonly importBtn;
  private readonly objText;
  private objPath = '';
  private seamId = 0;
  private fray = 3;
  private isOpen = false;
  private yielding = false;
  private selStyle: number | null = null;
  private creating: StyleKind = 'button';

  constructor(root: HTMLElement, private readonly deps: ShellDeps, private readonly doc: DocStore, readonly sel: Selection) {
    this.toggle = el<HTMLButtonElement>(`<button class="cv-icon-btn ff-tr-toggle" type="button" aria-expanded="false">${icon('trims', 18)}</button>`);
    titled(this.toggle, 'Trims');
    const zoom = q(root, '[data-k="zoom"]');
    zoom.parentElement?.insertBefore(this.toggle, zoom);

    this.drawer = el('<aside class="ff-trims" data-k="trims" hidden></aside>');
    this.drawer.innerHTML = HTML;
    q(root, '.ff-body').appendChild(this.drawer);
    for (const [k, text] of SECTIONS) label(q(this.drawer, `[data-k="h-${k}"]`), text);
    label(q(this.drawer, '[data-k="title"]'), 'Trims');
    for (const c of this.drawer.querySelectorAll<HTMLElement>('.ff-tr-chev')) c.innerHTML = icon('chevronDown', 12);
    const close = q<HTMLButtonElement>(this.drawer, '[data-k="close"]');
    close.innerHTML = icon('close', 16);
    titled(close, 'Close');
    close.addEventListener('click', () => this.setOpen(false));
    this.toggle.addEventListener('click', () => { this.setOpen(!this.isOpen); this.toggle.blur(); });
    root.addEventListener('click', (e) => {
      if (this.isOpen && !this.yielding && e.target instanceof Element && e.target.closest('.ff-fb-toggle, .ff-bd-toggle')) this.setOpen(false);
    }, true);

    const part = (k: string): HTMLElement => q(this.drawer, `[data-k="${k}"].ff-tr-body`);
    const styles = part('styles');
    const bar = el('<div class="ff-tr-bar"></div>');
    this.newKind = selRow('Kind', STYLE_KINDS.map((k): [string, string] => [k, KIND_NAMES[k]]), true, (v) => { this.creating = v as StyleKind; });
    this.newKind.el.classList.add('ff-tr-kind');
    this.newKind.select.disabled = false;
    const mk = action('New style', 'add', () => { void this.create(); });
    const dupBtn = action('Duplicate', 'duplicate', () => { void this.duplicate(); });
    const delBtn = action('Delete', 'delete', () => { void this.remove(); });
    this.dup = dupBtn;
    this.del = delBtn;
    titled(mk.el, 'New style');
    titled(dupBtn.el, 'Duplicate');
    titled(delBtn.el, 'Delete');
    bar.append(this.newKind.el, mk.el, dupBtn.el, delBtn.el);
    this.list = el('<div class="ff-tr-list ff-tr-list-styles" role="listbox"></div>');
    this.listEmpty = el('<p class="ff-tr-empty"></p>');
    label(this.listEmpty, 'No styles');
    this.list.appendChild(this.listEmpty);
    this.name = textRow('Name', (v) => {
      if (v && this.selStyle !== null) void this.doc.run('style.update', { style: this.selStyle, name: v });
    });
    styles.append(bar, this.list, this.name.el);
    for (const kind of STYLE_KINDS) {
      const group = el('<div class="ff-tr-group" hidden></div>');
      this.groups.set(kind, group);
      for (const def of DEFS[kind]) {
        const pr = this.paramRow(def);
        group.appendChild(pr.row.el);
        this.params.push([kind, pr]);
      }
      styles.appendChild(group);
    }

    this.buildTools(part('tools'));
    const toolsBody = part('tools');
    const importHead = el('<p class="ff-tr-sub"></p>');
    label(importHead, 'Trim import');
    const objRow = el('<div class="ff-tr-pathrow"></div>');
    this.objText = textRow('File', (v) => { this.objPath = v; });
    this.objText.input.disabled = false;
    objRow.appendChild(this.objText.el);
    if (nativeDialogs()) {
      const browse = el<HTMLButtonElement>(`<button class="cv-icon-btn" type="button">${icon('folder', 16)}</button>`);
      titled(browse, 'Browse');
      browse.addEventListener('click', () => { void pickPath('open', null, 'obj').then((p) => { if (p) { this.objPath = p; this.objText.set(p); } }); });
      objRow.appendChild(browse);
    }
    this.importBtn = action('Import trim', 'tTrimMesh', () => { void this.importTrim(); });
    toolsBody.append(importHead, objRow, this.importBtn.el);

    const seamsBody = part('seams');
    this.seamPick = selRow('Seam', [['0', '-']], false, (v) => { this.seamId = Number(v); this.render(); });
    this.seamPick.select.disabled = false;
    this.ripBtn = action('Rip', null, () => { void this.doc.run('sew.rip', { seams: [this.seamId] }); });
    this.mendBtn = action('Mend', null, () => { void this.doc.run('sew.mend', { seams: [this.seamId] }); });
    this.frayAmount = numRow('Fray', 'mm', { min: 0.1, max: 30, step: 0.5 }, (v) => { this.fray = Math.min(30, Math.max(0.1, v)); });
    this.frayAmount.input.disabled = false;
    this.frayAmount.set(this.fray);
    this.frayBtn = action('Add fray', null, () => { if (this.seamId > 0) void this.doc.run('fray.add', { seam: this.seamId, amount: this.fray }); });
    const seamActions = el('<div class="ff-tr-actions"></div>');
    seamActions.append(this.ripBtn.el, this.mendBtn.el, this.frayBtn.el);
    seamsBody.append(this.seamPick.el, seamActions, this.frayAmount.el);

    this.items = el('<div class="ff-tr-list ff-tr-list-items" role="listbox"></div>');
    this.itemsEmpty = el('<p class="ff-tr-empty"></p>');
    label(this.itemsEmpty, 'No trims on this garment');
    this.items.appendChild(this.itemsEmpty);
    part('items').appendChild(this.items);

    this.list.addEventListener('click', (e) => {
      const node = (e.target as Element).closest<HTMLElement>('.ff-tr-item');
      if (node) this.pickStyle(Number(node.dataset.key));
    });
    this.items.addEventListener('click', (e) => {
      const node = (e.target as Element).closest<HTMLElement>('.ff-tr-item');
      if (!node) return;
      const [kind, id] = (node.dataset.key ?? '').split(':');
      if ((e.target as Element).closest('.cv-icon-btn')) void removeTrim(this.doc, { kind: kind as TrimItem['kind'], id: Number(id) });
      else trimState.select({ kind: kind as TrimItem['kind'], id: Number(id) });
    });

    doc.onChange(() => this.render());
    trimState.onChange(() => this.render());
    onLanguage(() => this.render());
    document.addEventListener('nm:ui-settings-changed', () => this.render());

    if (!deps.commands.get('app.trims')) {
      deps.commands.add({ id: 'app.trims', label: 'Trims Panel', group: 'Sew', icon: 'trims', run: () => { this.setOpen(!this.isOpen); }, checked: () => this.isOpen });
    }
    if (localStorage.getItem(OPEN_KEY) === '1') this.setOpen(true);
    else setClass(this.toggle, 'is-active', false);
  }

  private unit(): string { return this.deps.prefs().units; }

  private paramRow(def: Def): ParamRow {
    const send = (value: unknown): void => {
      if (this.selStyle !== null) void this.doc.run('style.update', { style: this.selStyle, params: { [def.key]: value } });
    };
    const read = (s: TStyle): number | string | boolean => {
      const raw = s.params[def.key];
      return raw === undefined || raw === null ? def.def : (raw as number | string | boolean);
    };
    if (def.type === 'bool') {
      const r = toggleRow(def.label, (on) => send(on));
      return { def, row: r, set: (s) => { r.disable(false); r.set(flag(s.params, def.key, def.def as boolean)); } };
    }
    if (def.type === 'sel') {
      const r = selRow(def.label, (def.options ?? []).map((o): [string, string] => [o, o.charAt(0).toUpperCase() + o.slice(1)]), true, (v) => send(v));
      return { def, row: r, set: (s) => { r.disable(false); r.set(str(s.params, def.key, def.def as string)); } };
    }
    const r = numRow(def.label, def.mm ? this.unit() : def.unit ?? '', { step: def.step }, (v) => send(def.mm ? fromUnit(v, this.unit()) : v));
    return {
      def, row: r,
      set: (s) => {
        r.disable(false);
        const v = Number(read(s));
        r.set(def.mm ? Number(toUnit(v, this.unit()).toFixed(decimals(this.unit()) + 2)) : v);
        if (def.mm) r.unit(this.unit());
      },
    };
  }

  private buildTools(body: HTMLElement): void {
    const o = trimState.opt;
    const u = (): string => this.unit();
    const sub = (key: string): HTMLElement => {
      const h = el('<p class="ff-tr-sub"></p>');
      label(h, key);
      body.appendChild(h);
      return h;
    };
    const optional = (labelKey: string, get: () => number | null, put: (v: number | null) => void, mm: boolean, unitText = ''): void => {
      const r = numRow(labelKey, mm ? u() : unitText, {}, (v) => put(mm ? fromUnit(v, u()) : v));
      r.input.disabled = false;
      r.input.placeholder = t('Style');
      r.input.addEventListener('change', () => { if (r.input.value.trim() === '') put(null); });
      body.appendChild(r.el);
      this.toolRows.push(() => {
        const v = get();
        r.set(v === null ? null : mm ? Number(toUnit(v, u()).toFixed(decimals(u()) + 2)) : v);
        if (mm) r.unit(u());
      });
    };
    sub('Button and buttonhole');
    const rot = numRow('Rotation', '°', { step: 1 }, (v) => { o.rotation = v; });
    rot.input.disabled = false;
    body.appendChild(rot.el);
    this.toolRows.push(() => rot.set(o.rotation));
    sub('Topstitch');
    optional('Offset', () => o.stitchOffset, (v) => { o.stitchOffset = v; }, true);
    sub('Edge Finish');
    const kind = selRow('Kind', FINISH_KINDS.map((k): [string, string] => [k, FINISH_NAMES[k]]), true, (v) => { o.finishKind = v as typeof o.finishKind; });
    kind.select.disabled = false;
    body.appendChild(kind.el);
    this.toolRows.push(() => kind.set(o.finishKind));
    optional('Width', () => o.finishWidth, (v) => { o.finishWidth = v; }, true);
    optional('Ratio', () => o.finishRatio, (v) => { o.finishRatio = v; }, false);
    optional('Strength', () => o.finishStrength, (v) => { o.finishStrength = v; }, false);
    sub('Print');
    const wrap = el('<div class="ff-tr-pathrow"></div>');
    const img = textRow('Image', (v) => { o.printImage = v; });
    img.input.disabled = false;
    wrap.appendChild(img.el);
    if (canPickImage()) {
      const browse = el<HTMLButtonElement>(`<button class="cv-icon-btn" type="button">${icon('folder', 16)}</button>`);
      titled(browse, 'Browse');
      browse.addEventListener('click', () => { void pickImage().then((p) => { if (p) { o.printImage = p; this.renderTools(); } }); });
      wrap.appendChild(browse);
    }
    body.appendChild(wrap);
    this.toolRows.push(() => img.set(o.printImage));
    const size = pairRow('Size', u(), (w, h) => { if (w > 0 && h > 0) { o.printWidth = fromUnit(w, u()); o.printHeight = fromUnit(h, u()); } });
    size.disable(false);
    body.appendChild(size.el);
    this.toolRows.push(() => {
      size.unit(u());
      size.set([Number(toUnit(o.printWidth, u()).toFixed(decimals(u()) + 2)), Number(toUnit(o.printHeight, u()).toFixed(decimals(u()) + 2))]);
    });
    const side = selRow('Side', SIDES, true, (v) => { o.printSide = v as typeof o.printSide; });
    side.select.disabled = false;
    body.appendChild(side.el);
    this.toolRows.push(() => side.set(o.printSide));
  }

  private renderTools(): void {
    for (const f of this.toolRows) f();
  }

  private pickStyle(id: number | null): void {
    this.selStyle = id;
    const s = id === null ? undefined : styleOf(trimDoc(this.doc.garment), id);
    if (s) trimState.setStyle(s.kind === 'puckering' ? 'topstitch' : s.kind, s.id);
    this.render();
  }

  private async create(): Promise<void> {
    const r = await this.doc.run<{ id?: number }>('style.create', { kind: this.creating });
    if (r && typeof r.id === 'number') this.pickStyle(r.id);
  }

  private async duplicate(): Promise<void> {
    const s = this.selStyle === null ? undefined : styleOf(trimDoc(this.doc.garment), this.selStyle);
    if (!s) return;
    const params = { ...s.params };
    delete params.mesh;
    const r = await this.doc.run<{ id?: number }>('style.create', { kind: s.kind, name: `${s.name} copy`, params });
    if (r && typeof r.id === 'number') this.pickStyle(r.id);
  }

  private async remove(): Promise<void> {
    if (this.selStyle === null) return;
    await this.doc.run('style.delete', { style: this.selStyle });
    this.render();
  }

  private async importTrim(): Promise<void> {
    if (!this.objPath && nativeDialogs()) {
      const p = await pickPath('open', null, 'obj');
      if (p) { this.objPath = p; this.objText.set(p); }
    }
    const ids = this.sel.list().filter((id) => this.doc.panel(id));
    if (!this.objPath) return;
    if (ids.length !== 1) { toast(t('Select one panel'), 'error'); return; }
    const geo = this.doc.geos().find((x) => x.panel.id === ids[0]);
    const point = geo ? geo.center : this.doc.panel(ids[0])?.position;
    if (!point) return;
    await this.doc.run('trim.import', { path: this.objPath, panel: ids[0], point });
  }

  setOpen(on: boolean): void {
    if (on === this.isOpen) return;
    this.isOpen = on;
    localStorage.setItem(OPEN_KEY, on ? '1' : '0');
    setClass(this.toggle, 'is-active', on);
    this.toggle.setAttribute('aria-expanded', String(on));
    if (on) {
      const other = document.querySelector<HTMLElement>('.ff-fb-toggle.is-active, .ff-bd-toggle.is-active');
      if (other) {
        this.yielding = true;
        other.click();
        this.yielding = false;
      }
      this.render();
    }
    setHidden(this.drawer, !on);
  }

  render(): void {
    if (!this.isOpen) return;
    const g = this.doc.garment;
    const d = trimDoc(g);
    if (this.selStyle !== null && !d.styles.some((s) => s.id === this.selStyle)) this.selStyle = null;
    if (this.selStyle === null && d.styles.length) this.selStyle = d.styles[0].id;
    const cur = this.selStyle === null ? undefined : styleOf(d, this.selStyle);

    const used = (id: number): number => d.hardware.filter((h) => h.style === id).length + d.zippers.filter((z) => z.style === id).length + d.stitches.filter((s) => s.style === id).length;
    patchKeyed(this.list, d.styles, {
      key: (s) => String(s.id),
      create: (s) => {
        const n = el(`<div class="ff-tr-item" role="option" tabindex="0">${icon(KIND_ICON[s.kind] ?? 'info', 14)}<span class="ff-tr-name"></span><span class="ff-tr-tag"></span><span class="ff-tr-cnt"></span></div>`);
        return n;
      },
      update: (n, s) => {
        setText(q(n, '.ff-tr-name'), s.name);
        setText(q(n, '.ff-tr-tag'), t(KIND_NAMES[s.kind] ?? s.kind));
        setText(q(n, '.ff-tr-cnt'), String(used(s.id)));
        setClass(n, 'is-sel', s.id === this.selStyle);
        n.setAttribute('aria-selected', String(s.id === this.selStyle));
      },
    });
    setHidden(this.listEmpty, d.styles.length > 0);

    this.name.disable(!cur);
    this.name.set(cur?.name ?? '');
    this.dup.set(!cur || (cur.kind === 'trim' && cur.params.mesh !== undefined));
    this.del.set(!cur);
    for (const [kind, group] of this.groups) setHidden(group, !cur || cur.kind !== kind);
    for (const [kind, pr] of this.params) {
      if (!cur || cur.kind !== kind) continue;
      pr.set(cur);
    }
    this.renderTools();

    const rows: TrimItem[] = [
      ...d.hardware.map((x): TrimItem => ({ kind: 'hardware', id: x.id })),
      ...d.zippers.map((x): TrimItem => ({ kind: 'zipper', id: x.id })),
      ...d.stitches.map((x): TrimItem => ({ kind: 'stitch', id: x.id })),
      ...d.finishes.map((x): TrimItem => ({ kind: 'finish', id: x.id })),
      ...g.panels.flatMap((p) => ((p as unknown as { prints?: Array<{ id: number }> }).prints ?? []).map((x): TrimItem => ({ kind: 'print', id: x.id }))),
    ];
    const here = trimState.item;
    patchKeyed(this.items, rows, {
      key: (r) => `${r.kind}:${r.id}`,
      create: (r) => {
        const iconName = r.kind === 'hardware' ? KIND_ICON[styleOf(d, d.hardware.find((h) => h.id === r.id)?.style ?? 0)?.kind ?? 'button'] : KIND_ICON[r.kind];
        const n = el(`<div class="ff-tr-item" role="option" tabindex="0">${icon(iconName ?? 'info', 14)}<span class="ff-tr-name"></span><span class="ff-tr-tag"></span><button class="cv-icon-btn" type="button" data-k="del">${icon('delete', 14)}</button></div>`);
        titled(q(n, '[data-k="del"]'), 'Delete');
        return n;
      },
      update: (n, r) => {
        setText(q(n, '.ff-tr-name'), `${t(itemLabel(g, r))} ${r.id}`);
        let tag = '';
        if (r.kind === 'hardware') tag = styleOf(d, d.hardware.find((h) => h.id === r.id)?.style ?? 0)?.name ?? '';
        else if (r.kind === 'zipper') tag = styleOf(d, d.zippers.find((z) => z.id === r.id)?.style ?? 0)?.name ?? '';
        else if (r.kind === 'stitch') tag = styleOf(d, d.stitches.find((s) => s.id === r.id)?.style ?? 0)?.name ?? '';
        else if (r.kind === 'finish') tag = t(FINISH_NAMES[d.finishes.find((f) => f.id === r.id)?.kind ?? ''] ?? '');
        else tag = findPrint(g, r.id)?.print.name ?? '';
        setText(q(n, '.ff-tr-tag'), tag);
        const on = here !== null && here.kind === r.kind && here.id === r.id;
        setClass(n, 'is-sel', on);
        n.setAttribute('aria-selected', String(on));
      },
    });
    setHidden(this.itemsEmpty, rows.length > 0);

    const seams = this.doc.sew.seams;
    this.seamPick.options(seams.length ? seams.map((s): [string, string] => [String(s.id), `${t('Seam')} ${s.id}`]) : [['0', '-']], false);
    if (!seams.some((s) => s.id === this.seamId)) this.seamId = this.sel.seam !== null && seams.some((s) => s.id === this.sel.seam) ? (this.sel.seam as number) : seams.length ? seams[0].id : 0;
    this.seamPick.set(String(this.seamId));
    const torn = d.ripped.includes(this.seamId);
    this.ripBtn.set(this.seamId === 0 || torn);
    this.mendBtn.set(this.seamId === 0 || !torn);
    this.frayBtn.set(this.seamId === 0);
  }
}
