import { Dialog } from '../../ui/dialogs';
import { el, setHidden, setText } from '../../ui/dom';
import { t } from '../../ui/i18n';
import { icon } from '../../ui/icons';
import { browseSave, canBrowse, recall, remember } from './files';
import type { OutputStore } from './store';
import { bar, numRow, selectRow, textRow, titled, toggleRow } from './widgets';

type Notify = (text: string, kind?: 'info' | 'error') => void;

abstract class JobDialog extends Dialog {
  protected readonly note = el('<p class="cv-note ou-note"></p>');
  protected readonly status = el('<p class="ou-status"></p>');
  protected readonly progress = bar();
  protected readonly pathRow = textRow('Path');
  protected readonly browse = el<HTMLButtonElement>(`<button class="cv-icon-btn" type="button">${icon('folderOpen', 16)}</button>`);
  protected readonly go: HTMLButtonElement;
  protected busy = false;

  constructor(title: string, protected readonly command: string, protected readonly store: OutputStore, protected readonly notify: Notify, private readonly slot: string, private readonly fallback: string, private readonly ext: string, private readonly extName: string, runLabel: string) {
    super(title, 480);
    titled(this.browse, 'Browse');
    setHidden(this.browse, !canBrowse());
    this.browse.addEventListener('click', () => { void this.pick(); });
    this.pathRow.onInput(() => remember(`${this.slot}.path`, this.pathRow.get()));
    this.button('Close', 'ghost', () => this.close());
    this.go = this.button(runLabel, 'primary', () => { void this.run(); });
  }

  protected pathLine(): HTMLElement {
    const line = el('<div class="ou-pathline"></div>');
    line.append(this.pathRow.el, this.browse);
    return line;
  }

  protected tail(): void {
    this.body.append(this.note, this.progress.el, this.status);
    this.pathRow.set(recall<string>(`${this.slot}.path`, this.fallback));
  }

  protected abstract args(): Record<string, unknown>;

  private async pick(): Promise<void> {
    const chosen = await browseSave(this.ext, this.pathRow.get(), this.extName);
    if (chosen) { this.pathRow.set(chosen); remember(`${this.slot}.path`, chosen); }
  }

  protected paint(): void {
    const have = this.store.has(this.command);
    const msg = have ? '' : `${t('Not available in this engine build')}: ${this.command}`;
    setText(this.note, msg);
    setHidden(this.note, msg === '');
    const off = !have || this.busy;
    if (this.go.disabled !== off) this.go.disabled = off;
  }

  async present(): Promise<void> {
    await this.store.loadCaps();
    setText(this.status, '');
    this.progress.show(false);
    this.paint();
    this.open(this.pathRow.input);
  }

  private async run(): Promise<void> {
    if (this.busy) return;
    const path = this.pathRow.get().trim();
    if (!path) { setText(this.status, t('Choose a path first.')); this.pathRow.input.focus(); return; }
    this.busy = true;
    this.progress.set(null);
    this.progress.show(true);
    setText(this.status, t('Working...'));
    this.paint();
    const r = await this.store.attempt<Record<string, unknown>>(this.command, this.args());
    this.busy = false;
    this.progress.show(false);
    this.paint();
    if (!r.ok) { setText(this.status, r.message); this.notify(r.message, 'error'); return; }
    setText(this.status, '');
    this.notify(`${t('Finished')}: ${typeof r.result?.path === 'string' ? r.result.path : path}`, 'info');
    this.close();
  }
}

export class BakeDialog extends JobDialog {
  private readonly maps: Array<[string, ReturnType<typeof toggleRow>]> = [];
  private readonly size = selectRow('Size', [['512', '512'], ['1024', '1024'], ['2048', '2048'], ['4096', '4096']]);
  private readonly fill = numRow('Seam fill', 'px', '1', 0, 256);

  constructor(store: OutputStore, notify: Notify) {
    super('Bake Maps', 'bake.run', store, notify, 'bake', 'bake/garment', 'png', 'Image', 'Bake');
    const saved = recall<{ maps: string[]; size: string; fill?: number }>('bake.opts', { maps: ['normal', 'ao'], size: '2048', fill: 4 });
    this.fill.set(typeof saved.fill === 'number' ? saved.fill : 4);
    this.fill.onInput(() => this.keep());
    for (const [id, text] of [['diffuse', 'Diffuse'], ['metalness', 'Metalness'], ['roughness', 'Roughness'], ['opacity', 'Opacity'], ['normal', 'Normal'], ['ao', 'Ambient occlusion'], ['displacement', 'Displacement']] as const) {
      const row = toggleRow(text);
      row.set(saved.maps.includes(id));
      row.onInput(() => { this.keep(); this.paint(); });
      this.maps.push([id, row]);
      this.body.appendChild(row.el);
    }
    this.size.set(saved.size);
    this.size.onInput(() => this.keep());
    this.body.append(this.size.el, this.fill.el, this.pathLine());
    this.tail();
  }

  private keep(): void {
    remember('bake.opts', { maps: this.chosen(), size: this.size.get(), fill: this.fill.get() });
  }

  private chosen(): string[] {
    return this.maps.filter(([, r]) => r.get()).map(([id]) => id);
  }

  protected args(): Record<string, unknown> {
    return { maps: this.chosen(), size: Number(this.size.get()), fill_px: Math.min(256, Math.max(0, Math.round(this.fill.get()))), path: this.pathRow.get().trim() };
  }

  protected override paint(): void {
    super.paint();
    if (this.chosen().length === 0 && !this.go.disabled) this.go.disabled = true;
  }
}

export class GameReadyDialog extends JobDialog {
  private readonly tris = numRow('Target triangles', '', '100', 100, 5000000);
  private readonly quads = toggleRow('Quads');

  constructor(store: OutputStore, notify: Notify) {
    super('Game-Ready', 'gameready.run', store, notify, 'game', 'GameReady.glb', 'glb', 'Mesh', 'Create');
    const saved = recall<{ tris: number; quads: boolean }>('game.opts', { tris: 10000, quads: false });
    this.tris.set(saved.tris);
    this.quads.set(saved.quads);
    const keep = (): void => remember('game.opts', { tris: this.tris.get(), quads: this.quads.get() });
    this.tris.onInput(keep);
    this.quads.onInput(keep);
    this.body.append(this.tris.el, this.quads.el, this.pathLine());
    this.tail();
  }

  protected args(): Record<string, unknown> {
    return { path: this.pathRow.get().trim(), target_tris: Math.max(100, Math.round(this.tris.get())), quads: this.quads.get() };
  }
}
