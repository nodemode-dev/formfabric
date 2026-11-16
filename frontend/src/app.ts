import { VERSION } from './version';
import { call, catalogue, fetchVersion, onEngineChanged, onEngineLink, serviceUrl, type CatalogueCommand, type Link } from './engine/service';
import { mountDock, type DockHandle } from './dock';
import { mountWorkspace } from './workspace';
import { CaptionButtons } from './ui/caption-buttons';
import { Commands, engineCommands, hasRequired } from './ui/commands';
import { CommandPalette } from './ui/command-palette';
import { Dialog, anyDialogOpen, askSaveChanges } from './ui/dialogs';
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
import { mountHome } from './shell/home/index';
import { api as libraryApi } from './shell/home/api';
import { saveThumb } from './shell/home/thumb';
import { DEMOS } from './ui/demos';
import { confirmExit, inDesktop, listenShell } from './shell/desktop';
import { probeGpu } from './shell/gpu';
import { LayoutState } from './shell/layout';
import { buildMenus } from './shell/menus';
import { dxfCommand, nativeDialogs, pickPath } from './shell/file-dialog';
import { Session } from './shell/session';
import { StatusBar } from './shell/statusbar';
import { TitleBar } from './shell/titlebar';
import { registerWindowCommands } from './shell/window-commands';
import { KeyHints } from './shell/keyhints';
import type { WorkspaceLayouts } from './shell/layouts';
import { offerRecovery } from './shell/recovery';
import { layoutSlotIds, registerWorkspaceCommands } from './shell/workspace-commands';

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

  status.setUnits(loadPrefs().units);
  const refresh = async (): Promise<void> => {
    status.setUnits(loadPrefs().units);
    if (commands.get('pattern.get')) {
      try { status.setPanels((await call<{ panels?: unknown[] }>('pattern.get', {})).panels?.length ?? 0); } catch { status.setPanels(null); }
    }
  };

  const savePlace = async (): Promise<string | null> => session.path ?? (await libraryApi.library().catch(() => null))?.untitled ?? null;

  const runEngine = async (c: CatalogueCommand, params?: Record<string, unknown>): Promise<unknown> => {
    let args = params;
    if ((c.id === 'file.new' || c.id === 'file.open') && session.dirty) {
      const choice = await askSaveChanges(session.name());
      if (choice === 'cancel') return undefined;
      if (choice === 'save' && (await commands.call('file.save')).result === undefined) return undefined;
    }
    const filing = c.id === 'file.open' || c.id === 'file.save';
    if (!args && filing && nativeDialogs() && !(c.id === 'file.save' && session.path)) {
      const picked = await pickPath(c.id === 'file.open' ? 'open' : 'save', c.id === 'file.save' ? await savePlace() : session.path);
      if (!picked) return undefined;
      args = { path: picked };
    }
    const dxf = dxfCommand(c.id);
    const dxfName = session.path ? `${session.path.replace(/\.ffbrc$/i, '')}.dxf` : null;
    if (!args && dxf && nativeDialogs()) {
      const picked = await pickPath(dxf, dxf === 'save' ? dxfName : null, 'dxf');
      if (!picked) return undefined;
      args = { path: picked };
    }
    if (c.id === 'file.save' && !args && session.path) args = { path: session.path };
    if (!args && hasRequired(c.inputSchema)) {
      const initial = c.id === 'file.save' ? { path: (await savePlace()) ?? undefined } : c.id === 'file.open' ? { path: session.path ?? undefined } : dxf === 'save' && dxfName ? { path: dxfName } : {};
      args = (await exactInput(t(c.label).replace(/\.\.\.$/, ''), c.inputSchema, initial)) ?? undefined;
      if (!args) return undefined;
    }
    const end = status.working(c.label);
    try {
      const result = await call(c.id, args ?? {});
      const path = typeof args?.path === 'string' ? args.path : session.path;
      if (c.id === 'file.new') session.set(null, false);
      else if ((c.id === 'file.open' || c.id === 'file.save') && path) { session.set(path, false); session.remember(path, loadPrefs().recentFileCount); void session.pull(); }
      if (c.id === 'file.save' && path) void saveThumb(path).then(() => home.refresh());
      else if (c.id === 'file.new' || c.id === 'file.open') home.leave();
      void refresh();
      return result;
    } catch (err) {
      toast(message(err), 'error');
      status.setHint(message(err), true);
      return undefined;
    } finally { end(); }
  };

  let engineUp: () => void = () => undefined;
  const engineReady = new Promise<void>((done) => { engineUp = done; });
  const adopt = (): void => {
    if (!session.active) void call<{ panels?: unknown[] }>('pattern.get', {}).then((g) => { if (g.panels?.length && !session.active) session.set(null, true); }).catch(() => undefined);
  };
  const load = async (): Promise<void> => {
    if (loaded) return;
    try {
      const cat = await catalogue(true);
      if (link === 'down') return;
      loaded = true;
      status.setEngine('up', cat.version);
      version = cat.version;
      commands.add(...engineCommands(cat, (c, p) => runEngine(c, p)).filter((c) => !commands.get(c.id)));
      applyShortcuts(commands);
      engineUp();
      native?.refresh();
      home.refresh();
      void refresh();
      void session.pull();
      pushAgents();
      adopt();
      void offerRecovery((path) => { session.set(path, true); void refresh(); });
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
  let wsLayouts: WorkspaceLayouts | null = null;
  const titlebar = new TitleBar(q(app, '.cv-titlebar'), commands, (id) => { wsLayouts?.applyWorkspace(id); });
  const captions = new CaptionButtons(q(app, '.cv-titlebar'), () => localStorage.getItem('formfabric.customTitleBar') === '1');
  document.addEventListener('ff:title-bar', () => captions.sync());
  document.addEventListener(PREFS_EVENT, () => captions.sync());
  let agentsSent: Promise<unknown> = Promise.resolve();
  const pushAgents = (): void => {
    agentsSent = agentsSent.then(() => fetch(serviceUrl('/api/commands/agents.prefs'), { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ agentsAllowed: loadPrefs().agentsAllowed }) })).catch(() => undefined);
  };
  document.addEventListener(PREFS_EVENT, (e) => { if ((e as CustomEvent<{ keys?: string[] }>).detail?.keys?.includes('agentsAllowed')) pushAgents(); });

  const menus = () => buildMenus(commands, () => session.recent(), () => (wsLayouts ? layoutSlotIds(wsLayouts) : []));
  const palette = new CommandPalette(commands, menus);
  const openPattern = async (path: string): Promise<void> => {
    if (!/\.dxf$/i.test(path)) { await commands.call('file.open', { path }); return; }
    if ((await commands.call('file.new')).result === undefined) return;
    await commands.call('exchange.import_dxf', { path, mode: 'as_new' });
  };
  const closeGarment = async (): Promise<void> => {
    if (!session.active) return;
    if (session.dirty) {
      const choice = await askSaveChanges(session.name());
      if (choice === 'cancel') return;
      if (choice === 'save' && (await commands.call('file.save')).result === undefined) return;
    }
    try { await call('file.new', {}); } catch (err) { toast(message(err), 'error'); return; }
    session.close();
    void refresh();
  };
  let quitting = false;
  const exitApp = async (): Promise<void> => {
    if (quitting) return;
    quitting = true;
    try {
      let go = true;
      if (session.dirty) {
        const choice = await askSaveChanges(session.name());
        go = choice !== 'cancel' && (choice !== 'save' || (await commands.call('file.save')).result !== undefined);
      }
      if (inDesktop()) await confirmExit(!go);
      else if (go) { window.close(); toast(t('This page can be closed now.')); }
    } catch (err) {
      toast(message(err), 'error');
      if (inDesktop()) await confirmExit(true).catch(() => undefined);
    } finally { quitting = false; }
  };
  const home = mountHome(root, {
    openGarment: (g) => { if (session.active && session.path === g.path) home.leave(); else openPath(g.path); },
    closeGarment: (path) => {
      if (!session.active || session.path !== path) return;
      if (session.dirty) session.set(null, true);
      else void closeGarment();
    },
    createGarment: async (template, name) => {
      try {
        const made = await libraryApi.create(template, name);
        home.refresh();
        await commands.call('file.open', { path: made.path });
      } catch (err) { toast(message(err), 'error'); }
    },
    blankGarment: () => { commands.run('file.new'); },
    openFile: () => { commands.run('file.open'); },
    chooseFile: () => {
      if (!nativeDialogs()) return false;
      void pickPath('open', null, 'pattern').then((p) => (p ? openPattern(p) : undefined)).catch((err) => toast(message(err), 'error'));
      return true;
    },
    openFiles: (files) => {
      const file = files.find((f) => /\.(ffbrc|dxf)$/i.test(f.name)) ?? files[0];
      if (file) void libraryApi.importFile(file).then((got) => { home.refresh(); return openPattern(got.path); }).catch((err) => toast(message(err), 'error'));
    },
    openPaths: (paths) => {
      const path = paths.find((p) => /\.(ffbrc|dxf)$/i.test(p));
      if (path) void openPattern(path);
    },
    recentChanged: () => { void session.pull(); },
    preferences: () => settings.show(),
    agents: () => settings.show('Plugins'),
    demo: (id) => { const d = DEMOS.find((x) => x.id === id); if (d) commands.run(d.command); },
    hasGarment: () => session.active,
    autoShow: () => loadPrefs().autoShowHome,
    afterClose: () => loadPrefs().homeOnLastClose,
  });

  registerWindowCommands({
    commands, layout, session,
    openSettings: (s) => settings.show(s),
    openPalette: () => palette.show(),
    openShortcuts: () => shortcuts.show(),
    openAbout: () => about.open(),
    showHome: () => home.force('start'),
    closeGarment,
    savePlace,
    workspace: (id) => titlebar.choose(id),
    openOutput: (section) => {
      titlebar.choose('render');
      wsLayouts?.openDrawer('output');
      window.dispatchEvent(new CustomEvent('ffb:output-section', { detail: { section } }));
    },
    setUnits: (u) => { status.setUnits(u); void commands.call('view.units', { units: u }); },
    openPath,
  });
  applyShortcuts(commands);
  const keyHints = new KeyHints(commands);
  wsLayouts = registerWorkspaceCommands({ commands, layout, hints: keyHints });

  new MenuBar(q(app, '.cv-menubar'), commands, menus());
  if (wantsNativeMenus()) {
    native = new NativeMenus(commands, menus);
    void native.build().catch(() => undefined);
    onLanguage(() => { void native?.build().catch(() => undefined); });
  }
  void listenShell({
    open: async (paths) => {
      await engineReady;
      const path = paths.find((p) => /\.ffbrc$/i.test(p)) ?? paths[0];
      if (path) await commands.call('file.open', { path });
    },
    exitRequested: () => { void exitApp(); },
  }).catch(() => undefined);
  document.addEventListener('ff:exit', () => { void exitApp(); });

  status.onUnits = () => {
    const order = ['mm', 'cm', 'in'] as const;
    commands.run(`app.units_${order[(order.indexOf(loadPrefs().units) + 1) % order.length]}`);
  };

  const title = (): void => {
    const text = session.active ? session.title() : 'FormFabric';
    document.title = text;
    setText(q(app, '.cv-tb-doc'), text);
    if ('__TAURI_INTERNALS__' in window) void import('@tauri-apps/api/core').then((m) => m.invoke('set_window_title', { title: text })).catch(() => undefined);
    home.setOpenGarments(session.active ? [{ id: 'open', title: session.path ? session.name().replace(/\.ffbrc$/i, '') : '' }] : []);
    home.sync();
    native?.refresh();
  };
  session.onChange(title);
  title();

  document.addEventListener('keydown', (e) => {
    if (e.defaultPrevented || anyDialogOpen()) return;
    if (home.isOpen && !(e.ctrlKey || e.metaKey)) return;
    if (e.key === ' ' && e.target instanceof HTMLButtonElement && e.target.matches(':focus-visible')) return;
    const cmd = commands.forEvent(e);
    if (!cmd) return;
    const field = textFieldFocused();
    const own = ['history.undo', 'history.redo', 'app.cut', 'app.copy', 'app.paste', 'app.select_all'].includes(cmd.id);
    if (field && (own || !(e.ctrlKey || e.metaKey))) return;
    e.preventDefault();
    if (native && !native.claim(cmd.id, 'key')) return;
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
    native?.refresh();
    if (e.name.startsWith('file.')) return;
    session.touch();
    adopt();
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
    const { kind, on } = (e as CustomEvent<{ kind: 'shell' | 'claude'; on?: boolean }>).detail;
    if (on === undefined) void dock.toggleKind(kind);
    else if (on !== dock.isOpen()) dock.toggle(on);
  });

  const fonts = (document as Document & { fonts?: { ready: Promise<unknown> } }).fonts;
  void Promise.race([fonts?.ready ?? Promise.resolve(), new Promise((r) => setTimeout(r, 2500))]).then(() => revealApp());
}
