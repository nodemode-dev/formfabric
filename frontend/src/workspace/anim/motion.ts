import { nativeDialogs } from '../../shell/file-dialog';
import { el, q, setDisabled, setHidden } from '../../ui/dom';
import { t } from '../../ui/i18n';
import { icon } from '../../ui/icons';
import type { AnimStore } from './store';
import { bodyRows } from './types';
import { checkRow, label, numRow, repaint, selRow, textRow, titled, type Field, type SelRow } from './widgets';

const EXT = ['bvh', 'gltf', 'glb', 'fbx'];

const HTML =
  '<div class="ff-an-pop-sec" data-k="import"><h4 data-k="h-import"></h4><div class="ff-an-pathrow"></div><div data-k="body"></div><div data-k="extras"></div><h5 data-k="h-retarget"></h5><div data-k="retarget"></div><div class="ff-an-actions"><button class="ff-btn" type="button" data-k="go"></button></div></div>' +
  '<div class="ff-an-pop-sec" data-k="morph"><h4 data-k="h-morph"></h4><div data-k="morph-rows"></div><div class="ff-an-actions"><button class="ff-btn" type="button" data-k="morph-go"></button></div></div>';

interface Extra { where: 'import' | 'retarget'; key: string; read(): unknown; field: Field<unknown> | SelRow }

export class MotionPanel {
  readonly root: HTMLElement;
  private readonly path: Field<string>;
  private readonly body: SelRow;
  private readonly from: SelRow;
  private readonly to: SelRow;
  private readonly frames: Field<number>;
  private readonly go: HTMLButtonElement;
  private readonly morphGo: HTMLButtonElement;
  private readonly extras: Extra[] = [];
  private built = false;
  private busy = false;

  constructor(private readonly store: AnimStore) {
    this.root = el('<div class="ff-an-pop" hidden></div>');
    this.root.innerHTML = HTML;
    label(q(this.root, '[data-k="h-import"]'), 'Import Motion');
    label(q(this.root, '[data-k="h-morph"]'), 'Morph');
    label(q(this.root, '[data-k="h-retarget"]'), 'Retarget');

    this.path = textRow('Motion file', true);
    const pathrow = q(this.root, '.ff-an-pathrow');
    pathrow.appendChild(this.path.el);
    if (nativeDialogs()) {
      const browse = el<HTMLButtonElement>(`<button class="cv-icon-btn" type="button">${icon('folderOpen', 16)}</button>`);
      titled(browse, 'Browse');
      browse.addEventListener('click', () => { void this.browse(); browse.blur(); });
      pathrow.appendChild(browse);
    }
    this.path.input.addEventListener('keydown', (e) => { if (e.key === 'Enter') { e.preventDefault(); this.go.click(); } });

    this.body = selRow('Body', true, [], false);
    q(this.root, '[data-k="body"]').appendChild(this.body.el);

    this.go = q<HTMLButtonElement>(this.root, '[data-k="go"]');
    this.go.innerHTML = `${icon('fileIn', 14)}<span></span>`;
    label(this.go.children[1] as HTMLElement, 'Import');
    this.go.addEventListener('click', () => { void this.importMotion(); this.go.blur(); });

    this.from = selRow('From body', true, [], false);
    this.to = selRow('To body', true, [], false);
    this.frames = numRow('Frames', true, 2, 2000, 1, 30);
    const rows = q(this.root, '[data-k="morph-rows"]');
    rows.append(this.from.el, this.to.el, this.frames.el);
    this.morphGo = q<HTMLButtonElement>(this.root, '[data-k="morph-go"]');
    this.morphGo.innerHTML = `${icon('body', 14)}<span></span>`;
    label(this.morphGo.children[1] as HTMLElement, 'Create Morph');
    this.morphGo.addEventListener('click', () => { void this.morph(); this.morphGo.blur(); });
    repaint(() => { const s = t('Motion file'); if (this.path.input.placeholder !== s) this.path.input.placeholder = s; });
  }

  get open(): boolean {
    return !this.root.hidden;
  }

  setOpen(on: boolean): void {
    if (on === this.open) return;
    if (on) { this.build(); void this.loadBodies(); }
    setHidden(this.root, !on);
  }

  sync(): void {
    this.build();
    setDisabled(this.go, !this.store.has('motion.import') || this.busy);
    setDisabled(this.morphGo, !this.store.has('morph.set') || this.busy);
    setHidden(q(this.root, '[data-k="morph"]'), false);
  }

  private build(): void {
    if (this.built || !this.store.loaded) return;
    this.built = true;
    const host = q(this.root, '[data-k="extras"]');
    const retHost = q(this.root, '[data-k="retarget"]');
    const add = (where: 'import' | 'retarget', key: string, prop: Record<string, unknown>): void => {
      const type = Array.isArray(prop.type) ? String(prop.type[0]) : String(prop.type ?? 'string');
      const en = Array.isArray(prop.enum) ? (prop.enum as Array<string | number>) : null;
      const title = key.replace(/_/g, ' ');
      let field: Field<unknown> | SelRow;
      let read: () => unknown;
      let touched = false;
      if (en) {
        const s = selRow(title, false, en.map((v) => [String(v), String(v)] as const), false);
        field = s;
        s.select.addEventListener('change', () => { touched = true; });
        read = () => { const v = s.get(); return touched ? en.find((x) => String(x) === v) ?? v : undefined; };
      } else if (type === 'boolean') {
        const c = checkRow(title, false, prop.default === true);
        field = c as Field<unknown>;
        c.input.addEventListener('change', () => { touched = true; });
        read = () => (touched ? c.get() : undefined);
      } else if (type === 'number' || type === 'integer') {
        const n = numRow(title, false, -1e6, 1e6, type === 'integer' ? 1 : 0.01, typeof prop.default === 'number' ? prop.default : 1);
        field = n as Field<unknown>;
        n.input.addEventListener('input', () => { touched = true; });
        read = () => (touched ? n.get() : undefined);
      } else {
        const s = textRow(title, false);
        field = s as Field<unknown>;
        read = () => s.get().trim() || undefined;
      }
      this.extras.push({ where, key, read, field });
      (where === 'retarget' ? retHost : host).appendChild(field.el);
    };
    const skip = ['path', 'body', 'name', 'retarget', 'start', 'lead_in'];
    for (const [k, p] of Object.entries(this.store.props('motion.import'))) if (!skip.includes(k)) add('import', k, p);
    const ret = this.store.props('motion.import').retarget ?? this.store.props('motion.set').retarget;
    const sub = ret && typeof ret.properties === 'object' && ret.properties ? (ret.properties as Record<string, Record<string, unknown>>) : null;
    const known: Record<string, Record<string, unknown>> = sub && Object.keys(sub).length ? sub : { root: { type: 'string', enum: ['keep', 'inplace', 'off'] }, scale: { type: 'number', default: 1 }, loop: { type: 'boolean' }, align: { type: 'boolean' } };
    if (ret) {
      retHost.hidden = false;
      for (const [k, p] of Object.entries(known)) add('retarget', k, p);
    }
  }

  private async browse(): Promise<void> {
    const dialog = await import('@tauri-apps/plugin-dialog');
    const chosen = await dialog.open({ multiple: false, directory: false, filters: [{ name: t('Motion file'), extensions: EXT }] });
    if (typeof chosen === 'string') this.path.set(chosen);
  }

  private async loadBodies(): Promise<void> {
    const r = await this.store.read<unknown>('body.list', {});
    const rows = r === undefined ? [] : bodyRows(r);
    const items = rows.map((b) => [String(b.id), b.name] as const);
    for (const s of [this.body, this.from, this.to]) s.options(items, false);
    if (rows.length > 1 && this.to.select.value === this.from.select.value) this.to.select.value = String(rows[1].id);
    for (const s of [this.body, this.from, this.to]) s.disable(rows.length === 0);
  }

  private retarget(): unknown {
    const out: Record<string, unknown> = {};
    for (const x of this.extras) if (x.where === 'retarget') { const v = x.read(); if (v !== undefined) out[x.key] = v; }
    return Object.keys(out).length ? out : undefined;
  }

  private async importMotion(): Promise<void> {
    const path = this.path.get().trim();
    if (!path) { this.path.input.focus(); return; }
    if (this.busy) return;
    this.busy = true;
    this.sync();
    const args: Record<string, unknown> = { path };
    const body = Number(this.body.get());
    if (this.body.get() !== '' && Number.isFinite(body)) args.body = body;
    for (const x of this.extras) if (x.where === 'import') { const v = x.read(); if (v !== undefined) args[x.key] = v; }
    const retarget = this.retarget();
    if (retarget !== undefined) args.retarget = retarget;
    await this.store.run<Record<string, unknown>>('motion.import', args);
    this.busy = false;
    this.sync();
    await this.store.refresh();
  }

  private async morph(): Promise<void> {
    const from = Number(this.from.get());
    const to = Number(this.to.get());
    if (!Number.isFinite(from) || !Number.isFinite(to) || this.from.get() === '' || this.to.get() === '') return;
    this.busy = true;
    this.sync();
    await this.store.run('morph.set', { from_body: from, to_body: to, frames: Math.round(this.frames.get()) });
    this.busy = false;
    this.sync();
    await this.store.refresh();
  }
}
