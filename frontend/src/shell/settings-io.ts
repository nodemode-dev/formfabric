import type { Commands } from '../ui/commands';
import { loadPrefs, sanitizePrefs, savedLayout, updatePrefs, type Prefs } from '../ui/prefs';
import { nativeDialogs, pickPath } from './file-dialog';

export const BUNDLE_KIND = 'formfabric.settings';
const KEYS_KEY = 'formfabric.keys';

export interface Bundle {
  kind: typeof BUNDLE_KIND;
  version: 1;
  prefs: Record<string, unknown>;
  shortcuts: Record<string, string[]>;
  layouts: Prefs['layouts'];
}

export function buildBundle(commands: Commands): Bundle {
  const p = loadPrefs();
  const { dialogValues: _dv, shortcuts: _sc, shortcutSets: _ss, layouts, ...simple } = p;
  void _dv; void _sc; void _ss;
  return { kind: BUNDLE_KIND, version: 1, prefs: simple, shortcuts: commands.overrides(), layouts };
}

export interface ImportResult { ok: boolean; message: string; prefs: number; shortcuts: number; layouts: number }

export function applyBundle(commands: Commands, raw: unknown): ImportResult {
  const bad = (message: string): ImportResult => ({ ok: false, message, prefs: 0, shortcuts: 0, layouts: 0 });
  if (!raw || typeof raw !== 'object') return bad('Not a settings file');
  const b = raw as Partial<Bundle>;
  if (b.kind !== BUNDLE_KIND || b.version !== 1) return bad('Not a FormFabric settings file');
  const patch = sanitizePrefs(b.prefs && typeof b.prefs === 'object' ? (b.prefs as Record<string, unknown>) : {});
  const shortcuts: Record<string, string[]> = {};
  if (b.shortcuts && typeof b.shortcuts === 'object') {
    for (const [id, keys] of Object.entries(b.shortcuts)) if (Array.isArray(keys) && commands.get(id)) shortcuts[id] = keys.filter((x): x is string => typeof x === 'string');
  }
  const layouts: Prefs['layouts'] = {};
  if (b.layouts && typeof b.layouts === 'object') {
    for (const [name, v] of Object.entries(b.layouts)) { const s = savedLayout(v); if (s && name.trim()) layouts[name.trim().slice(0, 60)] = s; }
  }
  commands.resetKeys();
  for (const [id, keys] of Object.entries(shortcuts)) commands.setKeys(id, keys);
  try { localStorage.setItem(KEYS_KEY, JSON.stringify(commands.overrides())); } catch { /* this session only */ }
  updatePrefs({ ...patch, shortcuts: commands.overrides(), layouts: { ...loadPrefs().layouts, ...layouts } });
  return { ok: true, message: '', prefs: Object.keys(patch).length, shortcuts: Object.keys(shortcuts).length, layouts: Object.keys(layouts).length };
}

function download(text: string, name: string): void {
  const a = document.createElement('a');
  a.href = URL.createObjectURL(new Blob([text], { type: 'application/json' }));
  a.download = name;
  document.body.appendChild(a);
  a.click();
  a.remove();
  window.setTimeout(() => URL.revokeObjectURL(a.href), 4000);
}

async function nativeFs(): Promise<{ writeTextFile(p: string, t: string): Promise<void>; readTextFile(p: string): Promise<string> } | null> {
  const name = '@tauri-apps/plugin-fs';
  try { return (await import(/* @vite-ignore */ name)) as never; } catch { return null; }
}

export async function exportSettings(commands: Commands): Promise<string | null> {
  const text = JSON.stringify(buildBundle(commands), null, 2);
  const fs = nativeDialogs() ? await nativeFs() : null;
  if (fs) {
    const path = await pickPath('save', 'formfabric-settings.json', 'json');
    if (!path) return null;
    await fs.writeTextFile(path, text);
    return path;
  }
  download(text, 'formfabric-settings.json');
  return 'formfabric-settings.json';
}

function pickFile(): Promise<string | null> {
  return new Promise((resolve) => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = 'application/json,.json';
    input.addEventListener('change', () => {
      const f = input.files?.[0];
      if (!f) { resolve(null); return; }
      void f.text().then(resolve, () => resolve(null));
    });
    input.addEventListener('cancel', () => resolve(null));
    input.click();
  });
}

export async function importSettings(commands: Commands): Promise<ImportResult | null> {
  let text: string | null = null;
  const fs = nativeDialogs() ? await nativeFs() : null;
  if (fs) {
    const path = await pickPath('open', null, 'json');
    if (!path) return null;
    text = await fs.readTextFile(path);
  }
  text ??= await pickFile();
  if (text === null) return null;
  try { return applyBundle(commands, JSON.parse(text)); } catch { return { ok: false, message: 'Not valid JSON', prefs: 0, shortcuts: 0, layouts: 0 }; }
}
