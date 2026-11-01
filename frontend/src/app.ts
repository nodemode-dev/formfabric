import { VERSION } from './version';
import { call, catalogue, fetchVersion, onEngineChanged, onEngineLink, serviceUrl, type CatalogueCommand, type Link } from './engine/service';
import { mountDock, type DockHandle } from './dock';
import { mountWorkspace } from './workspace';
import { CaptionButtons } from './ui/caption-buttons';
import { Commands, engineCommands, hasRequired } from './ui/commands';
import { CommandPalette } from './ui/command-palette';
import { Dialog, anyDialogOpen } from './ui/dialogs';
import { el, q, revealApp, setText } from './ui/dom';
import { enhanceAllSelects } from './ui/dropdown';
import { exactInput } from './ui/exact-input';
import { icon } from './ui/icons';
import { brandMark } from './ui/brand';
import { language, onLanguage, t } from './ui/i18n';
import { MenuBar } from './ui/menu';
import { NativeMenus, textFieldFocused, wantsNativeMenus } from './ui/native-menu';
import { installNumberFields } from './ui/number-fields';
import { loadPrefs, PREFS_EVENT, type Prefs } from './ui/prefs';
import { SettingsDialog } from './ui/settings';
import { ShortcutsDialog, applyShortcuts } from './ui/shortcuts';
import { applyTheme } from './ui/themes';
import { toast } from './ui/toast';
import type { ShellDeps } from './shell/deps';
import { HomeScreen } from './shell/home';
import { probeGpu } from './shell/gpu';
import { LayoutState } from './shell/layout';
import { buildMenus } from './shell/menus';
import { Session } from './shell/session';
import { StatusBar } from './shell/statusbar';
import { TitleBar } from './shell/titlebar';
import { registerWindowCommands } from './shell/window-commands';

const SHELL = '<div class="cv-app"><header class="cv-titlebar"></header><main class="cv-main"><div class="cv-workspace cv-stage" data-region="workspace"></div></main><section class="cv-dock" data-region="dock"></section><footer class="cv-status"></footer></div>';
const FONT_BASE = [9, 10, 11, 12, 13, 14, 16, 18];
const FONT_SCALE = { small: 0.92, normal: 1, large: 1.1 };
const message = (err: unknown): string => (err instanceof Error ? err.message : String(err));

class AboutDialog extends Dialog {
  constructor() {
    super('About FormFabric', 380);
    this.body.append(el(`<div class="cv-about">${brandMark(48)}<p class="cv-about-name">FormFabric</p><p class="cv-note">Version ${VERSION}</p><p class="cv-note">Garment pattern and cloth simulation. Apache-2.0, by NodeMode.</p></div>`));
    this.button('Close', 'primary', () => this.close());
  }
}

function applyPrefs(p: Prefs): void {
  const k = FONT_SCALE[p.uiFontSize];
  FONT_BASE.forEach((n) => document.documentElement.style.setProperty(`--fs-${n}`, `${Math.round(n * k * 10) / 10}px`));
  document.body.classList.toggle('animations-disabled', !p.animations);
}

export function startApp(): void {
  applyTheme();
  installNumberFields();
  enhanceAllSelects();
  const root = q(document, '#app');
  root.innerHTML = SHELL;
  const app = q(root, '.cv-app');
  applyPrefs(loadPrefs());

  const commands = new Commands();
  const session = new Session();
  const layout = new LayoutState();
  const status = new StatusBar(q(app, '.cv-status'));
  let link: Link = 'connecting';
  let loaded = false;
  let version = '';
  commands.blocked = (cmd) => (link === 'down' && !cmd.id.startsWith('app.') ? t('The engine is not running') : '');

  const prefsListeners: Array<(p: Prefs, keys: string[]) => void> = [];
  const deps: ShellDeps = {
    commands, status, layout,
    engine: { call, catalogue, onEngineChanged },
    toast,
    prefs: loadPrefs,
    onPrefs: (cb) => { prefsListeners.push(cb); },
  };

  const refresh = async (): Promise<void> => {
    try {
      const u = await call<{ units?: string }>('view.units', {});
      if (u.units) status.setUnits(u.units);
    } catch { /* the engine may lack view.units */ }
    if (commands.get('pattern.get')) {
      try { status.setPanels((await call<{ panels?: unknown[] }>('pattern.get', {})).panels?.length ?? 0); } catch { status.setPanels(null); }
    }
  };

  const runEngine = async (c: CatalogueCommand, params?: Record<string, unknown>): Promise<unknown> => {
    let args = params;
    if (!args && hasRequired(c.inputSchema)) {
      args = (await exactInput(t(c.label).replace(/\.\.\.$/, ''), c.inputSchema, c.id === 'file.save' || c.id === 'file.open' ? { path: session.path ?? undefined } : {})) ?? undefined;
      if (!args) return undefined;
    }
    if (c.id === 'file.save' && !args && session.path) args = { path: session.path };
    const end = status.working(c.label);
    try {
      const result = await call(c.id, args ?? {});
      const path = typeof args?.path === 'string' ? args.path : session.path;
      if (c.id === 'file.new') session.set(null, false);
      else if ((c.id === 'file.open' || c.id === 'file.save') && path) { session.set(path, false); session.remember(path, loadPrefs().recentFileCount); }
      void refresh();
      return result;
    } catch (err) {
      toast(message(err), 'error');
      status.setHint(message(err), true);
      return undefined;
    } finally { end(); }
  };

  const load = async (): Promise<void> => {
    if (loaded) return;
    try {
      const cat = await catalogue(true);
      loaded = true;
      status.setEngine('up', cat.version);
      version = cat.version;
      commands.add(...engineCommands(cat, (c, p) => runEngine(c, p)));
      applyShortcuts(commands);
      native?.refresh();
      home.refresh();
      void refresh();
    } catch { loaded = false; }
  };

  let native: NativeMenus | null = null;
  const openPath = (path: string): void => { void commands.call('file.open', { path }); };
  const settings = new SettingsDialog((p, keys) => {
    applyPrefs(p);
    if (keys.includes('units')) void commands.call('view.units', { units: p.units }).then(() => refresh());
    for (const cb of prefsListeners) cb(p, keys);
  });
  const shortcuts = new ShortcutsDialog(commands);
  const about = new AboutDialog();
  const titlebar = new TitleBar(q(app, '.cv-titlebar'), commands, () => undefined);
  const captions = new CaptionButtons(q(app, '.cv-titlebar'), () => localStorage.getItem('formfabric.customTitleBar') === '1');
  document.addEventListener('ff:title-bar', () => captions.sync());
  document.addEventListener(PREFS_EVENT, () => captions.sync());

  const menus = () => buildMenus(commands, () => session.recent());
  const palette = new CommandPalette(commands, menus);
  const home = new HomeScreen(root, {
    autoShow: () => loadPrefs().autoShowHome,
    afterClose: () => loadPrefs().homeOnLastClose,
    recent: () => session.recent(),
    openPath,
    hasProject: () => session.active,
    can: (id) => { const c = commands.get(id); return !!c && commands.enabled(c); },
    run: (id) => { commands.run(id); },
  });

  registerWindowCommands({
    commands, layout, session,
    openSettings: (s) => settings.show(s),
    openPalette: () => palette.show(),
    openShortcuts: () => shortcuts.show(),
    openAbout: () => about.open(),
    showHome: () => home.show(),
    workspace: (id) => titlebar.choose(id),
    setUnits: (u) => { status.setUnits(u); void commands.call('view.units', { units: u }); },
    openPath,
  });
  applyShortcuts(commands);

  new MenuBar(q(app, '.cv-menubar'), commands, menus());
  if (wantsNativeMenus()) {
    native = new NativeMenus(commands, menus);
    void native.build().catch(() => undefined);
    onLanguage(() => { void native?.build().catch(() => undefined); });
  }

  status.onUnits = () => {
    const order = ['mm', 'cm', 'in'] as const;
    commands.run(`app.units_${order[(order.indexOf(loadPrefs().units) + 1) % order.length]}`);
  };

  const title = (): void => {
    const text = session.title();
    document.title = text;
    setText(q(app, '.cv-tb-doc'), text);
    if ('__TAURI_INTERNALS__' in window) void import('@tauri-apps/api/core').then((m) => m.invoke('set_window_title', { title: text })).catch(() => undefined);
    home.refresh();
    home.sync();
    native?.refresh();
  };
  session.onChange(title);
  title();

  document.addEventListener('keydown', (e) => {
    if (e.defaultPrevented || anyDialogOpen()) return;
    const cmd = commands.forEvent(e);
    if (!cmd) return;
    const field = textFieldFocused();
    const native = ['history.undo', 'history.redo', 'app.cut', 'app.copy', 'app.paste', 'app.select_all'].includes(cmd.id);
    if (field && (native || !(e.ctrlKey || e.metaKey))) return;
    e.preventDefault();
    commands.run(cmd.id);
  });

  onEngineLink((state) => {
    link = state;
    status.setEngine(state, version);
    if (state === 'up') void load().then(() => { home.refresh(); });
    if (state === 'down') { loaded = false; }
    home.refresh();
  });
  onEngineChanged((e) => {
    if (e.name.startsWith('file.')) return;
    session.touch();
    void refresh();
  });
  void fetchVersion().then((v) => { if (v) { version = v.version; status.setEngine(link, v.version); } });
  void probeGpu().then((g) => status.setGpu(g));
  void load();
  onLanguage(() => { document.documentElement.lang = language(); });

  const mountPoint = q<HTMLElement>(app, '[data-region="workspace"]');
  mountWorkspace(mountPoint, deps);
  const dock: DockHandle = mountDock(q(app, '[data-region="dock"]'), {
    t, icons: { icon }, serviceUrl, language, onFail: (err) => toast(message(err), 'error'),
  });
  dock.onChange(() => titlebar.syncDock(dock.isOpen(), dock.isOpen() ? 'shell' : null));
  document.addEventListener('ff:dock-toggle', (e) => {
    const kind = (e as CustomEvent<{ kind: 'shell' | 'claude' }>).detail.kind;
    void dock.toggleKind(kind);
  });

  const fonts = (document as Document & { fonts?: { ready: Promise<unknown> } }).fonts;
  void Promise.race([fonts?.ready ?? Promise.resolve(), new Promise((r) => setTimeout(r, 2500))]).then(() => revealApp());
}
