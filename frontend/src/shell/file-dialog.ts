const FILTERS = [{ name: 'FormFabric project', extensions: ['ffbrc'] }];

export const nativeDialogs = (): boolean => '__TAURI_INTERNALS__' in window;

export async function pickPath(kind: 'open' | 'save', current: string | null): Promise<string | null> {
  const dialog = await import('@tauri-apps/plugin-dialog');
  if (kind === 'open') {
    const chosen = await dialog.open({ multiple: false, directory: false, filters: FILTERS });
    return typeof chosen === 'string' ? chosen : null;
  }
  const chosen = await dialog.save({ filters: FILTERS, defaultPath: current ?? 'Untitled.ffbrc' });
  if (!chosen) return null;
  return /\.ffbrc$/i.test(chosen) ? chosen : `${chosen}.ffbrc`;
}
