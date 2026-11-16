import type { ShellDeps } from '../../shell/deps';
import { el, q, setClass, setHidden } from '../../ui/dom';
import { icon } from '../../ui/icons';
import { ComponentsTab } from '../components/panel';
import type { DocStore } from '../doc';
import { label, titled } from '../fabric/widgets';
import type { PatternView } from '../pattern-view/view';
import { SnapshotsTab } from '../snapshots/panel';
import { quiet } from '../viz/api';
import type { Selection } from '../state';
import { LibraryTab } from './browser';
import './icons';

const OPEN_KEY = 'formfabric.library.open';
const TAB_KEY = 'formfabric.library.tab';
const TABS = [['library', 'Library', 'libBrowse'], ['snapshots', 'Snapshots', 'libSnapshot'], ['components', 'Components', 'libComponent']] as const;
type TabId = (typeof TABS)[number][0];

const HTML =
  '<header class="ff-lib-head"><div class="ff-lib-tabs" role="tablist">' +
  TABS.map(([id, , ico]) => `<button class="ff-lib-tab" type="button" role="tab" data-tab="${id}">${icon(ico, 14)}<span data-k="t-${id}"></span></button>`).join('') +
  '</div><button class="cv-icon-btn" type="button" data-k="close"></button></header>';

export class LibraryDrawer {
  readonly drawer: HTMLElement;
  private readonly toggle: HTMLButtonElement;
  private readonly tabs = new Map<TabId, HTMLButtonElement>();
  private readonly library: LibraryTab;
  private readonly snaps: SnapshotsTab;
  private readonly comps: ComponentsTab;
  private isOpen = false;
  private tab: TabId = 'library';
  private yielding = false;

  constructor(root: HTMLElement, private readonly deps: ShellDeps, doc: DocStore, sel: Selection, view: PatternView) {
    this.toggle = el<HTMLButtonElement>(`<button class="cv-icon-btn ff-lib-toggle" type="button" aria-expanded="false">${icon('libBrowse', 18)}</button>`);
    titled(this.toggle, 'Library');
    const zoom = q(root, '[data-k="zoom"]');
    zoom.parentElement?.insertBefore(this.toggle, zoom);

    this.drawer = el('<aside class="ff-lib" data-k="library" hidden></aside>');
    this.drawer.innerHTML = HTML;
    q(root, '.ff-body').appendChild(this.drawer);
    for (const [id, text] of TABS) {
      label(q(this.drawer, `[data-k="t-${id}"]`), text);
      const b = q<HTMLButtonElement>(this.drawer, `[data-tab="${id}"]`);
      this.tabs.set(id, b);
      b.addEventListener('click', () => { this.setTab(id); b.blur(); });
    }
    const close = q<HTMLButtonElement>(this.drawer, '[data-k="close"]');
    close.innerHTML = icon('close', 16);
    titled(close, 'Close');
    close.addEventListener('click', () => this.setOpen(false));
    this.toggle.addEventListener('click', (e) => { this.setOpen(!this.isOpen); if (e.detail > 0) this.toggle.blur(); });

    this.library = new LibraryTab(this.drawer);
    this.snaps = new SnapshotsTab(this.drawer);
    this.comps = new ComponentsTab(this.drawer, q(root, '[data-k="pane2d"]'), doc, sel, view, () => deps.prefs().units);
    this.library.fromSelection = (kind) => this.sourceFor(kind);

    document.addEventListener('click', (e) => {
      if (!this.isOpen || this.yielding || !(e.target instanceof Element)) return;
      const other = e.target.closest<HTMLElement>('.ff-fb-toggle, .ff-bd-toggle, .ff-tr-toggle, .ou-toggle-btn');
      if (other && other !== this.toggle) this.setOpen(false);
    }, true);
    for (const id of ['pane2d', 'pane3d']) {
      const pane = q(root, `[data-k="${id}"]`);
      pane.addEventListener('dragover', (e) => { if (e.dataTransfer?.types.includes('application/x-ff-library')) { e.preventDefault(); e.dataTransfer.dropEffect = 'copy'; } });
      pane.addEventListener('drop', (e) => {
        const key = e.dataTransfer?.getData('application/x-ff-library');
        if (!key) return;
        e.preventDefault();
        void this.library.insert(key);
      });
    }
    deps.engine.onEngineChanged((e) => {
      if (!this.isOpen) return;
      if (e.name.startsWith('snapshot.')) void this.snaps.refresh();
      else if (e.name.startsWith('component.')) void this.comps.refresh();
      else if (e.name.startsWith('library.')) void this.library.reload();
    });

    const saved = localStorage.getItem(TAB_KEY);
    if (TABS.some((x) => x[0] === saved)) this.tab = saved as TabId;
    this.paintTabs();
    this.registerCommands();
    if (localStorage.getItem(OPEN_KEY) === '1') this.setOpen(true);
  }

  private async sourceFor(kind: string): Promise<Record<string, unknown> | null> {
    const pick = (v: unknown): Record<string, unknown>[] => {
      const o = v && typeof v === 'object' ? (v as Record<string, unknown>) : {};
      const l = Array.isArray(v) ? v : (o.styles ?? o.items ?? o.fabrics ?? o.motions);
      return Array.isArray(l) ? (l as Record<string, unknown>[]) : [];
    };
    if (kind === 'component') { const id = this.comps.fromSelection(); return id ? { from: id } : null; }
    if (kind === 'fabric') {
      const row = document.querySelector<HTMLElement>('.ff-fabric[data-k="fabric"] .ff-fb-item.is-sel');
      if (row?.dataset.key) return { from: Number(row.dataset.key) };
      const r = await quiet<Record<string, unknown>>('fabric.list', {});
      const id = r?.default_fabric ?? pick(r?.fabrics ?? r)[0]?.id;
      return id === undefined ? null : { from: id };
    }
    if (kind === 'style' || kind === 'trim') {
      const all = pick(await quiet<unknown>('style.list', {}));
      const hit = all.find((s) => (kind === 'trim') === (s.kind === 'trim')) ?? all[0];
      return hit?.id === undefined ? null : { from: hit.id };
    }
    if (kind === 'motion') {
      const a = await quiet<Record<string, unknown>>('anim.get', {});
      const m = pick(a?.motions)[0];
      return m?.id === undefined ? null : { from: m.id };
    }
    return {};
  }

  private registerCommands(): void {
    const { commands } = this.deps;
    const open = (tab: TabId): void => { if (this.isOpen && this.tab === tab) this.setOpen(false); else { this.setTab(tab); this.setOpen(true); } };
    commands.add(
      { id: 'app.library', label: 'Library', group: 'Window', icon: 'libBrowse', run: () => open('library'), checked: () => this.isOpen && this.tab === 'library' },
      { id: 'app.library_snapshots', label: 'Snapshots', group: 'Window', icon: 'libSnapshot', run: () => open('snapshots'), checked: () => this.isOpen && this.tab === 'snapshots' },
      { id: 'app.library_components', label: 'Components', group: 'Window', icon: 'libComponent', run: () => open('components'), checked: () => this.isOpen && this.tab === 'components' },
    );
  }

  private paintTabs(): void {
    for (const [id, b] of this.tabs) { setClass(b, 'is-on', id === this.tab); b.setAttribute('aria-selected', String(id === this.tab)); }
    setHidden(q(this.drawer, '[data-tab="library"].ff-lib-pane'), this.tab !== 'library');
    setHidden(q(this.drawer, '[data-tab="snapshots"].ff-lib-pane'), this.tab !== 'snapshots');
    setHidden(q(this.drawer, '[data-tab="components"].ff-lib-pane'), this.tab !== 'components');
  }

  private setTab(id: TabId): void {
    if (id === this.tab) return;
    if (this.tab === 'components') this.comps.hide();
    this.tab = id;
    localStorage.setItem(TAB_KEY, id);
    this.paintTabs();
    this.shown();
  }

  private shown(): void {
    if (!this.isOpen) return;
    if (this.tab === 'library') this.library.show();
    else if (this.tab === 'snapshots') this.snaps.show();
    else this.comps.show();
  }

  setOpen(on: boolean): void {
    if (on === this.isOpen) return;
    this.isOpen = on;
    localStorage.setItem(OPEN_KEY, on ? '1' : '0');
    setClass(this.toggle, 'is-active', on);
    this.toggle.setAttribute('aria-expanded', String(on));
    if (on) {
      this.yielding = true;
      for (const other of document.querySelectorAll<HTMLElement>('.ff-body > aside[data-k]:not([hidden])')) {
        if (other !== this.drawer) other.querySelector<HTMLButtonElement>('[data-k="close"]')?.click();
      }
      this.yielding = false;
    } else this.comps.hide();
    setHidden(this.drawer, !on);
    this.shown();
  }
}
