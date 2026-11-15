import type { Commands } from './commands';
import type { MenuDef, MenuItem as MenuSpec } from './menu';
import { anyDialogOpen } from './dialogs';
import { t } from './i18n';
import { VERSION } from '../version';

export const wantsNativeMenus = (): boolean => '__TAURI_INTERNALS__' in window && /Mac/.test(navigator.platform) && !menusInWindow();

export const MENUS_IN_WINDOW_KEY = 'formfabric.menusInWindow';
export function menusInWindow(): boolean {
  try { return localStorage.getItem(MENUS_IN_WINDOW_KEY) === '1'; } catch { return false; }
}

const NAMES: Record<string, string> = { Mod: 'CmdOrCtrl', Alt: 'Alt', Shift: 'Shift', Ctrl: 'Ctrl', Plus: 'Plus', '=': '=', '-': '-', '[': '[', ']': ']', ',': ',', '.': '.', '/': '/', ';': ';', "'": "'", '\\': '\\', '`': '`', Delete: 'Delete', Backspace: 'Backspace', Escape: 'Escape', Enter: 'Enter', Tab: 'Tab', Space: 'Space', Up: 'Up', Down: 'Down', Left: 'Left', Right: 'Right', ArrowUp: 'Up', ArrowDown: 'Down', ArrowLeft: 'Left', ArrowRight: 'Right', Home: 'Home', End: 'End', PageUp: 'PageUp', PageDown: 'PageDown' };

export function accelerator(combo: string | undefined): string | null {
  if (!combo) return null;
  const parts = combo.endsWith('++') ? [...combo.slice(0, -2).split('+'), 'Plus'] : combo.split('+');
  const key = parts[parts.length - 1];
  const fkey = /^F([1-9]|1[0-2])$/.test(key);
  if (!fkey && !parts.some((p) => p === 'Mod' || p === 'Ctrl' || p === 'Alt')) return null;
  const out: string[] = [];
  for (const p of parts) {
    if (NAMES[p]) out.push(NAMES[p]);
    else if (/^[A-Za-z0-9]$/.test(p)) out.push(p.toUpperCase());
    else if (/^F([1-9]|1[0-2])$/.test(p)) out.push(p);
    else return null;
  }
  return out.join('+');
}

const sameKeys = (acc: string): string => acc.split('+').sort().join('+');
const SYSTEM_KEYS: readonly string[] = ['CmdOrCtrl+H', 'CmdOrCtrl+Q', 'CmdOrCtrl+M', 'Ctrl+CmdOrCtrl+F'].map(sameKeys);
const COPYRIGHT = 'Copyright 2026 nodemode-ai. Apache-2.0';

type Api = typeof import('@tauri-apps/api/menu');
type Live = { setEnabled(on: boolean): Promise<void>; setChecked?(on: boolean): Promise<void> };
type Built = { id: string; item: Live; on: boolean };
type Read = Array<{ label: string; items: MenuSpec[] }>;

const TEXT_EDITS = new Map<string, string>([['history.undo', 'undo'], ['history.redo', 'redo'], ['app.cut', 'cut'], ['app.copy', 'copy'], ['app.paste', 'paste'], ['app.select_all', 'select-all']]);
const NOT_TEXT = new Set(['button', 'checkbox', 'radio', 'range', 'color', 'file', 'submit', 'reset', 'image']);

export function textFieldFocused(): boolean {
  const a = document.activeElement;
  if (a instanceof HTMLTextAreaElement) return !a.readOnly && !a.disabled;
  if (a instanceof HTMLInputElement) return !NOT_TEXT.has(a.type) && !a.readOnly && !a.disabled;
  return a instanceof HTMLElement && a.isContentEditable;
}

export class NativeMenus {
  private items: Built[] = [];
  private building = false;
  private again = false;
  private late = false;
  private stateTimer = 0;
  private shape = '';
  private last = { id: '', by: '', at: 0 };

  constructor(private readonly commands: Commands, private readonly menus: () => MenuDef[]) {
    document.documentElement.classList.add('has-native-menus');
    for (const ev of ['pointerup', 'keyup', 'focusin', 'focusout'] as const) window.addEventListener(ev, () => { if (this.items.length) this.refresh(); }, { passive: true });
  }

  claim(id: string, by: 'key' | 'menu'): boolean {
    const now = performance.now(), twice = this.last.id === id && this.last.by !== by && now - this.last.at < 100;
    this.last = twice ? { id: '', by: '', at: 0 } : { id, by, at: now };
    return !twice;
  }

  private read(): Read {
    return this.menus().map((d) => ({ label: d.label, items: typeof d.items === 'function' ? d.items() : d.items }));
  }

  private shapeOf(defs: Read): string {
    const row = (i: MenuSpec): unknown => {
      if (i === '-') return 0;
      if (typeof i === 'object') return [t(i.label), i.items.map(row)];
      const cmd = this.commands.get(i);
      return cmd ? [i, t(this.commands.label(cmd)), cmd.keys?.[0] ?? '', cmd.checked ? 1 : 0] : null;
    };
    return JSON.stringify(defs.map((d) => [t(d.label), d.items.map(row)]));
  }

  async build(): Promise<void> {
    if (this.building) { this.again = true; return; }
    this.building = true;
    try {
      const api: Api = await import('@tauri-apps/api/menu');
      const defs = this.read(), shape = this.shapeOf(defs);
      const items: Built[] = [];
      const taken = new Set(SYSTEM_KEYS);
      const sep = (): Promise<unknown> => api.PredefinedMenuItem.new({ item: 'Separator' });
      const make = async (spec: MenuSpec): Promise<unknown> => {
        if (spec === '-') return sep();
        if (typeof spec === 'object') return api.Submenu.new({ text: t(spec.label), items: (await Promise.all(spec.items.map(make))).filter(Boolean) as never[] });
        const cmd = this.commands.get(spec);
        if (!cmd) return null;
        const text = t(this.commands.label(cmd)), key = accelerator(cmd.keys?.[0]);
        const acc = key && !taken.has(sameKeys(key)) ? key : null;
        if (acc) taken.add(sameKeys(acc));
        const on = this.enabled(spec), checked = !!cmd.checked?.();
        const opts = { text, enabled: on, action: () => {
          const edit = TEXT_EDITS.get(spec);
          if (edit && textFieldFocused()) void import('@tauri-apps/api/core').then((m) => m.invoke('text_edit', { action: edit })).catch(() => { document.execCommand(edit === 'select-all' ? 'selectAll' : edit); });
          else if (!anyDialogOpen() && this.claim(spec, 'menu')) this.commands.run(spec);
          this.refresh();
        } };
        let item: Live;
        if (cmd.checked) {
          const copts = { ...opts, checked };
          try { item = await api.CheckMenuItem.new(acc ? { ...copts, accelerator: acc } : copts); } catch { item = await api.CheckMenuItem.new(copts); }
        } else {
          try { item = await api.MenuItem.new(acc ? { ...opts, accelerator: acc } : opts); } catch { item = await api.MenuItem.new(opts); }
        }
        items.push({ id: spec, item, on });
        return item;
      };
      const top: unknown[] = [];
      top.push(await api.Submenu.new({ text: 'FormFabric', items: [
        await api.PredefinedMenuItem.new({ item: { About: { name: 'FormFabric', version: VERSION, copyright: COPYRIGHT, license: 'Apache-2.0' } }, text: t('About FormFabric') }),
        await sep(),
        ...(this.commands.get('app.settings') ? [await make('app.settings')] : []),
        await sep(),
        await api.PredefinedMenuItem.new({ item: 'Services', text: t('Services') }),
        await sep(),
        await api.PredefinedMenuItem.new({ item: 'Hide', text: t('Hide FormFabric') }),
        await api.PredefinedMenuItem.new({ item: 'HideOthers', text: t('Hide Others') }),
        await api.PredefinedMenuItem.new({ item: 'ShowAll', text: t('Show All') }),
        await sep(),
        await api.PredefinedMenuItem.new({ item: 'Quit', text: t('Quit FormFabric') }),
      ] as never[] }));
      let windows: { setAsWindowsMenuForNSApp(): Promise<void> } | null = null;
      let help: { setAsHelpMenuForNSApp(): Promise<void> } | null = null;
      for (const def of defs) {
        const rows = (await Promise.all(def.items.map(make))).filter(Boolean);
        if (def.label === 'Window') rows.push(...(rows.length ? [await sep()] : []),
          await api.PredefinedMenuItem.new({ item: 'Minimize', text: t('Minimize') }),
          await api.PredefinedMenuItem.new({ item: 'Maximize', text: t('Maximize') }),
          await api.PredefinedMenuItem.new({ item: 'Fullscreen', text: t('Full Screen') }),
          await sep(),
          await api.PredefinedMenuItem.new({ item: 'BringAllToFront', text: t('Bring All to Front') }));
        const sub = await api.Submenu.new({ text: t(def.label), items: rows as never[] });
        if (def.label === 'Window') windows = sub;
        if (def.label === 'Help') help = sub;
        top.push(sub);
      }
      const menu = await api.Menu.new({ items: top as never[] });
      await menu.setAsAppMenu();
      if (windows) void windows.setAsWindowsMenuForNSApp().catch(() => undefined);
      if (help) void help.setAsHelpMenuForNSApp().catch(() => undefined);
      this.items = items;
      this.shape = shape;
      if (!document.documentElement.classList.contains('has-native-menus')) document.documentElement.classList.add('has-native-menus');
    } catch (err) {
      this.late = false;
      if (!this.items.length) document.documentElement.classList.remove('has-native-menus');
      throw err;
    } finally {
      this.building = false;
      const more = this.again, late = this.late;
      this.again = false;
      this.late = false;
      if (more) void this.build().catch(() => undefined);
      else if (late) this.refresh();
    }
  }

  private enabled(id: string): boolean {
    if (TEXT_EDITS.has(id) && textFieldFocused()) return true;
    if (id === 'app.none' || anyDialogOpen()) return false;
    const cmd = this.commands.get(id);
    return !!cmd && this.commands.enabled(cmd);
  }

  refresh(): void {
    if (this.building) { this.late = true; return; }
    if (this.stateTimer) return;
    this.stateTimer = window.setTimeout(() => {
      this.stateTimer = 0;
      if (this.building) { this.late = true; return; }
      if (!this.shape) return;
      if (this.shapeOf(this.read()) !== this.shape) { void this.build().catch(() => undefined); return; }
      for (const b of this.items) {
        const cmd = this.commands.get(b.id);
        if (!cmd) continue;
        const on = this.enabled(b.id);
        if (on !== b.on) { b.on = on; void b.item.setEnabled(on).catch(() => undefined); }
        if (cmd.checked && b.item.setChecked) void b.item.setChecked(!!cmd.checked()).catch(() => undefined);
      }
    }, 150);
  }
}
