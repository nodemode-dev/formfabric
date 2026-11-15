import { el, setClass } from '../ui/dom';
import { icon } from '../ui/icons';
import { brandMark } from '../ui/brand';
import { t } from '../ui/i18n';
import { readChoice, setTheme } from '../ui/themes';
import type { Commands } from '../ui/commands';

export const WORKSPACES = [
  { id: 'pattern', label: 'Pattern', ready: true },
  { id: 'sew', label: 'Sew', ready: true },
  { id: 'drape', label: 'Drape', ready: true },
  { id: 'animate', label: 'Animate', ready: true },
  { id: 'render', label: 'Render', ready: true },
] as const;

const WS_KEY = 'formfabric.workspace';

export class TitleBar {
  private readonly tabs = new Map<string, HTMLButtonElement>();
  private current: string = 'pattern';
  private readonly dockBtns = new Map<string, HTMLButtonElement>();

  constructor(readonly header: HTMLElement, private readonly commands: Commands, private readonly onWorkspace: (id: string) => void) {
    header.insertAdjacentHTML('beforeend',
      `<div class="cv-brand" role="button" tabindex="0" data-cmd="app.home" title="Home" aria-label="Home">${brandMark(18)}</div>` +
      '<nav class="cv-menubar" aria-label="Main menu"></nav>' +
      '<div class="cv-tb-doc"></div>' +
      '<div class="cv-tb-app">FormFabric</div>' +
      '<div class="cv-ws" role="tablist" aria-label="Workspace"></div>' +
      `<div class="cv-zoom"><button class="cv-tt-btn" type="button" data-cmd="app.settings" title="Preferences" aria-label="Preferences">${icon('settings', 16)}</button></div>` +
      '<div class="cv-title-tools"></div>');
    const ws = header.querySelector('.cv-ws') as HTMLElement;
    let saved = 'pattern';
    try { saved = localStorage.getItem(WS_KEY) ?? 'pattern'; } catch { /* default */ }
    this.current = WORKSPACES.some((w) => w.id === saved && w.ready) ? saved : 'pattern';
    for (const w of WORKSPACES) {
      const b = el<HTMLButtonElement>('<button class="cv-ws-tab" type="button" role="tab"></button>');
      b.textContent = t(w.label);
      b.dataset.ws = w.id;
      b.disabled = !w.ready;
      if (!w.ready) b.title = t('Not available yet');
      b.addEventListener('click', () => this.choose(w.id));
      ws.appendChild(b);
      this.tabs.set(w.id, b);
    }
    this.paint();
    const tools = header.querySelector('.cv-title-tools') as HTMLElement;
    const theme = el<HTMLButtonElement>(`<button class="cv-tt-btn" type="button" title="Light or dark" aria-label="Light or dark"><span class="cv-tt-sun">${icon('sun', 16)}</span><span class="cv-tt-moon">${icon('moon', 16)}</span></button>`);
    const sync = (): void => {
      const light = readChoice().theme === 'light';
      (theme.children[0] as HTMLElement).hidden = light;
      (theme.children[1] as HTMLElement).hidden = !light;
      theme.setAttribute('aria-pressed', String(light));
    };
    theme.addEventListener('click', () => { setTheme(readChoice().theme === 'light' ? 'studio' : 'light'); sync(); });
    sync();
    document.addEventListener('nm:ui-settings-changed', sync);
    const mk = (cmd: string, ic: string, label: string): HTMLButtonElement => {
      const b = el<HTMLButtonElement>(`<button class="cv-tt-btn" type="button" data-cmd="${cmd}" title="${t(label)}" aria-label="${t(label)}" aria-pressed="false">${icon(ic, 16)}</button>`);
      this.dockBtns.set(cmd, b);
      return b;
    };
    const search = el<HTMLButtonElement>(`<button class="cv-tt-btn" type="button" data-cmd="app.palette" title="${t('Search commands')}" aria-label="${t('Search commands')}">${icon('search', 16)}</button>`);
    tools.append(theme, search, mk('app.claude', 'agent', 'Claude Code'), mk('app.terminal', 'terminal', 'Show Terminal'));
    header.addEventListener('click', (e) => {
      const b = e.target instanceof Element ? e.target.closest<HTMLElement>('[data-cmd]') : null;
      if (b?.dataset.cmd) this.commands.run(b.dataset.cmd);
    });
    header.querySelector('.cv-brand')?.addEventListener('keydown', (e) => {
      const key = (e as KeyboardEvent).key;
      if (key !== 'Enter' && key !== ' ') return;
      e.preventDefault();
      this.commands.run('app.home');
    });
  }

  choose(id: string): void {
    const w = WORKSPACES.find((x) => x.id === id);
    if (!w?.ready || id === this.current) return;
    this.current = id;
    try { localStorage.setItem(WS_KEY, id); } catch { /* this session only */ }
    this.paint();
    this.onWorkspace(id);
  }

  private paint(): void {
    for (const [id, b] of this.tabs) {
      setClass(b, 'is-active', id === this.current);
      b.setAttribute('aria-selected', String(id === this.current));
    }
  }

  syncDock(open: boolean, kind: string | null): void {
    for (const [cmd, b] of this.dockBtns) {
      const on = open && ((cmd === 'app.claude' && kind === 'claude') || (cmd === 'app.terminal' && kind === 'shell'));
      setClass(b, 'is-on', on);
      if (b.getAttribute('aria-pressed') !== String(on)) b.setAttribute('aria-pressed', String(on));
    }
  }
}
