const FORMATS = {
  ffbrc: { filters: [{ name: 'FormFabric project', extensions: ['ffbrc'] }], fallback: 'Untitled.ffbrc', ext: /\.ffbrc$/i, suffix: '.ffbrc' },
  dxf: { filters: [{ name: 'DXF', extensions: ['dxf'] }], fallback: 'Pattern.dxf', ext: /\.dxf$/i, suffix: '.dxf' },
  obj: { filters: [{ name: 'OBJ', extensions: ['obj'] }], fallback: 'Trim.obj', ext: /\.obj$/i, suffix: '.obj' },
  image: { filters: [{ name: 'Picture', extensions: ['png', 'jpg', 'jpeg', 'webp'] }], fallback: 'Background.png', ext: /\.(png|jpe?g|webp)$/i, suffix: '.png' },
  json: { filters: [{ name: 'JSON', extensions: ['json'] }], fallback: 'formfabric-settings.json', ext: /\.json$/i, suffix: '.json' },
};

export const nativeDialogs = (): boolean => '__TAURI_INTERNALS__' in window;

export const dxfCommand = (id: string): 'open' | 'save' | null => (id === 'exchange.import_dxf' ? 'open' : id === 'exchange.export_dxf' ? 'save' : null);

export async function pickPath(kind: 'open' | 'save', current: string | null, format: keyof typeof FORMATS = 'ffbrc'): Promise<string | null> {
  const dialog = await import('@tauri-apps/plugin-dialog');
  const f = FORMATS[format];
  if (kind === 'open') {
    const chosen = await dialog.open({ multiple: false, directory: false, filters: f.filters });
    return typeof chosen === 'string' ? chosen : null;
  }
  const chosen = await dialog.save({ filters: f.filters, defaultPath: current ?? f.fallback });
  if (!chosen) return null;
  return f.ext.test(chosen) ? chosen : `${chosen}${f.suffix}`;
}
