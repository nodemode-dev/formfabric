import { Dialog, confirmAction } from './dialogs';
import { el, q, setClass, setHidden, setText, setValue } from './dom';
import { icon } from './icons';
import { language, languages, onLanguage, setLanguage, t, type Lang } from './i18n';
import { DEFAULT_PREFS, FONT_SIZES, LAYOUTS, UNITS, loadPrefs, resetPrefs, updatePrefs, type Prefs } from './prefs';
import { THEMES, readChoice, setAccent, setIntensity, setTheme } from './themes';
import { checkOf, numberRow, selectRow, sliderRow, toggleRow, type Row } from './controls';

export const PREF_SECTIONS = ['General', 'Interface', 'Workspace', 'Performance', 'History & Cache', 'Pattern', 'Drape', 'Viewport', 'Plugins'] as const;
export type PrefSection = (typeof PREF_SECTIONS)[number];

const RAIL_ICONS: Record<PrefSection, string> = {
  General: 'pfGeneral', Interface: 'pfInterface', Workspace: 'pfWorkspace', Performance: 'pfPerformance', 'History & Cache': 'pfHistory',
  Pattern: 'pen', Drape: 'panel', Viewport: 'layout3d', Plugins: 'pfPlugins',
};

const RAIL_KEY = 'formfabric.prefsRailCollapsed';
const TITLE_BAR_KEY = 'formfabric.customTitleBar';
const MENUS_KEY = 'formfabric.menusInWindow';
const isMac = /Mac/.test(navigator.platform);

function stored(key: string): boolean {
  try { return localStorage.getItem(key) === '1'; } catch { return false; }
}

function store(key: string, on: boolean): void {
  try { localStorage.setItem(key, on ? '1' : '0'); } catch { return; }
}

export class SettingsDialog extends Dialog {
  private readonly nav = el('<div class="cv-prefs-nav" role="tablist"></div>');
  private readonly grid = el('<div class="nm-theme-grid" role="group" aria-label="Theme"></div>');
  private readonly accent = el<HTMLInputElement>('<input class="cv-input" type="color" aria-label="Accent color">');
  private readonly sections = new Map<string, HTMLElement>();
  private pagesEl!: HTMLElement;
  private shell!: { frame: HTMLElement; rail: HTMLElement; port: HTMLElement; shade: SVGPathElement; line: SVGPathElement };
  private current = '';
  private wireRaf = 0;
  private spyRaf = 0;
  private spyMuted = 0;
  private wireD = '';
  private portY = Number.NaN;
  private prefs: Prefs = loadPrefs();
  private readonly rows = new Map<keyof Prefs, Row<never>>();

  constructor(private readonly onPrefs: (p: Prefs, keys: string[]) => void) {
    super('Preferences', 620);
    const pages = el('<div class="cv-prefs-pages"></div>');
    const grp = (label: string, ...content: HTMLElement[]): HTMLElement => {
      const g = el('<section class="cv-prefs-group"><h3 class="cv-prefs-label"></h3></section>');
      const h = q<HTMLElement>(g, '.cv-prefs-label');
      setText(h, t(label));
      onLanguage(() => setText(h, t(label)));
      g.append(...content);
      return g;
    };
    const note = (text: string): HTMLElement => { const n = el('<p class="cv-note"></p>'); setText(n, t(text)); onLanguage(() => setText(n, t(text))); return n; };
    const cols = (...content: HTMLElement[]): HTMLElement => { const c = el('<div class="cv-prefs-cols"></div>'); c.append(...content); return c; };
    const add = (name: PrefSection, ...content: HTMLElement[]): void => {
      const tab = el<HTMLButtonElement>(`<button class="cv-prefs-railtab" type="button" role="tab"><span class="cv-prefs-tabicon">${icon(RAIL_ICONS[name], 18)}</span><span class="cv-prefs-tablabel"></span></button>`);
      tab.dataset.section = name;
      const tabLabel = q<HTMLElement>(tab, '.cv-prefs-tablabel');
      const paint = (): void => { setText(tabLabel, t(name)); tab.title = t(name); };
      paint();
      onLanguage(paint);
      tab.addEventListener('click', () => this.showSection(name, 'smooth'));
      this.nav.appendChild(tab);
      const page = el(`<section class="cv-prefs-page cv-prefs-sec" role="tabpanel"><button class="cv-prefs-sechead" type="button" aria-expanded="true"><span class="cv-prefs-secicon">${icon(RAIL_ICONS[name], 16)}</span><span class="cv-prefs-seclabel"></span><span class="cv-prefs-chev">${icon('chevronDown', 16)}</span></button><div class="cv-prefs-secbody"></div></section>`);
      page.dataset.section = name;
      const headLabel = q<HTMLElement>(page, '.cv-prefs-seclabel');
      const paintHead = (): void => setText(headLabel, t(name));
      paintHead();
      onLanguage(paintHead);
      q(page, '.cv-prefs-sechead').addEventListener('click', () => this.foldSection(name));
      q<HTMLElement>(page, '.cv-prefs-secbody').append(...content);
      pages.appendChild(page);
      this.sections.set(name, page);
    };

    const toggle = (key: keyof Prefs, label: string): Row<boolean> => this.keep(key, toggleRow(label, (v) => this.change({ [key]: v } as Partial<Prefs>)));
    const slider = (key: keyof Prefs, label: string, min: number, max: number, step: number): Row<number> => this.keep(key, sliderRow(label, min, max, step, (v) => this.change({ [key]: v } as Partial<Prefs>), DEFAULT_PREFS[key] as number));
    const num = (key: keyof Prefs, label: string, min: number, max: number, step: number): Row<number> => this.keep(key, numberRow(label, min, max, step, (v) => this.change({ [key]: v } as Partial<Prefs>)));
    const pickRow = (key: keyof Prefs, label: string, options: ReadonlyArray<readonly [string, string]>, map: (v: string) => unknown = (v) => v): Row<string> => this.keep(key, selectRow(label, options, (v) => this.change({ [key]: map(v) } as Partial<Prefs>)));
    const later = <T>(row: Row<T>): HTMLElement => { row.disable(true); return row.el; };
    const soon = (): HTMLElement => note('Not available yet');

    const reset = el<HTMLButtonElement>('<button class="cv-btn cv-btn-danger" type="button">Reset All Preferences...</button>');
    reset.addEventListener('click', () => {
      void confirmAction('Reset all preferences?', 'Every preference and the theme go back to how FormFabric comes; your projects are kept. FormFabric reloads.', 'Reset', true).then((ok) => {
        if (!ok) return;
        resetPrefs();
        location.reload();
      });
    });

    const titleBar = checkOf('Use a custom title bar', () => stored(TITLE_BAR_KEY), (on) => { store(TITLE_BAR_KEY, on); this.onPrefs(loadPrefs(), ['customTitleBar']); });
    const menusIn = checkOf('Keep the menus in the window (next start)', () => stored(MENUS_KEY), (on) => store(MENUS_KEY, on));
    titleBar.disable(!('__TAURI_INTERNALS__' in window) || isMac);
    menusIn.disable(!('__TAURI_INTERNALS__' in window) || !isMac);

    add('General',
      grp('Startup', toggle('autoShowHome', 'Show Home screen at launch').el, toggle('homeOnLastClose', 'Show Home screen when the last document closes').el, later(toggle('updateAtStart', 'Look for updates at start')), soon()),
      grp('Recent Files', slider('recentFileCount', 'Recent files listed', 0, 40, 1).el),
      grp('Reset', reset));

    for (const [id, th] of Object.entries(THEMES)) {
      const sw = document.createElement('button');
      sw.className = 'nm-theme-swatch';
      sw.type = 'button';
      sw.dataset.themeId = id;
      sw.title = th.label;
      sw.style.background = th.vars['--bg-secondary'];
      sw.style.borderColor = th.vars['--border-color'];
      this.grid.append(sw);
    }
    const accentRow = el('<label class="cv-field"><span>Accent</span></label>');
    accentRow.appendChild(this.accent);
    const intensity = sliderRow('Accent intensity (%)', 0, 150, 5, (v) => setIntensity(v / 100), 100);
    this.rows.set('themeAuto', intensity as unknown as Row<never>);
    const lights = Object.entries(THEMES).filter(([, th]) => th.light).map(([k, th]) => [k, th.label] as const);
    const darks = Object.entries(THEMES).filter(([, th]) => !th.light).map(([k, th]) => [k, th.label] as const);
    this.intensity = intensity;
    add('Interface',
      grp('Theme', this.grid, accentRow, intensity.el, later(toggle('themeAuto', 'Match system appearance')), cols(later(pickRow('themeLight', 'Light theme', lights)), later(pickRow('themeDark', 'Dark theme', darks))), soon(),
        note('Right-click a theme to favorite it. Theme and accent are shared with the other NodeMode apps on this device.')),
      grp('Language and Text', pickRow('language', 'Language', languages() as ReadonlyArray<readonly [string, string]>, (v) => { void setLanguage(v as Lang); return v; }).el,
        pickRow('uiFontSize', 'Interface text size', FONT_SIZES.map((s) => [s, s[0].toUpperCase() + s.slice(1)] as const)).el),
      grp('Motion and Notices', toggle('animations', 'Enable animations').el, slider('noticeSeconds', 'Notice stays (seconds, 0 is automatic)', 0, 30, 1).el));

    add('Workspace',
      grp('Layout', pickRow('defaultLayout', 'Layout at start', [['split', 'Pattern and 3D'], ['2d', 'Pattern only'], ['3d', '3D only']]).el, toggle('rememberLayout', 'Remember the layout between sessions').el),
      grp('Window', titleBar.el, menusIn.el),
      grp('Settings File', ...(['app.settings_export', 'app.settings_import'] as const).map((id) => {
        const b = el<HTMLButtonElement>(`<button class="cv-btn" type="button" data-cmd="${id}">${id === 'app.settings_export' ? 'Export Settings...' : 'Import Settings...'}</button>`);
        b.addEventListener('click', () => document.dispatchEvent(new CustomEvent('ff:command', { detail: { id } })));
        return b;
      }), note('Preferences, shortcuts and saved layouts, as one JSON file.')));

    add('Performance',
      grp('Graphics', later(toggle('useGpu', 'Use the graphics processor')),
        later(pickRow('framePacingMs', 'Frame pacing', [['0', 'Display rate'], ['16', '60 frames a second'], ['33', '30 frames a second']], Number)), soon()),
      grp('Threads', later(slider('workerThreads', 'Worker threads (0 is automatic)', 0, 16, 1)), soon()));

    add('History & Cache',
      grp('Undo', later(slider('historyStates', 'History steps kept', 10, 1000, 10)), later(slider('historyCapMb', 'History kept in projects (MB, 0 is none)', 0, 512, 16)), soon()));

    add('Pattern',
      grp('Units', pickRow('units', 'Units', UNITS.map((u) => [u, u] as const)).el, note('Lengths are stored in millimetres; this only changes how they are shown and typed.')),
      grp('Grid and Snapping', num('gridSize', 'Grid size (mm)', 1, 1000, 1).el, toggle('snapGrid', 'Snap to the grid').el, toggle('snapPoints', 'Snap to points').el, toggle('snapEdges', 'Snap to edges').el),
      grp('Sewing', num('seamAllowance', 'Default seam allowance (mm)', 0, 50, 0.5).el));

    const drape = [
      num('simSubsteps', 'Substeps per frame', 1, 64, 1),
      num('simGravity', 'Gravity (m/s2)', 0, 30, 0.01),
      num('simThickness', 'Collision thickness (mm)', 0.1, 20, 0.1),
      pickRow('simDevice', 'Solver device', [['auto', 'Automatic'], ['cpu', 'Processor'], ['gpu', 'Graphics processor']]),
    ];
    drape[3].disable(true);
    add('Drape', grp('Simulation', ...drape.map((r) => r.el), soon(), note('Changes apply to the simulation as you make them.')));

    add('Viewport',
      grp('Renderer', later(pickRow('renderer', 'Renderer', [['auto', 'Automatic'], ['webgpu', 'WebGPU'], ['webgl', 'WebGL']])),
        later(pickRow('vpAntialias', 'Antialiasing', [['0', 'Off'], ['2', '2x'], ['4', '4x']], Number)), soon()),
      grp('Scene', later(toggle('vpShadows', 'Shadows')), later(toggle('vpFloor', 'Show the floor')),
        later(pickRow('vpBackground', 'Background', [['theme', 'Theme'], ['dark', 'Dark'], ['light', 'Light']])), soon()));

    const mcp = el<HTMLInputElement>('<input class="cv-input" type="text" readonly aria-label="Agent address" spellcheck="false">');
    mcp.value = `${location.origin}/api/mcp`;
    const mcpRow = el('<label class="cv-field"><span>Agent address</span></label>');
    mcpRow.appendChild(mcp);
    const nodemode = el<HTMLInputElement>('<input class="cv-input" type="url" aria-label="NodeMode address" spellcheck="false">');
    nodemode.addEventListener('change', () => this.change({ nodemodeUrl: nodemode.value.trim() }));
    this.nodemode = nodemode;
    const nodemodeRow = el('<label class="cv-field"><span>NodeMode address</span></label>');
    nodemodeRow.appendChild(nodemode);
    nodemode.disabled = true;
    nodemodeRow.classList.add('is-off');
    add('Plugins',
      grp('Connected Apps', toggle('agentsAllowed', 'Allow outside agents').el, mcpRow,
        note('Agents connect to the address above; every command in the menus is also a tool for them.'), nodemodeRow, soon()),
      grp('Plugins', note('No plugins are installed.')));

    this.pagesEl = pages;
    const wrap = el('<div class="cv-prefs"></div>');
    wrap.append(pages);
    this.body.appendChild(wrap);
    const frame = q<HTMLElement>(this.scrim, '.cv-dialog');
    frame.classList.add('cv-prefs-dlg');
    q<HTMLElement>(this.scrim, '.cv-dialog-head').prepend(el(`<span class="cv-prefs-headicon">${icon('settings', 16)}</span>`));
    let collapsed = stored(RAIL_KEY);
    const fold = el<HTMLButtonElement>(`<button class="cv-prefs-fold" type="button">${icon('pfFold', 16)}</button>`);
    const paintRail = (): void => {
      setClass(frame, 'is-rail-collapsed', collapsed);
      fold.title = t(collapsed ? 'Expand tabs' : 'Collapse tabs');
      fold.setAttribute('aria-label', fold.title);
      fold.setAttribute('aria-expanded', String(!collapsed));
    };
    fold.addEventListener('click', () => { collapsed = !collapsed; store(RAIL_KEY, collapsed); paintRail(); this.queueWire(); });
    paintRail();
    onLanguage(paintRail);
    const rail = el('<div class="cv-prefs-rail"></div>');
    rail.append(this.nav, fold);
    const wire = el<HTMLElement>('<svg class="cv-prefs-wire" aria-hidden="true"><path class="cv-prefs-wire-shade"></path><path class="cv-prefs-wire-line"></path></svg>') as unknown as SVGSVGElement;
    const port = el('<span class="cv-prefs-port" aria-hidden="true"></span>');
    frame.append(rail, wire, port);
    this.shell = { frame, rail, port, shade: q<SVGPathElement>(wire, '.cv-prefs-wire-shade'), line: q<SVGPathElement>(wire, '.cv-prefs-wire-line') };
    pages.addEventListener('scroll', () => this.queueSpy(), { passive: true });
    new ResizeObserver(() => this.queueWire()).observe(frame);
    window.addEventListener('resize', () => this.queueWire());
    this.button('Done', 'primary', () => this.close());
    this.grid.addEventListener('contextmenu', (e) => {
      const id = (e.target as HTMLElement).closest<HTMLElement>('[data-theme-id]')?.dataset.themeId;
      if (!id) return;
      e.preventDefault();
      const cur = this.prefs.favouriteThemes;
      this.change({ favouriteThemes: cur.includes(id) ? cur.filter((x) => x !== id) : [...cur, id] });
      this.sync();
    });
    this.grid.addEventListener('click', (e) => {
      const b = (e.target as HTMLElement).closest<HTMLElement>('[data-theme-id]');
      if (!b?.dataset.themeId) return;
      setTheme(b.dataset.themeId);
      this.sync();
    });
    this.accent.addEventListener('input', () => { setAccent(this.accent.value); this.sync(); });
    this.showSection('General');
  }

  private intensity!: Row<number>;
  private nodemode!: HTMLInputElement;

  private keep<T>(key: keyof Prefs, row: Row<T>): Row<T> {
    this.rows.set(key, row as unknown as Row<never>);
    return row;
  }

  show(section?: string): void {
    this.prefs = loadPrefs();
    for (const [key, row] of this.rows) {
      if (key === 'themeAuto' && row === (this.intensity as unknown as Row<never>)) continue;
      const v = this.prefs[key];
      (row as unknown as Row<unknown>).set(typeof v === 'number' && ['framePacingMs', 'vpAntialias'].includes(key) ? String(v) : v);
    }
    this.rows.get('language')?.set(language() as never);
    this.intensity.set(Math.round((readChoice().intensity ?? 1) * 100));
    setValue(this.nodemode, this.prefs.nodemodeUrl);
    this.sync();
    if (section && this.sections.has(section)) this.showSection(section);
    this.open();
    if (section && this.sections.has(section)) this.scrollToSection(section, 'auto');
    this.queueWire();
  }

  private change(patch: Partial<Prefs>): void {
    this.prefs = updatePrefs(patch);
    this.onPrefs(this.prefs, Object.keys(patch));
  }

  private sync(): void {
    const c = readChoice();
    setValue(this.accent, c.accentColor);
    for (const b of Array.from(this.grid.querySelectorAll<HTMLElement>('[data-theme-id]'))) {
      const id = b.dataset.themeId ?? '';
      const active = id === c.theme;
      setClass(b, 'is-active', active);
      const fav = this.prefs.favouriteThemes.includes(id);
      setClass(b, 'is-fav', fav);
      const order = fav ? '-1' : '0';
      if (b.style.order !== order) b.style.order = order;
      const title = `${THEMES[id]?.label ?? id} (${fav ? 'favorite, right-click to remove' : 'right-click to add to favorites'})`;
      if (b.title !== title) b.title = title;
      const border = active ? c.accentColor : THEMES[id]?.vars['--border-color'] ?? '';
      if (b.dataset.border !== border) { b.dataset.border = border; b.style.borderColor = border; }
    }
  }

  private showSection(name: string, scroll: ScrollBehavior | null = null): void {
    if (!this.sections.has(name)) return;
    this.markSection(name);
    if (scroll) this.scrollToSection(name, scroll);
  }

  private markSection(name: string): void {
    if (this.current === name) return;
    this.current = name;
    for (const [n, page] of this.sections) setClass(page, 'is-current', n === name);
    for (const tab of Array.from(this.nav.children) as HTMLElement[]) { const on = tab.dataset.section === name; setClass(tab, 'is-active', on); tab.setAttribute('aria-selected', String(on)); }
    this.queueWire();
  }

  private scrollToSection(name: string, behavior: ScrollBehavior): void {
    const sec = this.sections.get(name);
    if (!sec) return;
    if (sec.classList.contains('is-folded')) this.foldSection(name);
    const top = sec.getBoundingClientRect().top - this.pagesEl.getBoundingClientRect().top + this.pagesEl.scrollTop - 12;
    this.spyMuted = performance.now() + 700;
    this.pagesEl.scrollTo({ top, behavior });
  }

  private foldSection(name: string): void {
    const sec = this.sections.get(name);
    if (!sec) return;
    const folded = !sec.classList.contains('is-folded');
    setClass(sec, 'is-folded', folded);
    setHidden(q(sec, '.cv-prefs-secbody'), folded);
    q(sec, '.cv-prefs-sechead').setAttribute('aria-expanded', String(!folded));
    this.queueWire();
    this.queueSpy();
  }

  private queueSpy(): void {
    if (this.spyRaf) return;
    this.spyRaf = requestAnimationFrame(() => {
      this.spyRaf = 0;
      const pages = this.pagesEl;
      if (!pages.clientHeight || performance.now() < this.spyMuted) return;
      const edge = pages.getBoundingClientRect().top + 24;
      const names = Array.from(this.sections.keys());
      let best = '', bestD = Infinity;
      if (pages.scrollHeight > pages.clientHeight + 4 && pages.scrollTop + pages.clientHeight >= pages.scrollHeight - 2) best = names[names.length - 1] ?? '';
      else for (const [n, sec] of this.sections) {
        const top = sec.getBoundingClientRect().top, d = top <= edge ? edge - top : top - edge + 100000;
        if (d < bestD) { bestD = d; best = n; }
      }
      if (best) this.markSection(best);
    });
  }

  private queueWire(): void {
    if (this.wireRaf) return;
    this.wireRaf = requestAnimationFrame(() => { this.wireRaf = 0; this.drawWire(); });
  }

  private drawWire(): void {
    const s = this.shell;
    if (this.scrim.hidden) return;
    const F = s.frame.getBoundingClientRect();
    const tab = this.nav.querySelector<HTMLElement>('.cv-prefs-railtab.is-active');
    const head = s.frame.querySelector<HTMLElement>('.cv-dialog-head');
    if (!F.width || !tab || !head) return;
    const N = this.nav.getBoundingClientRect();
    let T = tab.getBoundingClientRect();
    if (T.top < N.top || T.bottom > N.bottom) {
      this.nav.scrollTop += T.top < N.top ? T.top - N.top - 6 : T.bottom - N.bottom + 6;
      T = tab.getBoundingClientRect();
    }
    const R = s.rail.getBoundingClientRect(), H = head.getBoundingClientRect();
    const portY = Math.round(H.bottom - F.top), sx = Math.round(T.right - F.left), ty = Math.round(T.top - F.top + T.height / 2);
    const rr = R.right - F.left, trunk = Math.round(rr - rr * 0.45), dy = portY - ty, r = 10, dir = dy > 0 ? 1 : -1;
    const d = Math.abs(dy) < 2 * r ? `M ${sx} ${ty} H ${trunk} V ${portY} H -7` : `M ${sx} ${ty} H ${trunk - r} a ${r} ${r} 0 0 ${dir > 0 ? 1 : 0} ${r} ${dir * r} V ${portY - dir * r} a ${r} ${r} 0 0 ${dir > 0 ? 0 : 1} ${r} ${dir * r} H -7`;
    if (d !== this.wireD) { this.wireD = d; s.shade.setAttribute('d', d); s.line.setAttribute('d', d); }
    if (portY !== this.portY) { this.portY = portY; s.port.style.transform = `translateY(${portY}px)`; }
  }
}

export { LAYOUTS };
