import './home.css';
import { brandMark } from '../../ui/brand';
import { DEMOS, type DemoId } from '../../ui/demos';
import { Dialog, anyDialogOpen } from '../../ui/dialogs';
import { el, escapeHtml, patchList, setClass, setHidden, setText, setTitle } from '../../ui/dom';
import { icon } from '../../ui/icons';
import { language, onLanguage, t as say } from '../../ui/i18n';
import { toast } from '../../ui/toast';
import { api, garmentThumb, picture, templateThumb, type GarmentSummary, type TemplateSummary, type TrashedGarment } from './api';
import { watchSecondService } from './second';

export interface HomeHooks {
  openGarment(garment: GarmentSummary): void;
  closeGarment(path: string): void;
  createGarment(template: string, title?: string): Promise<void>;
  blankGarment(): void;
  openFile(): void;
  chooseFile(): boolean;
  openFiles(files: File[]): void;
  openPaths(paths: string[]): void;
  recentChanged(): void;
  preferences(): void;
  agents(): void;
  demo(id: DemoId): void;
  hasGarment(): boolean;
  autoShow(): boolean;
  afterClose(): boolean;
  onVisible?(open: boolean): void;
}

export type HomePage = 'start' | 'recent' | 'templates' | 'trash';

const NAV_KEY = 'formfabric.homeNav';
const NAV_HIDDEN_KEY = 'formfabric.homeNavHidden';
const ATTRS = ['title', 'aria-label', 'placeholder'] as const;
const ALL = 'All';
const spoken = new Map<string, string>();
const t = (english: string): string => { const shown = say(english); spoken.set(shown, english); return shown; };
const message = (e: unknown): string => (e instanceof Error ? e.message : String(e));
const failed = (e: unknown): void => { toast(t('That did not work: {error}').replace('{error}', message(e)), 'error'); };
const folderOf = (path: string): string => path.replace(/[\\/][^\\/]*$/, '');
const TPL_SKELETON = [0, 1, 2, 3].map((i) => `<button class="cv-card is-skeleton" type="button" tabindex="-1" data-key="__skeleton${i}" aria-hidden="true"><span class="cv-card-preview"></span><span class="cv-card-body"><span class="cv-card-name">&nbsp;</span><span class="cv-card-meta">&nbsp;</span></span></button>`).join('');

function ago(iso: string): string {
  const at = Date.parse(iso);
  if (!Number.isFinite(at)) return '';
  const s = (Date.now() - at) / 1000;
  if (s < 60) return t('just now');
  if (s < 3600) return t('{n} min ago').replace('{n}', String(Math.round(s / 60)));
  if (s < 86400) return t('{n} h ago').replace('{n}', String(Math.round(s / 3600)));
  try { return new Date(at).toLocaleDateString(language(), { month: 'short', day: 'numeric' }); } catch { return new Date(at).toLocaleDateString(); }
}

class TemplateDialog extends Dialog {
  private tpl: TemplateSummary | null = null;
  private readonly name: HTMLInputElement;

  constructor(private readonly hooks: HomeHooks) {
    super('New garment from a template', 420);
    this.name = el<HTMLInputElement>('<input class="cv-input" type="text" spellcheck="false" maxlength="80">');
    this.field('Name', this.name);
    this.button('Cancel', 'ghost', () => this.close());
    this.button('Use template', 'primary', () => this.use());
    this.name.addEventListener('keydown', (e) => { if (e.key === 'Enter') { e.preventDefault(); this.use(); } });
  }

  private use(): void {
    const tpl = this.tpl;
    if (!tpl) return;
    this.close();
    void this.hooks.createGarment(tpl.id, this.name.value.trim() || undefined);
  }

  show(tpl: TemplateSummary): void {
    this.tpl = tpl;
    this.name.value = say(tpl.name);
    this.open(this.name);
    this.name.select();
  }
}

export class Home {
  readonly root: HTMLElement;
  private view: HomePage = 'start';
  private templates: TemplateSummary[] = [];
  private tplReady = false;
  private categories: string[] = [ALL];
  private category = ALL;
  private query = '';
  private garments: GarmentSummary[] = [];
  private trashed: TrashedGarment[] = [];
  private pinned: string[] = [];
  private hiddenRecent: string[] = [];
  private loads = 0;
  private forced = false;
  private hadGarment = false;
  private lastHas = false;
  private readonly selected = new Set<string>();
  private anchor = '';
  private onConfirm: (() => void) | null = null;
  private readonly tplGrid: HTMLElement;
  private readonly tplGridFull: HTMLElement;
  private readonly recentGrid: HTMLElement;
  private readonly recentGridFull: HTMLElement;
  private readonly trashList: HTMLElement;
  private readonly confirmBar: HTMLElement;
  private readonly menu: HTMLElement;
  private readonly back: HTMLElement;
  private readonly preview: TemplateDialog;
  private readonly labels: Array<{ node: Text | Element; attr: string; en: string }> = [];

  constructor(host: HTMLElement, private readonly hooks: HomeHooks) {
    const demos = DEMOS.length ? `<section class="cv-home-section" data-section="demos"><div class="cv-home-section-head"><h2 class="cv-home-section-title">${t('Demos')}</h2><span class="cv-card-meta">${t('Each opens as a new garment in your library')}</span></div><div class="cv-card-grid cv-demo-grid">${DEMOS.map((d) => `<button class="cv-card" type="button" data-demo="${escapeHtml(d.id)}"><span class="cv-card-preview" aria-hidden="true"><img alt="" decoding="async" draggable="false" src="${escapeHtml(d.picture)}"></span><span class="cv-card-body"><span class="cv-card-name" translate="no">${escapeHtml(d.title)}</span><span class="cv-card-meta" title="${escapeHtml(d.line)}">${escapeHtml(d.line)}</span></span></button>`).join('')}</div></section>` : '';
    this.root = el(`<section class="cv-home" aria-label="${t('Home')}" tabindex="-1" hidden>
<div class="cv-home-drag" data-tauri-drag-region></div>
<nav class="cv-home-nav">
<div class="cv-home-brand">${brandMark(26)}formfabric <span translate="no">by nodemode.ai</span></div>
<button class="cv-home-nav-item is-active" type="button" data-v="start">${icon('panel', 16)}<span>${t('Start')}</span></button>
<button class="cv-home-nav-item" type="button" data-v="recent">${icon('history', 16)}<span>${t('Recent garments')}</span><span class="cv-badge is-count cv-garment-count">0</span></button>
<button class="cv-home-nav-item" type="button" data-v="templates">${icon('template', 16)}<span>${t('Templates')}</span><span class="cv-badge is-count cv-tpl-count">0</span></button>
<button class="cv-home-nav-item" type="button" data-v="trash">${icon('delete', 16)}<span>${t('Trash')}</span><span class="cv-badge is-count cv-trash-count">0</span></button>
<div class="cv-home-open" hidden><div class="cv-home-nav-sep"></div><div class="cv-home-nav-title">${t('Open garment')}</div><div class="cv-home-open-list"></div></div>
<div class="cv-home-nav-sep"></div>
<div class="cv-home-nav-title">${t('Agents')}</div>
<button class="cv-home-nav-item" type="button" data-a="agents">${icon('agent', 16)}<span translate="no">Claude Code, Codex, Cursor</span></button>
<div class="cv-home-nav-foot">
<button class="cv-home-nav-item" type="button" data-a="back" hidden>${icon('chevronLeft', 16)}<span>${t('Back to Workspace')}</span></button>
<button class="cv-home-nav-item" type="button" data-a="local" title="${t('Your garments, fabrics and bodies stay on this computer.')}">${icon('cloudOff', 16)}<span>${t('Local only')}</span></button>
<button class="cv-home-nav-item" type="button" data-a="prefs">${icon('settings', 16)}<span>${t('Preferences')}</span></button>
</div></nav>
<div class="cv-home-resizer" role="separator" aria-orientation="vertical" aria-label="${t('Resize the side panel')}"></div>
<button class="cv-home-nav-hide" type="button" title="${t('Hide the side panel')}" aria-label="${t('Hide the side panel')}">${icon('chevronLeft', 14)}</button>
<button class="cv-home-nav-show" type="button" title="${t('Show the side panel')}" aria-label="${t('Show the side panel')}">${icon('chevronRight', 14)}</button>
<div class="cv-home-main">
<div class="cv-home-narrow"><button class="cv-home-nav-item" type="button" data-v="start">${icon('panel', 16)}<span>${t('Start')}</span></button><button class="cv-home-nav-item" type="button" data-v="recent">${icon('history', 16)}<span>${t('Recent')}</span></button><button class="cv-home-nav-item" type="button" data-v="templates">${icon('template', 16)}<span>${t('Templates')}</span></button><button class="cv-home-nav-item" type="button" data-v="trash">${icon('delete', 16)}<span>${t('Trash')}</span></button></div>
<div class="cv-home-confirm" role="alertdialog" aria-live="assertive" hidden><span class="cv-home-confirm-text"></span><button class="cv-btn cv-btn-sm" type="button" data-a="confirm-no">${t('Cancel')}</button><button class="cv-btn cv-btn-sm cv-btn-danger" type="button" data-a="confirm-yes"></button></div>
<div data-page="start">
<div class="cv-home-head"><div><h1 class="cv-home-title"></h1><p class="cv-home-sub">${t('Open a garment, bring in a pattern, or start from a template.')}</p></div><div class="cv-home-head-actions"><button class="cv-btn" type="button" data-a="open">${icon('folderOpen', 14)}${t('Open file')}</button><button class="cv-btn" type="button" data-a="blank">${icon('newFile', 14)}${t('Blank garment')}</button></div></div>
<div class="cv-home-hero">
<span class="cv-home-hero-icon">${icon('inbox', 26)}</span>
<div class="cv-home-hero-text"><div class="cv-home-hero-title">${t('Start from your pattern')}</div>
<div class="cv-home-hero-sub">${t('Drop a garment project or a DXF pattern here, or choose a file. FormFabric opens it, ready to sew, drape and fit on a body. Your agent can take it from there.')}</div>
<div class="cv-home-formats" translate="no"><span class="cv-home-format">FFBRC</span><span class="cv-home-format">DXF-AAMA</span><span class="cv-home-format">DXF</span></div></div>
<div class="cv-home-hero-actions"><button class="cv-btn cv-btn-primary" type="button" data-a="choose">${icon('upload', 14)}${t('Choose a file')}</button><button class="cv-btn cv-btn-ghost" type="button" data-a="agents">${icon('agent', 14)}${t('Use your agent')}</button></div>
</div>
<section class="cv-home-section"><div class="cv-home-section-head"><h2 class="cv-home-section-title">${t('Recent garments')}</h2><button class="cv-btn cv-btn-ghost cv-btn-sm" type="button" data-v="recent">${t('See all')}</button></div><div class="cv-card-grid cv-recent-grid"><button class="cv-card is-skeleton" type="button" tabindex="-1" data-key="__skeleton" aria-hidden="true"><span class="cv-card-preview"></span><span class="cv-card-body"><span class="cv-card-name">&nbsp;</span><span class="cv-card-meta">&nbsp;</span></span></button><button class="cv-card is-empty" type="button" tabindex="-1" hidden><span class="cv-card-preview is-blank">${icon('panel', 28)}</span><span class="cv-card-body"><span class="cv-card-name">${t('No garments yet')}</span><span class="cv-card-meta">${t('Start from your pattern or a template')}</span></span></button></div></section>
${demos}
<section class="cv-home-section"><div class="cv-home-section-head"><h2 class="cv-home-section-title">${t('Templates')}</h2><div class="cv-home-filters cv-cats"></div><label class="cv-search">${icon('search', 14)}<input type="search" placeholder="${t('Search templates')}" class="cv-tpl-search"></label></div><div class="cv-card-grid cv-tpl-grid">${TPL_SKELETON}</div></section>
</div>
<div data-page="recent" hidden><div class="cv-home-head"><div><h1 class="cv-home-title">${t('Recent garments')}</h1><p class="cv-home-sub">${t('Everything in your library and the files you opened, newest first.')}</p></div></div><div class="cv-card-grid cv-recent-full"></div><p class="cv-note cv-recent-none" hidden><span>${t('No garments yet')}</span>. <span>${t('Start from your pattern or a template')}</span>.</p></div>
<div data-page="trash" hidden><div class="cv-home-head"><div><h1 class="cv-home-title">${t('Trash')}</h1><p class="cv-home-sub">${t('Deleted garments wait here until you restore them or delete them permanently.')}</p></div><div class="cv-home-head-actions"><button class="cv-btn cv-btn-empty-trash" type="button" data-a="empty-trash" disabled>${icon('delete', 14)}${t('Empty Trash')}</button></div></div><div class="cv-trash-list"></div><p class="cv-note cv-trash-none" hidden>${t('Trash is empty.')}</p></div>
<div data-page="templates" hidden><div class="cv-home-head"><div><h1 class="cv-home-title">${t('Templates')}</h1><p class="cv-home-sub">${t('The garments that ship with FormFabric. Each one opens as a new garment in your library.')}</p></div></div>
<div class="cv-home-section-head"><div class="cv-home-filters cv-cats"></div><label class="cv-search">${icon('search', 14)}<input type="search" placeholder="${t('Search templates')}" class="cv-tpl-search"></label></div><div class="cv-card-grid cv-tpl-full">${TPL_SKELETON}</div></div>
</div></section>`);
    this.tplGrid = this.root.querySelector('.cv-tpl-grid') as HTMLElement;
    this.tplGridFull = this.root.querySelector('.cv-tpl-full') as HTMLElement;
    this.recentGrid = this.root.querySelector('.cv-recent-grid') as HTMLElement;
    this.recentGridFull = this.root.querySelector('.cv-recent-full') as HTMLElement;
    this.trashList = this.root.querySelector('.cv-trash-list') as HTMLElement;
    this.confirmBar = this.root.querySelector('.cv-home-confirm') as HTMLElement;
    this.back = this.root.querySelector('[data-a="back"]') as HTMLElement;
    this.menu = el('<div class="cv-menu cv-home-menu" role="menu" hidden></div>');
    document.getElementById('overlay-root')?.appendChild(this.menu);
    this.preview = new TemplateDialog(this.hooks);
    this.collect();
    this.greet();
    host.appendChild(this.root);
    this.wire();
    this.sidePanel();
    this.renderTemplates();
    onLanguage(() => this.relabel());
  }

  private collect(): void {
    const walker = document.createTreeWalker(this.root, NodeFilter.SHOW_TEXT);
    for (let n = walker.nextNode(); n; n = walker.nextNode()) {
      const en = spoken.get((n as Text).data);
      if (en !== undefined && !(n.parentElement?.closest('[translate="no"]'))) this.labels.push({ node: n as Text, attr: '', en });
    }
    for (const node of [this.root, ...Array.from(this.root.querySelectorAll('[title], [aria-label], [placeholder]'))]) {
      for (const attr of ATTRS) {
        const en = spoken.get(node.getAttribute(attr) ?? '');
        if (en !== undefined) this.labels.push({ node, attr, en });
      }
    }
  }

  private relabel(): void {
    for (const l of this.labels) {
      const shown = say(l.en);
      if (l.node instanceof Text) { if (l.node.data !== shown) l.node.data = shown; }
      else if (l.node.getAttribute(l.attr) !== shown) l.node.setAttribute(l.attr, shown);
    }
    this.greet();
    this.renderTemplates();
    this.renderGarments();
    this.renderTrash();
  }

  private greet(): void {
    const h = new Date().getHours();
    setText(this.root.querySelector('[data-page="start"] .cv-home-title'), t(h < 12 ? 'Good morning' : h < 18 ? 'Good afternoon' : 'Good evening'));
  }

  private sidePanel(): void {
    const root = this.root;
    const bar = root.querySelector<HTMLElement>('.cv-home-resizer');
    if (!bar) return;
    const save = (): void => {
      try {
        localStorage.setItem(NAV_KEY, root.style.getPropertyValue('--cv-home-nav-w'));
        localStorage.setItem(NAV_HIDDEN_KEY, root.classList.contains('is-nav-hidden') ? '1' : '0');
      } catch { return; }
    };
    const hide = (on: boolean): void => { setClass(root, 'is-nav-hidden', on); save(); };
    try {
      const w = (localStorage.getItem(NAV_KEY) ?? '').trim();
      if (/^\d{3}px$/.test(w)) root.style.setProperty('--cv-home-nav-w', `${Math.max(180, Math.min(420, parseInt(w, 10)))}px`);
      if (localStorage.getItem(NAV_HIDDEN_KEY) === '1') root.classList.add('is-nav-hidden');
    } catch { void 0; }
    root.querySelector('.cv-home-nav-hide')?.addEventListener('click', () => hide(true));
    root.querySelector('.cv-home-nav-show')?.addEventListener('click', () => hide(false));
    bar.addEventListener('pointerdown', (e) => {
      e.preventDefault();
      const x0 = e.clientX;
      const start = root.querySelector('.cv-home-nav')?.getBoundingClientRect().width || 232;
      const rtl = getComputedStyle(root).direction === 'rtl';
      bar.classList.add('is-dragging');
      const mv = (ev: PointerEvent): void => {
        const raw = start + (rtl ? x0 - ev.clientX : ev.clientX - x0);
        const collapse = raw < 120;
        if (root.classList.contains('is-nav-hidden') !== collapse) setClass(root, 'is-nav-hidden', collapse);
        if (!collapse) root.style.setProperty('--cv-home-nav-w', `${Math.round(Math.max(180, Math.min(420, raw)))}px`);
      };
      const up = (): void => {
        bar.classList.remove('is-dragging');
        window.removeEventListener('pointermove', mv);
        window.removeEventListener('pointerup', up);
        window.removeEventListener('pointercancel', up);
        save();
      };
      window.addEventListener('pointermove', mv);
      window.addEventListener('pointerup', up);
      window.addEventListener('pointercancel', up);
    });
  }

  private wire(): void {
    this.root.addEventListener('click', (e) => {
      const target = e.target as HTMLElement;
      this.closeMenu();
      const v = (target.closest('[data-v]') as HTMLElement | null)?.dataset.v as HomePage | undefined;
      if (v) { this.show(v); return; }
      if (target.closest('[data-open-garment]')) { this.leave(); return; }
      const a = (target.closest('[data-a]') as HTMLElement | null)?.dataset.a;
      const row = (target.closest('[data-entry]') as HTMLElement | null)?.dataset.entry;
      if (a === 'confirm-yes') { const go = this.onConfirm; this.askClose(); go?.(); return; }
      if (a === 'confirm-no') { this.askClose(); return; }
      if (a === 'empty-trash') { this.ask(t('Permanently delete everything in the Trash ({n})? This cannot be undone.').replace('{n}', String(this.trashed.length)), t('Empty Trash'), () => void this.purge(null)); return; }
      if (a === 'restore' && row) { void this.restore([row]); return; }
      if (a === 'purge' && row) { const x = this.trashed.find((d) => d.entry === row); this.ask(t('Permanently delete {name}? This cannot be undone.').replace('{name}', x?.title ?? ''), t('Delete permanently'), () => void this.purge(row)); return; }
      const more = target.closest('.cv-card-more') as HTMLElement | null;
      if (more) {
        e.stopPropagation();
        const id = (more.closest('.cv-card') as HTMLElement).dataset.garment;
        const r = more.getBoundingClientRect();
        if (id) this.openMenu(id, r.left, r.bottom + 4);
        return;
      }
      const hit = target.closest('.cv-card') as HTMLElement | null;
      if (hit?.dataset.garment && (e.ctrlKey || e.metaKey || e.shiftKey)) { this.pickCard(hit, e.shiftKey); return; }
      if (a === 'blank') this.hooks.blankGarment();
      else if (a === 'open') this.hooks.openFile();
      else if (a === 'choose') this.pick();
      else if (a === 'prefs') this.hooks.preferences();
      else if (a === 'agents') this.hooks.agents();
      else if (a === 'back') this.leave();
      else if (a === 'local') toast(t('Your garments, fabrics and bodies stay on this computer.'));
      if (hit?.dataset.demo) { this.hooks.demo(hit.dataset.demo); return; }
      if (hit?.dataset.garment) { const g = this.garments.find((x) => x.id === hit.dataset.garment); if (g) this.hooks.openGarment(g); }
      if (hit?.dataset.template) {
        const tpl = this.templates.find((x) => x.id === hit.dataset.template);
        if (tpl) this.preview.show(tpl);
      }
      const chip = target.closest('.cv-chip') as HTMLElement | null;
      if (chip?.dataset.cat) { this.category = chip.dataset.cat; this.renderTemplates(); }
    });
    for (const input of Array.from(this.root.querySelectorAll<HTMLInputElement>('.cv-tpl-search'))) {
      input.addEventListener('input', () => {
        this.query = input.value.trim().toLowerCase();
        for (const other of Array.from(this.root.querySelectorAll<HTMLInputElement>('.cv-tpl-search'))) if (other !== input && other.value !== input.value) other.value = input.value;
        this.renderTemplates();
      });
    }
    this.root.addEventListener('contextmenu', (e) => {
      const card = (e.target as HTMLElement).closest('.cv-card') as HTMLElement | null;
      if (!card?.dataset.garment) return;
      e.preventDefault();
      if (!this.selected.has(card.dataset.garment)) { this.clearSelection(); this.setSelected(card.dataset.garment, true); }
      this.openMenu(card.dataset.garment, e.clientX, e.clientY);
    });
    this.root.addEventListener('keydown', (e) => {
      if (e.key !== 'Delete') return;
      const card = (e.target as HTMLElement).closest('.cv-card') as HTMLElement | null;
      const id = card?.dataset.garment;
      if (!id) return;
      e.preventDefault();
      this.requestDelete(this.selected.has(id) ? [...this.selected] : [id]);
    });
    document.addEventListener('keydown', (e) => {
      if (e.key !== 'Escape' || !this.isOpen || e.defaultPrevented || anyDialogOpen()) return;
      if (!this.menu.hidden) { this.closeMenu(); return; }
      if (!this.confirmBar.hidden) { this.askClose(); return; }
      if (this.selected.size) { this.clearSelection(); return; }
      if (this.hooks.hasGarment()) this.leave();
    });
    this.menu.addEventListener('contextmenu', (e) => e.preventDefault());
    document.addEventListener('pointerdown', (e) => { if (!this.menu.hidden && !this.menu.contains(e.target as Node)) this.closeMenu(); }, true);
    const hero = this.root.querySelector('.cv-home-hero') as HTMLElement;
    const hasFiles = (e: DragEvent): boolean => Array.from(e.dataTransfer?.types ?? []).includes('Files');
    this.root.addEventListener('dragover', (e) => { if (hasFiles(e)) e.preventDefault(); });
    this.root.addEventListener('drop', (e) => {
      if (!hasFiles(e)) return;
      e.preventDefault();
      setClass(hero, 'is-drag-over', false);
      const files = Array.from(e.dataTransfer?.files ?? []);
      if (files.length && !('__TAURI_INTERNALS__' in window)) this.hooks.openFiles(files);
    });
    hero.addEventListener('dragover', (e) => { if (hasFiles(e)) setClass(hero, 'is-drag-over', true); });
    hero.addEventListener('dragleave', () => setClass(hero, 'is-drag-over', false));
    if ('__TAURI_INTERNALS__' in window) {
      void import('@tauri-apps/api/webview').then((m) => m.getCurrentWebview().onDragDropEvent((e) => {
        if (!this.isOpen) return;
        const p = e.payload;
        if (p.type === 'enter' || p.type === 'over') setClass(hero, 'is-drag-over', true);
        else setClass(hero, 'is-drag-over', false);
        if (p.type === 'drop' && p.paths.length && !anyDialogOpen()) this.hooks.openPaths(p.paths);
      })).catch(() => undefined);
    }
  }

  private pick(): void {
    if (this.hooks.chooseFile()) return;
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.ffbrc,.dxf';
    input.onchange = () => { const files = Array.from(input.files ?? []); if (files.length) this.hooks.openFiles(files); };
    input.click();
  }

  async load(): Promise<void> {
    const run = ++this.loads;
    const [tp, lib, tr] = await Promise.all([api.templates().catch(() => null), api.library().catch(() => null), api.trash().catch(() => null)]);
    if (run !== this.loads) return;
    if (tp) { this.templates = tp.templates; this.categories = [ALL, ...tp.categories]; }
    if (tr) this.trashed = tr.garments;
    if (lib) {
      this.garments = lib.garments;
      this.pinned = lib.pinned;
      this.hiddenRecent = lib.recentHidden;
      for (const id of [...this.selected]) if (!this.garments.some((g) => g.id === id)) this.selected.delete(id);
    }
    this.greet();
    this.tplReady = true;
    this.renderTemplates();
    if (lib) this.renderGarments();
    if (tr) this.renderTrash();
  }

  refresh(): void { if (this.isOpen) void this.load(); }

  private ask(text: string, action: string, go: () => void): void {
    this.onConfirm = go;
    const line = this.confirmBar.querySelector('.cv-home-confirm-text') as HTMLElement;
    setText(line, text);
    setTitle(line, text);
    setText(this.confirmBar.querySelector('[data-a="confirm-yes"]'), action);
    setHidden(this.confirmBar, false);
    (this.confirmBar.querySelector('[data-a="confirm-no"]') as HTMLElement).focus();
  }

  private askClose(): void {
    this.onConfirm = null;
    setHidden(this.confirmBar, true);
  }

  private closeMenu(): void { setHidden(this.menu, true); }

  private openMenu(id: string, x: number, y: number): void {
    const ids = this.selected.has(id) && this.selected.size > 1 ? [...this.selected] : [id];
    const many = ids.length > 1;
    const one = this.garments.find((g) => g.id === id);
    const inLibrary = ids.filter((x) => this.garments.find((g) => g.id === x)?.library);
    const rows: Array<[string, string, () => void]> = [];
    if (!many && one) {
      rows.push([t('Open'), 'open', () => this.hooks.openGarment(one)]);
      if (one.library) rows.push([t('Duplicate'), 'dup', () => { void api.duplicate(id).then(() => this.load()).then(() => toast(t('Duplicated.'))).catch(failed); }]);
    }
    const allPinned = ids.every((x) => this.pinned.includes(x));
    rows.push([allPinned ? t('Unpin from list') : t('Pin to list'), 'pin', () => this.setPinned(ids, !allPinned)]);
    if (!many && one && !one.library) rows.push([t('Remove from recent'), 'forget', () => void this.forget([id])]);
    else rows.push([t('Remove from list'), 'hide', () => this.setHiddenRecent(ids, true)]);
    if (many && inLibrary.length) rows.push([t('Delete selected ({n})').replace('{n}', String(inLibrary.length)), 'del', () => this.requestDelete(inLibrary)]);
    else if (!many && one?.library) rows.push([t('Delete'), 'del', () => this.requestDelete(ids)]);
    this.menu.textContent = '';
    for (const [label, key, run] of rows) {
      const b = el<HTMLButtonElement>(`<button class="cv-menu-item" role="menuitem" type="button" data-k="${key}"><span class="cv-menu-label"></span></button>`);
      setText(b.querySelector('.cv-menu-label'), label);
      b.addEventListener('click', () => { this.closeMenu(); run(); });
      this.menu.appendChild(b);
    }
    this.menu.style.visibility = 'hidden';
    setHidden(this.menu, false);
    const w = this.menu.offsetWidth, h = this.menu.offsetHeight;
    this.menu.style.left = `${Math.round(Math.max(4, Math.min(x, window.innerWidth - w - 4)))}px`;
    this.menu.style.top = `${Math.round(Math.max(4, Math.min(y, window.innerHeight - h - 4)))}px`;
    this.menu.style.visibility = '';
    (this.menu.firstElementChild as HTMLElement | null)?.focus();
  }

  private saveRecent(): void {
    this.renderGarments();
    void api.setSettings({ pinned: this.pinned, recentHidden: this.hiddenRecent }).catch(failed);
  }

  private setPinned(ids: string[], on: boolean): void {
    this.pinned = on ? [...this.pinned.filter((x) => !ids.includes(x)), ...ids] : this.pinned.filter((x) => !ids.includes(x));
    if (on) this.hiddenRecent = this.hiddenRecent.filter((x) => !ids.includes(x));
    this.saveRecent();
  }

  private setHiddenRecent(ids: string[], on: boolean): void {
    this.hiddenRecent = on ? [...this.hiddenRecent.filter((x) => !ids.includes(x)), ...ids] : this.hiddenRecent.filter((x) => !ids.includes(x));
    if (on) this.pinned = this.pinned.filter((x) => !ids.includes(x));
    this.saveRecent();
  }

  private setSelected(id: string, on: boolean): void {
    if (on) this.selected.add(id); else this.selected.delete(id);
    for (const grid of [this.recentGrid, this.recentGridFull]) {
      const n = grid.querySelector<HTMLElement>(`.cv-card[data-garment="${id}"]`);
      if (n) setClass(n, 'is-selected', on);
    }
  }

  private clearSelection(): void {
    for (const id of [...this.selected]) this.setSelected(id, false);
    this.anchor = '';
  }

  private pickCard(card: HTMLElement, range: boolean): void {
    const id = card.dataset.garment as string;
    if (range && this.anchor && this.anchor !== id) {
      const order = Array.from((card.parentElement as HTMLElement).querySelectorAll<HTMLElement>('.cv-card[data-garment]')).map((n) => n.dataset.garment as string);
      const a = order.indexOf(this.anchor), b = order.indexOf(id);
      if (a >= 0 && b >= 0) for (let i = Math.min(a, b); i <= Math.max(a, b); i++) this.setSelected(order[i], true);
      return;
    }
    this.setSelected(id, !this.selected.has(id));
    this.anchor = id;
  }

  private requestDelete(ids: string[]): void {
    const inLibrary = ids.filter((x) => this.garments.find((g) => g.id === x)?.library);
    const outside = ids.filter((x) => this.garments.some((g) => g.id === x && !g.library));
    if (outside.length) void this.forget(outside);
    if (!inLibrary.length) return;
    if (inLibrary.length === 1) { void this.trashGarments(inLibrary); return; }
    this.ask(t('Move the selected garments to the Trash ({n})?').replace('{n}', String(inLibrary.length)), t('Move to Trash'), () => void this.trashGarments(inLibrary));
  }

  private async forget(ids: string[]): Promise<void> {
    const done: string[] = [];
    for (const id of ids) {
      try { await api.forget(id); done.push(id); } catch (e) { failed(e); }
    }
    if (!done.length) return;
    for (const id of done) this.selected.delete(id);
    this.garments = this.garments.filter((g) => !done.includes(g.id));
    this.pinned = this.pinned.filter((x) => !done.includes(x));
    this.hiddenRecent = this.hiddenRecent.filter((x) => !done.includes(x));
    this.renderGarments();
    this.renderTrash();
    this.hooks.recentChanged();
  }

  private async trashGarments(ids: string[]): Promise<void> {
    const done: GarmentSummary[] = [];
    const entries: string[] = [];
    for (const id of ids) {
      const g = this.garments.find((x) => x.id === id);
      if (!g) continue;
      try { entries.push((await api.remove(id)).entry); done.push(g); } catch (e) { failed(e); }
    }
    if (!done.length) return;
    for (const g of done) { this.selected.delete(g.id); this.hooks.closeGarment(g.path); }
    this.garments = this.garments.filter((g) => !done.includes(g));
    this.pinned = this.pinned.filter((x) => !done.some((g) => g.id === x));
    this.hiddenRecent = this.hiddenRecent.filter((x) => !done.some((g) => g.id === x));
    this.renderGarments();
    const tr = await api.trash().catch(() => null);
    if (tr) this.trashed = tr.garments;
    this.renderTrash();
    this.hooks.recentChanged();
    const text = done.length === 1 ? t('Moved {name} to the Trash.').replace('{name}', done[0].title) : t('Moved to the Trash: {n}').replace('{n}', String(done.length));
    toast(text, 'info', { actions: [{ label: t('Undo'), run: () => void this.restore(entries) }], ms: 8000 });
  }

  private async restore(entries: string[]): Promise<void> {
    let n = 0;
    for (const entry of entries) {
      try { await api.restoreTrashed(entry); n++; } catch (e) { failed(e); }
    }
    if (!n) return;
    const [lib, tr] = await Promise.all([api.library().catch(() => null), api.trash().catch(() => null)]);
    if (lib) { this.garments = lib.garments; this.pinned = lib.pinned; this.hiddenRecent = lib.recentHidden; }
    if (tr) this.trashed = tr.garments;
    this.renderGarments();
    this.renderTrash();
    toast(n === 1 ? t('Garment restored.') : t('Restored: {n}').replace('{n}', String(n)));
  }

  private async purge(entry: string | null): Promise<void> {
    try {
      if (entry) await api.purgeTrashed(entry); else await api.emptyTrash();
    } catch (e) { failed(e); }
    const tr = await api.trash().catch(() => null);
    this.trashed = tr ? tr.garments : entry ? this.trashed.filter((d) => d.entry !== entry) : [];
    this.renderTrash();
  }

  private renderTrash(): void {
    setText(this.root.querySelector('.cv-trash-count'), String(this.trashed.length));
    setText(this.root.querySelector('.cv-garment-count'), String(this.garments.length));
    const empty = this.root.querySelector<HTMLButtonElement>('.cv-btn-empty-trash');
    if (empty && empty.disabled !== !this.trashed.length) empty.disabled = !this.trashed.length;
    const none = this.root.querySelector<HTMLElement>('.cv-trash-none');
    if (none) setHidden(none, this.trashed.length > 0);
    const update = (n: HTMLElement, d: TrashedGarment): void => {
      setText(n.querySelector('.cv-card-name'), d.title);
      setText(n.querySelector('.cv-card-meta'), `${d.panels === null ? '' : `${t('Panels')}: ${d.panels} · `}${t('Deleted {when}').replace('{when}', ago(new Date(d.deletedAt * 1000).toISOString()))}`);
      setText(n.querySelector('[data-a="restore"] .cv-btn-text'), t('Restore'));
      setText(n.querySelector('[data-a="purge"] .cv-btn-text'), t('Delete permanently'));
    };
    const create = (d: TrashedGarment): HTMLElement => el(`<div class="cv-trash-row" data-entry="${escapeHtml(d.entry)}"><span class="cv-trash-icon">${icon('panel', 18)}</span><span class="cv-trash-text"><span class="cv-card-name"></span><span class="cv-card-meta"></span></span><button class="cv-btn cv-btn-sm" type="button" data-a="restore">${icon('undo', 14)}<span class="cv-btn-text"></span></button><button class="cv-btn cv-btn-sm cv-btn-danger" type="button" data-a="purge">${icon('delete', 14)}<span class="cv-btn-text"></span></button></div>`);
    patchList(this.trashList, this.trashed, { key: (d) => d.entry, create, update });
  }

  show(v: HomePage = this.view): void {
    this.view = v;
    for (const page of Array.from(this.root.querySelectorAll<HTMLElement>('[data-page]'))) setHidden(page, page.dataset.page !== v);
    for (const b of Array.from(this.root.querySelectorAll<HTMLElement>('.cv-home-nav-item[data-v]'))) setClass(b, 'is-active', b.dataset.v === v);
  }

  open(): void {
    const was = this.isOpen;
    setHidden(this.root, false);
    setClass(document.body, 'nm-home', true);
    if (was) return;
    void this.load();
    this.root.focus({ preventScroll: true });
    this.hooks.onVisible?.(true);
    document.dispatchEvent(new CustomEvent('ff:home', { detail: { open: true } }));
  }

  close(): void {
    const was = this.isOpen;
    setHidden(this.root, true);
    this.closeMenu();
    this.askClose();
    this.clearSelection();
    setClass(document.body, 'nm-home', false);
    if (!was) return;
    if (this.root.contains(document.activeElement)) (document.activeElement as HTMLElement).blur();
    this.hooks.onVisible?.(false);
    document.dispatchEvent(new CustomEvent('ff:home', { detail: { open: false } }));
  }

  get isOpen(): boolean { return !this.root.hidden; }

  force(page?: HomePage): void {
    if (page) this.show(page);
    this.forced = true;
    this.sync();
  }

  leave(): void {
    this.forced = false;
    this.sync();
  }

  sync(): void {
    const has = this.hooks.hasGarment();
    if (has && !this.lastHas) this.forced = false;
    this.lastHas = has;
    if (has) this.hadGarment = true;
    setHidden(this.back, !has);
    if (this.forced || (!has && (this.hadGarment ? this.hooks.afterClose() : this.hooks.autoShow()))) this.open();
    else this.close();
  }

  setOpenGarments(list: Array<{ id: string; title: string }>): void {
    const group = this.root.querySelector('.cv-home-open') as HTMLElement;
    setHidden(group, !list.length);
    patchList(group.querySelector('.cv-home-open-list') as HTMLElement, list, {
      key: (d) => d.id,
      create: (d) => {
        const node = el(`<button class="cv-home-nav-item" type="button">${icon('panel', 16)}<span class="cv-home-open-name" translate="no"></span></button>`);
        node.dataset.openGarment = d.id;
        return node;
      },
      update: (node, d) => setText(node.querySelector('.cv-home-open-name'), d.title.trim() || t('Untitled')),
    });
  }

  private fill(prev: HTMLElement, key: string, blank: string): void {
    if (prev.dataset.pic === key) return;
    prev.dataset.pic = key;
    const toBlank = (): void => {
      if (prev.dataset.pic !== key || prev.classList.contains('is-blank')) return;
      prev.classList.add('is-blank');
      prev.innerHTML = icon(blank, 28);
    };
    if (!key) { toBlank(); return; }
    void picture(key).then((url) => {
      const probe = new Image();
      probe.alt = '';
      probe.draggable = false;
      probe.decoding = 'async';
      probe.src = url;
      return probe.decode().catch(() => undefined).then(() => {
        if (prev.dataset.pic !== key || !probe.naturalWidth) { if (!probe.naturalWidth) toBlank(); return; }
        setClass(prev, 'is-blank', false);
        prev.replaceChildren(probe);
      });
    }).catch(() => { if (!prev.querySelector('img')) toBlank(); });
  }

  private renderTemplates(): void {
    const q = this.query;
    const list = this.templates.filter((x) => (this.category === ALL || x.category === this.category) && (!q || `${x.name} ${say(x.name)} ${x.category} ${say(x.category)} ${x.tags.join(' ')} ${x.description}`.toLowerCase().includes(q)));
    setText(this.root.querySelector('.cv-tpl-count'), String(this.templates.length));
    for (const host of Array.from(this.root.querySelectorAll<HTMLElement>('.cv-cats'))) {
      const chip = (n: HTMLElement, c: string): void => {
        setClass(n, 'is-active', c === this.category);
        setText(n.querySelector('.cv-chip-name'), c === ALL ? t('All') : say(c));
        setText(n.querySelector('.cv-chip-count'), !this.templates.length ? '' : c === ALL ? String(this.templates.length) : String(this.templates.filter((x) => x.category === c).length));
      };
      patchList(host, this.categories, {
        key: (c) => c,
        create: (c) => el(`<button class="cv-chip" type="button" data-cat="${escapeHtml(c)}"><span class="cv-chip-name"></span><span class="cv-chip-count"></span></button>`),
        update: chip,
      });
    }
    const create = (x: TemplateSummary): HTMLElement => el(`<button class="cv-card" type="button" data-template="${escapeHtml(x.id)}"><span class="cv-card-preview" aria-hidden="true"></span><span class="cv-card-body"><span class="cv-card-name"></span><span class="cv-card-meta"></span></span></button>`);
    const update = (n: HTMLElement, x: TemplateSummary): void => {
      setText(n.querySelector('.cv-card-name'), say(x.name));
      setText(n.querySelector('.cv-card-meta'), `${say(x.category)}${x.panels === null ? '' : ` · ${t('Panels')}: ${x.panels}`}`);
      this.fill(n.querySelector('.cv-card-preview') as HTMLElement, x.thumb ? templateThumb(x) : '', 'template');
    };
    if (!this.tplReady) return;
    patchList(this.tplGrid, list.slice(0, 15), { key: (x) => x.id, create, update });
    patchList(this.tplGridFull, list, { key: (x) => x.id, create, update });
  }

  private renderGarments(): void {
    setText(this.root.querySelector('.cv-garment-count'), String(this.garments.length));
    const create = (g: GarmentSummary): HTMLElement => el(`<button class="cv-card" type="button" data-garment="${escapeHtml(g.id)}"><span class="cv-card-preview" aria-hidden="true"></span><span class="cv-card-body"><span class="cv-card-name" translate="no"></span><span class="cv-card-meta"></span></span><span class="cv-card-more" role="button" tabindex="-1">${icon('more3', 16)}</span></button>`);
    const update = (n: HTMLElement, g: GarmentSummary): void => {
      const pinned = this.pinned.includes(g.id);
      setText(n.querySelector('.cv-card-name'), g.title);
      setText(n.querySelector('.cv-card-meta'), `${pinned ? `${t('Pinned')} · ` : ''}${g.panels === null ? '' : `${t('Panels')}: ${g.panels} · `}${ago(g.modified)}${g.library ? '' : ` · ${folderOf(g.path)}`}`);
      setTitle(n, g.path);
      const more = n.querySelector('.cv-card-more') as HTMLElement;
      setTitle(more, t('More actions'));
      if (more.getAttribute('aria-label') !== more.title) more.setAttribute('aria-label', more.title);
      setClass(n, 'is-pinned', pinned);
      setClass(n, 'is-outside', !g.library);
      setClass(n, 'is-selected', this.selected.has(g.id));
      this.fill(n.querySelector('.cv-card-preview') as HTMLElement, g.thumb ? garmentThumb(g) : '', 'panel');
    };
    const pinnedFirst = (list: GarmentSummary[]): GarmentSummary[] => [...list.filter((g) => this.pinned.includes(g.id)), ...list.filter((g) => !this.pinned.includes(g.id))];
    const recent = pinnedFirst(this.garments.filter((g) => !this.hiddenRecent.includes(g.id)));
    patchList(this.recentGrid, recent.slice(0, 5), { key: (g) => g.id, create, update });
    patchList(this.recentGridFull, pinnedFirst(this.garments), { key: (g) => g.id, create, update });
    const none = this.recentGrid.querySelector<HTMLElement>('.cv-card.is-empty');
    if (none) setHidden(none, recent.length > 0);
    const noneFull = this.root.querySelector<HTMLElement>('.cv-recent-none');
    if (noneFull) setHidden(noneFull, this.garments.length > 0);
  }
}

let home: Home | null = null;

export function mountHome(host: HTMLElement, hooks: HomeHooks): Home {
  if (!home) watchSecondService();
  home ??= new Home(host, hooks);
  return home;
}

export function showHome(page?: HomePage): void { home?.force(page); }

export function hideHome(): void { home?.leave(); }

export function isHomeOpen(): boolean { return home?.isOpen ?? false; }
