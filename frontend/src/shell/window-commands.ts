import type { Command, Commands } from '../ui/commands';
import { promptName } from '../ui/dialogs';
import { t } from '../ui/i18n';
import { textFieldFocused } from '../ui/native-menu';
import { loadPrefs, updatePrefs, UNITS, type Prefs } from '../ui/prefs';
import { PREF_SECTIONS } from '../ui/settings';
import { readChoice, setTheme } from '../ui/themes';
import type { LayoutState } from './layout';
import type { Session } from './session';
import { WORKSPACES } from './titlebar';

export interface WindowCtx {
  commands: Commands;
  layout: LayoutState;
  session: Session;
  openSettings: (section?: string) => void;
  openPalette: () => void;
  openShortcuts: () => void;
  openAbout: () => void;
  showHome: () => void;
  workspace: (id: string) => void;
  setUnits: (u: string) => void;
  openPath: (path: string) => void;
}

export const SAMPLES: ReadonlyArray<readonly [string, string]> = [['tshirt', 'T-shirt'], ['skirt', 'A-line skirt'], ['dress', 'Sleeveless dress']];
export const SAMPLE_BODIES: ReadonlyArray<readonly [string, string, string | null]> = [['current', 'Current body', null], ['female', 'MPFB2 Female', 'MPFB2 Female'], ['male', 'MPFB2 Male', 'MPFB2 Male']];

const REPO = 'https://github.com/nodemode-dev/formfabric';
const inTauri = (): boolean => '__TAURI_INTERNALS__' in window;
const isMac = /Mac/.test(navigator.platform);

function edit(action: 'cut' | 'copy' | 'paste' | 'selectAll'): void {
  if (textFieldFocused()) { document.execCommand(action); return; }
  document.dispatchEvent(new CustomEvent('ff:edit', { detail: { action } }));
}

export function registerWindowCommands(ctx: WindowCtx): void {
  const { commands, layout, session } = ctx;
  const toggle = (key: keyof Prefs, id: string, label: string): Command => ({
    id, label, checked: () => loadPrefs()[key] === true, run: () => { updatePrefs({ [key]: !loadPrefs()[key] } as Partial<Prefs>); },
  });
  const list: Command[] = [
    { id: 'app.none', label: 'Nothing here yet', enabled: () => false, run: () => undefined },
    { id: 'app.settings', label: 'Preferences...', keys: ['Mod+,'], icon: 'settings', run: () => ctx.openSettings() },
    { id: 'app.palette', label: 'Search commands...', keys: ['Mod+K'], icon: 'search', run: () => ctx.openPalette() },
    { id: 'app.shortcuts', label: 'Keyboard Shortcuts...', keys: ['Mod+/'], run: () => ctx.openShortcuts() },
    { id: 'app.home', label: 'Launcher', run: () => ctx.showHome() },
    { id: 'app.about', label: 'About FormFabric', run: () => ctx.openAbout() },
    { id: 'app.source', label: 'Source Code', run: () => { window.open(REPO, '_blank', 'noopener'); } },
    { id: 'app.report_issue', label: 'Report an Issue...', run: () => { window.open(`${REPO}/issues`, '_blank', 'noopener'); } },
    { id: 'app.cut', label: 'Cut', keys: ['Mod+X'], run: () => edit('cut') },
    { id: 'app.copy', label: 'Copy', keys: ['Mod+C'], run: () => edit('copy') },
    { id: 'app.paste', label: 'Paste', keys: ['Mod+V'], run: () => edit('paste') },
    { id: 'app.select_all', label: 'Select All', keys: ['Mod+A'], run: () => edit('selectAll') },
    { id: 'app.layout_2d', label: 'Pattern Only', icon: 'layout2d', checked: () => layout.get() === '2d', run: () => layout.set('2d') },
    { id: 'app.layout_split', label: 'Pattern and 3D', icon: 'layoutSplit', checked: () => layout.get() === 'split', run: () => layout.set('split') },
    { id: 'app.layout_3d', label: '3D Only', icon: 'layout3d', checked: () => layout.get() === '3d', run: () => layout.set('3d') },
    toggle('snapGrid', 'app.snap_grid', 'Snap to Grid'),
    toggle('snapPoints', 'app.snap_points', 'Snap to Points'),
    toggle('snapEdges', 'app.snap_edges', 'Snap to Edges'),
    { id: 'app.theme_toggle', label: 'Light or Dark', run: () => setTheme(readChoice().theme === 'light' ? 'studio' : 'light') },
    { id: 'app.custom_title_bar', label: 'Custom Title Bar', enabled: () => inTauri() && !isMac, checked: () => localStorage.getItem('formfabric.customTitleBar') === '1', run: () => { localStorage.setItem('formfabric.customTitleBar', localStorage.getItem('formfabric.customTitleBar') === '1' ? '0' : '1'); document.dispatchEvent(new CustomEvent('ff:title-bar')); } },
    { id: 'app.menus_in_window', label: 'Menus in Window', enabled: () => inTauri() && isMac, checked: () => localStorage.getItem('formfabric.menusInWindow') === '1', run: () => { localStorage.setItem('formfabric.menusInWindow', localStorage.getItem('formfabric.menusInWindow') === '1' ? '0' : '1'); } },
    { id: 'app.terminal', label: 'Show Terminal', icon: 'terminal', keys: [], run: () => { document.dispatchEvent(new CustomEvent('ff:dock-toggle', { detail: { kind: 'shell' } })); } },
    { id: 'app.claude', label: 'Claude Code', icon: 'agent', run: () => { document.dispatchEvent(new CustomEvent('ff:dock-toggle', { detail: { kind: 'claude' } })); } },
    { id: 'app.save_as', label: 'Save As...', keys: ['Mod+Shift+S'], run: async () => {
      const path = await promptName('Save As', session.path ?? 'untitled.ffbrc', 'Save');
      if (path) await commands.call('file.save', { path });
    } },
    { id: 'app.clear_recent', label: 'Clear Recent', run: () => session.forget() },
    { id: 'app.exit', label: 'Exit', run: () => {
      if (inTauri()) void import('@tauri-apps/api/window').then((m) => m.getCurrentWindow().close());
      else window.close();
    } },
  ];
  for (const u of UNITS) list.push({ id: `app.units_${u}`, label: u === 'mm' ? 'Millimetres' : u === 'cm' ? 'Centimetres' : 'Inches', checked: () => loadPrefs().units === u, run: () => { updatePrefs({ units: u }); ctx.setUnits(u); } });
  for (const s of PREF_SECTIONS) list.push({ id: `app.prefs.${s}`, label: s, run: () => ctx.openSettings(s) });
  for (const w of WORKSPACES) list.push({ id: `app.ws_${w.id}`, label: w.label, enabled: () => w.ready, run: () => ctx.workspace(w.id) });
  for (let i = 0; i < 10; i++) list.push({ id: `app.recent.${i}`, label: () => session.recent()[i]?.split(/[\\/]/).pop() ?? '', enabled: () => !!session.recent()[i], run: () => { const p = session.recent()[i]; if (p) ctx.openPath(p); } });
  const openSample = async (name: string, body: string | null): Promise<void> => {
    const loaded = await commands.call('sample.load', body ? { name, body } : { name });
    if (!loaded.ran) return;
    session.set(null, false);
    await commands.call('sim.reset', {});
    layout.set('3d');
    const canvas = document.querySelector<HTMLElement>('[data-k="canvas3d"]');
    if (canvas) {
      if (!canvas.hasAttribute('tabindex')) canvas.tabIndex = -1;
      canvas.focus({ preventScroll: true });
    }
  };
  for (const [name, sample] of SAMPLES) {
    for (const [slot, text, body] of SAMPLE_BODIES) {
      list.push({ id: `app.sample.${name}.${slot}`, label: () => `${t(sample)}: ${t(text)}`, enabled: () => !!commands.get('sample.load'), run: () => openSample(name, body) });
    }
  }
  commands.add(...list);
}
