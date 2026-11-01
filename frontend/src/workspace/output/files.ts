import { nativeDialogs } from '../../shell/file-dialog';
import { t } from '../../ui/i18n';

export const canBrowse = (): boolean => nativeDialogs();

export async function browseSave(ext: string, current: string, name: string): Promise<string | null> {
  if (!nativeDialogs()) return null;
  const dialog = await import('@tauri-apps/plugin-dialog');
  const chosen = await dialog.save({ filters: [{ name: t(name), extensions: [ext] }], defaultPath: current || undefined });
  if (!chosen) return null;
  return new RegExp(`\\.${ext}$`, 'i').test(chosen) ? chosen : `${chosen}.${ext}`;
}

export async function browseFolder(current: string): Promise<string | null> {
  if (!nativeDialogs()) return null;
  const dialog = await import('@tauri-apps/plugin-dialog');
  const chosen = await dialog.open({ multiple: false, directory: true, defaultPath: current || undefined });
  return typeof chosen === 'string' ? chosen : null;
}

const KEY = 'formfabric.output.';

export function recall<T>(name: string, fallback: T): T {
  try {
    const raw = localStorage.getItem(KEY + name);
    return raw === null ? fallback : (JSON.parse(raw) as T);
  } catch { return fallback; }
}

export function remember(name: string, value: unknown): void {
  try { localStorage.setItem(KEY + name, JSON.stringify(value)); } catch { /* this session only */ }
}

export function swapExt(path: string, from: string, to: string): string {
  const re = new RegExp(`\\.${from}$`, 'i');
  if (re.test(path)) return path.replace(re, `.${to}`);
  return path;
}
