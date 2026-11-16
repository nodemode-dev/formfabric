import type { ShellDeps } from '../../shell/deps';
import { promptName } from '../../ui/dialogs';
import { el, q, setClass, setDisabled, setHidden, setText, setTitle } from '../../ui/dom';
import { icon } from '../../ui/icons';
import { t } from '../../ui/i18n';
import type { DocStore } from '../doc';
import { label, patchKeyed, titled } from '../fabric/widgets';
import { OFFLINE, engineDown, onEngineDown } from '../offline';
import type { PatternView } from '../pattern-view/view';
import type { Selection } from '../state';
import { hasCommand, onCommands, run } from '../viz/api';
import { SCENE_KINDS, TABS, loadFabric, loadGraphics, loadScene, loadStyles, sorted, type Row, type TabId } from './api';
import './icons';
import './objects.css';

const OPEN_KEY = 'formfabric.objects.open';
const TAB_KEY = 'formfabric.objects.tab';
const IGNORED = ['sim.', 'view.', 'anim.', 'cache.', 'viz.', 'capture.'];
const STYLE_TABS: string[] = ['button', 'buttonhole', 'topstitch', 'puckering', 'trim', 'zipper'];

const HTML =
  '<header class="ff-ob-head"><span class="ff-ob-title" data-k="title"></span><button class="cv-icon-btn" type="button" data-k="close"></button></header>' +
  `<div class="ff-ob-tabs" role="tablist">${TABS.map(([id]) => `<button class="ff-ob-tab" type="button" role="tab" data-tab="${id}"></button>`).join('')}</div>` +
  '<div class="ff-ob-bar">' +
  '<button class="cv-icon-btn" type="button" data-k="add"></button><button class="cv-icon-btn" type="button" data-k="copy"></button><button class="cv-icon-btn" type="button" data-k="merge"></button>' +
  '<span class="ff-ob-gap"></span><button class="cv-icon-btn" type="button" data-k="up"></button><button class="cv-icon-btn" type="button" data-k="down"></button></div>' +
  '<div class="ff-ob-cols"><button class="ff-ob-sort" type="button" data-k="sort" data-dir="0" data-sortable="0"></button></div>' +
  '<div class="ff-ob-list" data-k="list" tabindex="0" role="listbox" aria-multiselectable="true"><p class="ff-ob-note" data-k="note"></p></div>' +
  '<div class="ff-ob-foot" data-k="foot" role="status"></div>';

function read(key: string): string | null {
  try { return localStorage.getItem(key); } catch { return null; }
}

function write(key: string, value: string): void {
  try { localStorage.setItem(key, value); } catch { /* this session only */ }
}

export class ObjectsDrawer {
  readonly drawer: HTMLElement;
  private readonly toggle: HTMLButtonElement;
  private readonly tabs = new Map<TabId, HTMLButtonElement>();
  private readonly list: HTMLElement;
  private readonly note: HTMLElement;
  private readonly foot: HTMLElement;
  private readonly sortBtn: HTMLButtonElement;
  private readonly btn: Record<'add' | 'copy' | 'merge' | 'up' | 'down', HTMLButtonElement>;
  private rows: Row[] = [];
  private shown: Row[] = [];
  private supported = true;
  private picked = new Set<string>();
  private anchor: string | null = null;
  private dir: 0 | 1 | -1 = 0;
  private tab: TabId = 'scene';
  private isOpen = false;
  private yielding = false;
  private seq = 0;
  private timer = 0;

  constructor(root: HTMLElement, private readonly deps: ShellDeps, private readonly doc: DocStore, private readonly sel: Selection, private readonly view: PatternView) {
    this.toggle = el<HTMLButtonElement>(`<button class="cv-icon-btn ff-ob-toggle" type="button" aria-expanded="false">${icon('objects', 18)}</button>`);
    titled(this.toggle, 'Object Browser');
    const zoom = q(root, '[data-k="zoom"]');
    zoom.parentElement?.insertBefore(this.toggle, zoom);

    this.drawer = el('<aside class="ff-ob" data-k="objects" hidden></aside>');
    this.drawer.innerHTML = HTML;
    q(root, '.ff-body').appendChild(this.drawer);
    label(q(this.drawer, '[data-k="title"]'), 'Object Browser');
    for (const [id, text] of TABS) {
      const b = q<HTMLButtonElement>(this.drawer, `[data-tab="${id}"]`);
      label(b, text);
      this.tabs.set(id, b);
      b.addEventListener('click', () => { this.setTab(id); b.blur(); });
    }
    const close = q<HTMLButtonElement>(this.drawer, '[data-k="close"]');
    close.innerHTML = icon('close', 16);
    titled(close, 'Close');
    close.addEventListener('click', () => this.setOpen(false));
    this.toggle.addEventListener('click', (e) => { this.setOpen(!this.isOpen); if (e.detail > 0) this.toggle.blur(); });

    const k = <T extends HTMLElement>(n: string): T => q<T>(this.drawer, `[data-k="${n}"]`);
    this.list = k('list');
    this.note = k('note');
    this.foot = k('foot');
    this.sortBtn = k<HTMLButtonElement>('sort');
    label(this.sortBtn, 'Name');
    this.btn = { add: k('add'), copy: k('copy'), merge: k('merge'), up: k('up'), down: k('down') };
    const icons: Array<[keyof typeof this.btn, string, string]> = [['add', 'plus', 'Add'], ['copy', 'obCopy', 'Copy'], ['merge', 'obMerge', 'Merge'], ['up', 'obUp', 'Move up'], ['down', 'obDown', 'Move down']];
    for (const [id, ico, text] of icons) {
      this.btn[id].innerHTML = icon(ico, 16);
      titled(this.btn[id], text);
    }
    this.btn.add.addEventListener('click', () => void this.add());
    this.btn.copy.addEventListener('click', () => void this.copy());
    this.btn.merge.addEventListener('click', () => void this.merge());
    this.btn.up.addEventListener('click', () => void this.move(-1));
    this.btn.down.addEventListener('click', () => void this.move(1));
    this.sortBtn.addEventListener('click', () => {
      if (this.tab !== 'scene') return;
      this.dir = this.dir === 1 ? -1 : 1;
      this.paint();
    });
    this.list.addEventListener('click', (e) => this.onClick(e));
    this.list.addEventListener('keydown', (e) => this.onKey(e));

    document.addEventListener('click', (e) => {
      if (!this.isOpen || this.yielding || !(e.target instanceof Element)) return;
      const other = e.target.closest<HTMLElement>('.ff-fb-toggle, .ff-bd-toggle, .ff-tr-toggle, .ou-toggle-btn, .ff-lib-toggle');
      if (other && other !== this.toggle) this.setOpen(false);
    }, true);
    deps.engine.onEngineChanged((e) => {
      if (this.isOpen && !IGNORED.some((p) => e.name.startsWith(p))) this.schedule();
    });
    doc.onChange(() => { if (this.isOpen && this.tab === 'graphic') this.schedule(); });
    onCommands(() => this.gate());
    onEngineDown(() => this.paint());
    deps.commands.add({ id: 'app.objects', label: 'Object Browser', group: 'Window', icon: 'objects', run: () => this.setOpen(!this.isOpen), checked: () => this.isOpen });

    const saved = read(TAB_KEY);
    if (TABS.some((x) => x[0] === saved)) this.tab = saved as TabId;
    this.paintTabs();
    this.gate();
    if (read(OPEN_KEY) === '1') this.setOpen(true);
  }

  private paintTabs(): void {
    for (const [id, b] of this.tabs) {
      setClass(b, 'is-on', id === this.tab);
      b.setAttribute('aria-selected', String(id === this.tab));
    }
    const sortable = this.tab === 'scene';
    this.sortBtn.dataset.sortable = sortable ? '1' : '0';
    setTitle(this.sortBtn, sortable ? t('Sort by name') : '');
  }

  private setTab(id: TabId): void {
    if (id === this.tab) return;
    this.tab = id;
    write(TAB_KEY, id);
    this.picked = new Set();
    this.anchor = null;
    this.paintTabs();
    void this.reload();
  }

  setOpen(on: boolean): void {
    if (on === this.isOpen) return;
    this.isOpen = on;
    write(OPEN_KEY, on ? '1' : '0');
    setClass(this.toggle, 'is-active', on);
    this.toggle.setAttribute('aria-expanded', String(on));
    if (on) {
      this.yielding = true;
      for (const other of document.querySelectorAll<HTMLElement>('.ff-body > aside[data-k]:not([hidden])')) {
        if (other !== this.drawer) other.querySelector<HTMLButtonElement>('[data-k="close"]')?.click();
      }
      this.yielding = false;
    }
    setHidden(this.drawer, !on);
    if (on) void this.reload();
  }

  private schedule(): void {
    window.clearTimeout(this.timer);
    this.timer = window.setTimeout(() => void this.reload(), 160);
  }

  private async reload(): Promise<void> {
    if (!this.isOpen) return;
    const id = ++this.seq;
    const tab = this.tab;
    let rows: Row[] | undefined;
    if (tab === 'scene') rows = await loadScene();
    else if (tab === 'fabric') rows = await loadFabric();
    else if (tab === 'graphic') rows = loadGraphics(this.doc.garment);
    else rows = await loadStyles(tab);
    if (id !== this.seq) return;
    this.supported = rows !== undefined;
    this.rows = rows ?? [];
    this.paint();
  }

  private paint(): void {
    const live = new Set(this.rows.map((r) => r.key));
    for (const key of Array.from(this.picked)) if (!live.has(key)) this.picked.delete(key);
    this.shown = this.tab === 'scene' ? sorted(this.rows, this.dir) : this.rows;
    patchKeyed(this.list, this.shown, {
      key: (r) => r.key,
      create: () => el('<div class="ff-ob-row" role="option"><span class="ff-ob-sw"></span><span class="ff-ob-name"></span><span class="ff-ob-sub"></span></div>'),
      update: (node, r) => {
        const sw = node.firstElementChild as HTMLElement;
        if (sw.dataset.empty !== (r.swatch ? '0' : '1')) sw.dataset.empty = r.swatch ? '0' : '1';
        if (sw.style.getPropertyValue('--sw') !== r.swatch) sw.style.setProperty('--sw', r.swatch);
        setText(node.children[1], r.name);
        setText(node.children[2], this.tab === 'scene' ? t(SCENE_KINDS[r.sub] ?? r.sub) : r.sub);
        const on = this.picked.has(r.key);
        setClass(node, 'is-sel', on);
        if (node.getAttribute('aria-selected') !== String(on)) node.setAttribute('aria-selected', String(on));
      },
    });
    this.sortBtn.dataset.dir = this.tab === 'scene' ? String(this.dir) : '0';
    const empty = this.shown.length === 0;
    setHidden(this.note, !empty);
    setText(this.note, empty ? t(engineDown() ? OFFLINE : this.supported ? 'Nothing here yet' : 'The engine does not provide this list') : '');
    setText(this.foot, `${this.shown.length} ${t('items')}${this.picked.size ? `, ${this.picked.size} ${t('selected')}` : ''}`);
    this.gate();
  }

  private gate(): void {
    const fabric = this.tab === 'fabric';
    const style = STYLE_TABS.includes(this.tab);
    const n = this.picked.size;
    const editable = fabric || style;
    const prefix = fabric ? 'fabric' : 'style';
    const has = (c: string): boolean => hasCommand(`${prefix}.${c}`);
    const ids = this.shown.map((r) => r.key);
    const at = ids.map((key, i) => (this.picked.has(key) ? i : -1)).filter((i) => i >= 0);
    setDisabled(this.btn.add, !(editable && has('create')));
    setDisabled(this.btn.copy, !(editable && n >= 1 && has('copy')));
    setDisabled(this.btn.merge, !(editable && n >= 2 && has('merge')));
    const order = fabric && hasCommand('fabric.order') && at.length > 0;
    setDisabled(this.btn.up, !(order && at[0] > 0));
    setDisabled(this.btn.down, !(order && at[at.length - 1] < ids.length - 1));
    this.btn.up.style.visibility = fabric ? 'visible' : 'hidden';
    this.btn.down.style.visibility = fabric ? 'visible' : 'hidden';
  }

  private onClick(e: MouseEvent): void {
    const row = e.target instanceof Element ? e.target.closest<HTMLElement>('.ff-ob-row') : null;
    const key = row?.dataset.key;
    if (!key) return;
    this.choose(key, e.shiftKey, e.ctrlKey || e.metaKey);
  }

  private choose(key: string, range: boolean, toggle: boolean): void {
    const keys = this.shown.map((r) => r.key);
    if (range && this.anchor && keys.includes(this.anchor)) {
      const a = keys.indexOf(this.anchor);
      const b = keys.indexOf(key);
      this.picked = new Set(keys.slice(Math.min(a, b), Math.max(a, b) + 1));
    } else if (toggle) {
      if (this.picked.has(key)) this.picked.delete(key); else this.picked.add(key);
      this.anchor = key;
    } else {
      this.picked = new Set([key]);
      this.anchor = key;
    }
    this.syncRows();
    const panel = this.picked.size === 1 ? this.shown.find((r) => r.key === key)?.panel ?? null : null;
    if (panel !== null && !(this.sel.panels.size === 1 && this.sel.panels.has(panel))) {
      this.sel.only(panel);
      this.view.selected();
    }
  }

  private syncRows(): void {
    for (const node of Array.from(this.list.children) as HTMLElement[]) {
      const key = node.dataset.key;
      if (key === undefined) continue;
      const on = this.picked.has(key);
      setClass(node, 'is-sel', on);
      if (node.getAttribute('aria-selected') !== String(on)) node.setAttribute('aria-selected', String(on));
    }
    setText(this.foot, `${this.shown.length} ${t('items')}${this.picked.size ? `, ${this.picked.size} ${t('selected')}` : ''}`);
    this.gate();
  }

  private onKey(e: KeyboardEvent): void {
    if (e.key !== 'ArrowDown' && e.key !== 'ArrowUp') return;
    e.preventDefault();
    const keys = this.shown.map((r) => r.key);
    if (keys.length === 0) return;
    const at = this.anchor ? keys.indexOf(this.anchor) : -1;
    const next = Math.max(0, Math.min(keys.length - 1, at + (e.key === 'ArrowDown' ? 1 : -1)));
    this.choose(keys[next], e.shiftKey, false);
    this.list.querySelector<HTMLElement>(`[data-key="${CSS.escape(keys[next])}"]`)?.scrollIntoView({ block: 'nearest' });
  }

  private chosen(): Row[] {
    return this.shown.filter((r) => this.picked.has(r.key));
  }

  private async add(): Promise<void> {
    if (this.tab === 'fabric') await run('fabric.create', {});
    else if (STYLE_TABS.includes(this.tab)) await run('style.create', { kind: this.tab });
    await this.reload();
  }

  private async copy(): Promise<void> {
    const command = this.tab === 'fabric' ? 'fabric.copy' : 'style.copy';
    for (const r of this.chosen()) await run(command, { id: r.id });
    await this.reload();
  }

  private async merge(): Promise<void> {
    const rows = this.chosen();
    if (rows.length < 2) return;
    const name = await promptName(t('Merged name'), `${rows[0].name} ${t('merged')}`, t('Merge'));
    if (!name) return;
    const command = this.tab === 'fabric' ? 'fabric.merge' : 'style.merge';
    const r = await run<unknown>(command, { ids: rows.map((x) => x.id), name });
    if (r !== undefined) this.picked = new Set();
    await this.reload();
  }

  private async move(step: -1 | 1): Promise<void> {
    if (this.tab !== 'fabric') return;
    const ids = this.shown.map((r) => r.id);
    const keys = this.shown.map((r) => r.key);
    const order = keys.map((k, i) => (this.picked.has(k) ? i : -1)).filter((i) => i >= 0);
    if (order.length === 0) return;
    const walk = step < 0 ? order : order.slice().reverse();
    for (const i of walk) {
      const j = i + step;
      if (j < 0 || j >= ids.length || this.picked.has(keys[j])) continue;
      [ids[i], ids[j]] = [ids[j], ids[i]];
      [keys[i], keys[j]] = [keys[j], keys[i]];
    }
    await run('fabric.order', { ids });
    await this.reload();
  }
}
