import { serviceUrl } from '../../engine/service';
import { Dialog } from '../../ui/dialogs';
import { el, setText } from '../../ui/dom';
import { t } from '../../ui/i18n';
import { nativeDialogs } from '../../shell/file-dialog';

const IMAGE_EXT = ['png', 'jpg', 'jpeg', 'webp', 'tif', 'tiff', 'bmp'];

class PathDialog extends Dialog {
  private readonly input = el<HTMLInputElement>('<input class="cv-input" type="text" spellcheck="false" autocomplete="off">');
  private readonly ok: HTMLButtonElement;
  private resolve: ((v: string | null) => void) | null = null;

  constructor() {
    super('Path', 460);
    this.field('Path', this.input);
    this.button('Cancel', 'ghost', () => this.finish(null));
    this.ok = this.button('OK', 'primary', () => this.finish(this.input.value.trim() || null));
    this.input.addEventListener('keydown', (e) => {
      if (e.key === 'Enter') { e.preventDefault(); this.ok.click(); }
    });
    this.onClose = () => this.finish(null);
  }

  ask(title: string, value: string, okLabel: string): Promise<string | null> {
    this.setTitle(title);
    setText(this.ok, okLabel);
    this.input.value = value;
    this.open(this.input);
    this.input.select();
    return new Promise<string | null>((resolve) => { this.resolve = resolve; });
  }

  private finish(v: string | null): void {
    const done = this.resolve;
    this.resolve = null;
    if (this.isOpen) this.close();
    done?.(v);
  }
}

let pathDialog: PathDialog | null = null;

export async function pickFabricFile(kind: 'open' | 'save', current: string): Promise<string | null> {
  if (!nativeDialogs()) {
    pathDialog ??= new PathDialog();
    return kind === 'open' ? pathDialog.ask(t('Import fabric'), current, t('Import')) : pathDialog.ask(t('Export fabric'), current, t('Export'));
  }
  const dialog = await import('@tauri-apps/plugin-dialog');
  const filters = [{ name: t('Fabric'), extensions: ['ffab'] }];
  if (kind === 'open') {
    const chosen = await dialog.open({ multiple: false, directory: false, filters });
    return typeof chosen === 'string' ? chosen : null;
  }
  const chosen = await dialog.save({ filters, defaultPath: current });
  if (!chosen) return null;
  return /\.ffab$/i.test(chosen) ? chosen : `${chosen}.ffab`;
}

export const canPickImage = (): boolean => nativeDialogs();

export async function pickImage(): Promise<string | null> {
  if (!nativeDialogs()) return null;
  const dialog = await import('@tauri-apps/plugin-dialog');
  const chosen = await dialog.open({ multiple: false, directory: false, filters: [{ name: t('Images'), extensions: IMAGE_EXT }] });
  return typeof chosen === 'string' ? chosen : null;
}

const previews = new Map<string, Promise<string | null>>();

export function previewUrl(path: string): Promise<string | null> {
  if (!path || path.startsWith('maps/')) return Promise.resolve(null);
  const known = previews.get(path);
  if (known) return known;
  const loading = fetch(serviceUrl(`/api/files/image?path=${encodeURIComponent(path)}`), { cache: 'no-store' })
    .then((r) => (r.ok ? r.blob() : null))
    .then((blob) => (blob ? URL.createObjectURL(blob) : null))
    .catch(() => null);
  previews.set(path, loading);
  void loading.then((url) => { if (!url && previews.get(path) === loading) previews.delete(path); });
  return loading;
}
