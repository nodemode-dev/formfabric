import { nativeDialogs } from '../../shell/file-dialog';
import { el, q, setClass, setHidden, setText } from '../../ui/dom';
import { t } from '../../ui/i18n';
import { icon } from '../../ui/icons';
import { emptyLabel } from '../offline';
import type { Ctx } from './ctx';
import { IMPORT_EXT, type BodyInfo, type LibraryEntry } from './types';
import { flagBad, label, patchKeyed, repaint, selectRow, titled } from './widgets';

const LIST_HTML =
  '<div class="ff-bd-list" data-k="list" role="listbox"><p class="ff-bd-empty" data-k="empty"></p></div>';

const ATTRIBUTION = 'Default bodies from MakeHuman community assets via MPFB2 (CC0 1.0).';

const UNITS: ReadonlyArray<readonly [string, string]> = [['auto', 'Auto'], ['mm', 'mm'], ['cm', 'cm'], ['m', 'm'], ['in', 'in'], ['ft', 'ft']];
const AXES: ReadonlyArray<readonly [string, string]> = [['y', '+Y'], ['-y', '-Y'], ['z', '+Z'], ['-z', '-Z'], ['x', '+X'], ['-x', '-X']];

function iconButton(ico: string, key: string, cls: string): HTMLButtonElement {
  const b = el<HTMLButtonElement>(`<button class="cv-icon-btn ${cls}" type="button">${icon(ico, 16)}</button>`);
  titled(b, key);
  return b;
}

export class BodiesSection {
  private readonly list: HTMLElement;
  private readonly empty: HTMLElement;

  constructor(host: HTMLElement, private readonly ctx: Ctx) {
    host.innerHTML = LIST_HTML;
    this.list = q(host, '[data-k="list"]');
    this.empty = q(host, '[data-k="empty"]');
    emptyLabel(this.empty, 'No bodies yet.');
    repaint(() => { for (const v of this.list.querySelectorAll<HTMLElement>('.ff-bd-vis')) this.titleVis(v); });
  }

  private titleVis(vis: HTMLElement): void {
    const s = t(vis.dataset.state === 'off' ? 'Show' : 'Hide');
    if (vis.title !== s) vis.title = s;
    vis.setAttribute('aria-label', s);
  }

  update(): void {
    const { store } = this.ctx;
    const sel = store.selected()?.id ?? null;
    patchKeyed<BodyInfo>(this.list, store.list.bodies, {
      key: (b) => String(b.id),
      create: () => this.create(),
      update: (n, b) => this.paint(n, b, sel),
    });
    setHidden(this.empty, store.list.bodies.length > 0);
  }

  private create(): HTMLElement {
    const node = el('<div class="ff-bd-item" role="option" tabindex="0"><span class="ff-bd-vis-slot"></span><span class="ff-bd-name"></span><span class="ff-bd-tag"></span><span class="ff-bd-del-slot"></span></div>');
    const vis = el<HTMLButtonElement>(`<button class="cv-icon-btn ff-bd-vis" type="button"><span class="ff-bd-eye-on">${icon('eye', 16)}</span><span class="ff-bd-eye-off" hidden>${icon('eyeOff', 16)}</span></button>`);
    vis.dataset.state = 'on';
    this.titleVis(vis);
    const del = iconButton('delete', 'Remove', 'ff-bd-del');
    q(node, '.ff-bd-vis-slot').appendChild(vis);
    q(node, '.ff-bd-del-slot').appendChild(del);
    const id = (): number => Number(node.dataset.key);
    const pick = (): void => { this.ctx.store.select(id()); };
    node.addEventListener('click', pick);
    node.addEventListener('keydown', (e) => {
      if (e.target !== node || (e.key !== 'Enter' && e.key !== ' ')) return;
      e.preventDefault();
      pick();
    });
    vis.addEventListener('click', (e) => {
      e.stopPropagation();
      const b = this.ctx.store.body(id());
      if (b) void this.ctx.store.run('body.show', { id: b.id, visible: !b.visible });
      if (e.detail > 0) vis.blur();
    });
    del.addEventListener('click', (e) => {
      e.stopPropagation();
      void this.ctx.store.run('body.remove', { id: id() });
      if (e.detail > 0) del.blur();
    });
    return node;
  }

  private paint(node: HTMLElement, b: BodyInfo, sel: number | null): void {
    setText(q(node, '.ff-bd-name'), b.name);
    setText(q(node, '.ff-bd-tag'), b.license ?? '');
    setClass(node, 'is-sel', b.id === sel);
    setClass(node, 'is-hidden', !b.visible);
    node.setAttribute('aria-selected', String(b.id === sel));
    const vis = q(node, '.ff-bd-vis');
    const state = b.visible ? 'on' : 'off';
    if (vis.dataset.state !== state) {
      vis.dataset.state = state;
      setHidden(q(vis, '.ff-bd-eye-on'), !b.visible);
      setHidden(q(vis, '.ff-bd-eye-off'), b.visible);
      this.titleVis(vis);
    }
  }
}

export class LibrarySection {
  private readonly list: HTMLElement;

  constructor(host: HTMLElement, private readonly ctx: Ctx) {
    host.innerHTML = '<div class="ff-bd-list ff-bd-list-s" data-k="list"></div><p class="ff-bd-note" data-k="note"></p>';
    this.list = q(host, '[data-k="list"]');
    label(q(host, '[data-k="note"]'), ATTRIBUTION);
    repaint(() => { for (const b of this.list.querySelectorAll<HTMLButtonElement>('.ff-bd-add')) this.titleAdd(b); });
  }

  private titleAdd(b: HTMLButtonElement): void {
    const s = t('Add Body');
    if (b.title !== s) b.title = s;
    b.setAttribute('aria-label', s);
  }

  update(): void {
    patchKeyed<LibraryEntry>(this.list, this.ctx.store.list.library, {
      key: (e) => e.name,
      create: (e) => {
        const node = el('<div class="ff-bd-item"><span class="ff-bd-name"></span><span class="ff-bd-tag"></span></div>');
        const add = iconButton('add', 'Add Body', 'ff-bd-add');
        node.appendChild(add);
        add.addEventListener('click', (ev) => { void this.add(e.name); if (ev.detail > 0) add.blur(); });
        return node;
      },
      update: (n, e) => {
        setText(q(n, '.ff-bd-name'), e.name);
        setText(q(n, '.ff-bd-tag'), e.license ?? '');
      },
    });
  }

  private async add(name: string): Promise<void> {
    const r = await this.ctx.store.run<BodyInfo>('body.load_default', { name });
    if (r) this.ctx.store.select(r.id);
  }
}

export class ImportSection {
  private readonly path: HTMLInputElement;
  private readonly go: HTMLButtonElement;
  private units = 'auto';
  private up = 'y';

  constructor(host: HTMLElement, private readonly ctx: Ctx) {
    host.innerHTML =
      '<div class="ff-bd-pathrow"><input class="cv-input ff-bd-path" type="text" spellcheck="false" autocomplete="off"></div>' +
      '<div class="ff-bd-actions"><button class="ff-btn ff-bd-go" type="button"></button></div>';
    this.path = q<HTMLInputElement>(host, '.ff-bd-path');
    this.go = q<HTMLButtonElement>(host, '.ff-bd-go');
    repaint(() => { const s = t('Body file path'); if (this.path.placeholder !== s) this.path.placeholder = s; this.path.setAttribute('aria-label', s); });

    const row = q(host, '.ff-bd-pathrow');
    if (nativeDialogs()) {
      const browse = iconButton('folderOpen', 'Browse', 'ff-bd-browse');
      browse.addEventListener('click', (e) => { void this.browse(); if (e.detail > 0) browse.blur(); });
      row.appendChild(browse);
    }
    const units = selectRow('Units', UNITS, true, (v) => { this.units = v; });
    const up = selectRow('Up axis', AXES, false, (v) => { this.up = v; });
    host.insertBefore(units.el, q(host, '.ff-bd-actions'));
    host.insertBefore(up.el, q(host, '.ff-bd-actions'));
    this.go.innerHTML = `${icon('fileIn', 14)}<span></span>`;
    label(this.go.children[1] as HTMLElement, 'Import Body');
    this.go.addEventListener('click', (e) => { void this.run(); if (e.detail > 0) this.go.blur(); });
    this.path.addEventListener('keydown', (e) => { if (e.key === 'Enter') { e.preventDefault(); this.go.click(); } });
  }

  private async browse(): Promise<void> {
    const dialog = await import('@tauri-apps/plugin-dialog');
    const chosen = await dialog.open({ multiple: false, directory: false, filters: [{ name: t('Body'), extensions: IMPORT_EXT }] });
    if (typeof chosen === 'string') this.path.value = chosen;
  }

  private async run(): Promise<void> {
    const path = this.path.value.trim();
    if (!path) { flagBad(this.path, t('Body file path'), () => undefined); return; }
    const r = await this.ctx.store.run<BodyInfo>('body.import', { path, units: this.units, up: this.up });
    if (r) this.ctx.store.select(r.id);
  }
}
