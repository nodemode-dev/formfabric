import { el, patchList, q, setClass, setHidden, setText } from '../ui/dom';
import { icon } from '../ui/icons';
import { brandMark } from '../ui/brand';
import { t } from '../ui/i18n';
import { SAMPLES } from './window-commands';

export interface HomeDeps {
  autoShow: () => boolean;
  afterClose: () => boolean;
  recent: () => string[];
  openPath: (path: string) => void;
  hasProject: () => boolean;
  can: (command: string) => boolean;
  run: (command: string) => void;
}

type Page = 'start' | 'recent';


const baseName = (p: string): string => p.split(/[\\/]/).pop() ?? p;

export class HomeScreen {
  readonly root = el('<section class="cv-home" aria-label="Home" hidden></section>');
  private page: Page = 'start';
  private forced = false;
  private hadProject = false;
  private lastHas = false;
  private readonly back: HTMLElement;

  constructor(host: HTMLElement, private readonly d: HomeDeps) {
    const mod = /Mac|iPhone|iPad/.test(navigator.platform) ? '⌘' : 'Ctrl';
    const navItem = (v: Page, ic: string, label: string): string => `<button class="cv-home-nav-item" type="button" data-v="${v}">${icon(ic, 16)}<span>${label}</span></button>`;
    const cmdItem = (cmd: string, ic: string, label: string): string => `<button class="cv-home-nav-item" type="button" data-cmd="${cmd}">${icon(ic, 16)}<span>${label}</span></button>`;
    const key = (k: string): string => `<kbd class="cv-key">${mod}</kbd><kbd class="cv-key">${k}</kbd>`;
    const tiles = SAMPLES.map(([n, label]) => `<button class="cv-home-size" type="button" data-cmd="app.sample.${n}.current" disabled><span class="cv-home-size-box">${icon('panel', 28)}</span><span class="cv-home-size-name">${t(label)}</span><span class="cv-home-size-meta">${t('Sample garment')}</span></button>`).join('');
    this.root.innerHTML = `<nav class="cv-home-nav" aria-label="Home">
      <div class="cv-home-brand">${brandMark(18)}FormFabric <span>by NodeMode</span></div>
      ${navItem('start', 'panel', 'Start')}${navItem('recent', 'history', 'Recent')}
      <div class="cv-home-nav-sep"></div>
      <div class="cv-home-nav-title">Create</div>
      ${cmdItem('file.new', 'newFile', 'New garment')}${cmdItem('file.open', 'folderOpen', 'Open project')}
      <div class="cv-home-nav-foot"><button class="cv-home-nav-item" type="button" data-back hidden>${icon('chevronLeft', 16)}<span>Back to Workspace</span></button>${cmdItem('app.settings', 'settings', 'Preferences')}${cmdItem('app.about', 'info', 'About')}</div>
    </nav>
    <div class="cv-home-main">
      <div class="cv-home-page" data-page="start">
        <div class="cv-home-head"><div><h1 class="cv-home-title"></h1><p class="cv-home-sub">Start a new garment or open a project.</p></div></div>
        <div class="cv-home-hero">
          <span class="cv-home-hero-icon">${icon('panel', 26)}</span>
          <div class="cv-home-hero-text"><div class="cv-home-hero-title">Draw your pattern</div>
            <div class="cv-home-hero-sub">Draw panels and inner shapes in millimetres, then dress an avatar. Projects keep every panel and every step of history.</div>
            <div class="cv-home-formats"><span class="cv-home-format">FFBRC</span></div></div>
          <div class="cv-home-hero-actions">
            <button class="cv-home-action is-primary" type="button" data-cmd="file.new">${icon('newFile', 16)}<span class="cv-home-label">New garment</span>${key('N')}</button>
            <button class="cv-home-action" type="button" data-cmd="file.open">${icon('folderOpen', 16)}<span class="cv-home-label">Open project...</span>${key('O')}</button>
          </div>
        </div>
        <section class="cv-home-section"><div class="cv-home-section-head"><h2 class="cv-home-section-title">Recent</h2><button class="cv-btn cv-btn-ghost" type="button" data-v="recent">See all</button></div><div class="cv-home-start-recent" data-slot="start"></div></section>
        <section class="cv-home-section"><div class="cv-home-section-head"><h2 class="cv-home-section-title">Templates</h2></div><div class="cv-home-sizes">${tiles}</div></section>
      </div>
      <div class="cv-home-page" data-page="recent" hidden><div class="cv-home-head"><div><h1 class="cv-home-title">Recent</h1><p class="cv-home-sub">Projects you opened or saved, newest first.</p></div></div><div data-slot="recent"></div></div>
    </div>`;
    this.back = q(this.root, '[data-back]');
    this.root.addEventListener('click', (e) => {
      const target = e.target as HTMLElement;
      if (target.closest('[data-back]')) { this.forced = false; this.sync(); return; }
      const v = target.closest<HTMLElement>('[data-v]')?.dataset.v as Page | undefined;
      if (v) { this.show(v); return; }
      const cmd = target.closest<HTMLElement>('[data-cmd]')?.dataset.cmd;
      if (cmd) { this.d.run(cmd); return; }
      const file = target.closest<HTMLElement>('[data-path]')?.dataset.path;
      if (file) this.d.openPath(file);
    });
    host.appendChild(this.root);
    document.addEventListener('keydown', (e) => { if (e.key === 'Escape' && this.forced && this.d.hasProject()) { this.forced = false; this.sync(); } });
    this.paint();
    this.sync();
  }

  private paint(): void {
    for (const n of Array.from(this.root.querySelectorAll<HTMLElement>('.cv-home-page'))) setHidden(n, n.dataset.page !== this.page);
    for (const n of Array.from(this.root.querySelectorAll<HTMLElement>('.cv-home-nav-item[data-v]'))) setClass(n, 'is-active', n.dataset.v === this.page);
    const h = new Date().getHours();
    setText(this.root.querySelector('[data-page="start"] .cv-home-title'), t(h < 12 ? 'Good morning' : h < 18 ? 'Good afternoon' : 'Good evening'));
  }

  refresh(): void {
    const files = this.d.recent();
    const fill = (slot: string, list: string[]): void => {
      const box = q(this.root, `[data-slot="${slot}"]`);
      if (!list.length) {
        if (!box.querySelector('.cv-home-none')) box.replaceChildren(el(`<p class="cv-home-none">${t('No recent projects yet.')}</p>`));
        return;
      }
      box.querySelector('.cv-home-none')?.remove();
      let grid = box.querySelector<HTMLElement>('.cv-home-recent');
      if (!grid) { grid = el('<div class="cv-home-recent"></div>'); box.appendChild(grid); }
      patchList(grid, list, {
        key: (p) => p,
        create: (p) => { const n = el<HTMLElement>(`<button class="cv-home-file" type="button"><span class="cv-home-file-icon">${icon('panel', 18)}</span><span class="cv-home-file-text"><span class="cv-home-file-name"></span><span class="cv-home-file-path"></span></span></button>`); this.fillFile(n, p); return n; },
        update: (n, p) => this.fillFile(n, p),
      });
    };
    fill('start', files.slice(0, 8));
    fill('recent', files);
    for (const b of Array.from(this.root.querySelectorAll<HTMLButtonElement>('[data-cmd]'))) {
      const ok = this.d.can(b.dataset.cmd ?? '');
      if (b.disabled === ok) b.disabled = !ok;
    }
  }

  private fillFile(n: HTMLElement, p: string): void {
    n.dataset.path = p;
    setText(q(n, '.cv-home-file-name'), baseName(p));
    setText(q(n, '.cv-home-file-path'), p);
  }

  get isShown(): boolean { return !this.root.hidden; }

  show(page?: Page): void {
    if (page && page !== this.page) { this.page = page; this.paint(); }
    this.forced = true;
    this.sync();
  }

  sync(): void {
    const has = this.d.hasProject();
    if (has && !this.lastHas) this.forced = false;
    this.lastHas = has;
    if (has) this.hadProject = true;
    const on = this.forced || (!has && (this.hadProject ? this.d.afterClose() : this.d.autoShow()));
    if (on && this.root.hidden) { this.paint(); this.refresh(); }
    setHidden(this.root, !on);
    setHidden(this.back, !has);
  }
}
