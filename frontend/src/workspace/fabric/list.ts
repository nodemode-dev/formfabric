import { Dialog } from '../../ui/dialogs';
import { el, q, setClass, setDisabled, setHidden, setStyleVar, setText } from '../../ui/dom';
import { enhanceSelect } from '../../ui/dropdown';
import { icon } from '../../ui/icons';
import { t } from '../../ui/i18n';
import { emptyLabel, engineDown } from '../offline';
import type { Ctx } from './ctx';
import { pickFabricFile } from './files';
import { label, patchKeyed, repaint, titled } from './widgets';
import type { FabricInfo } from './types';

const HTML =
  '<div class="ff-fb-tools"></div>' +
  '<p class="ff-fb-note" data-k="note"></p>' +
  '<div class="ff-fb-list" data-k="list" role="listbox"><p class="ff-fb-empty" data-k="empty" hidden></p></div>' +
  '<div class="ff-fb-uses"><span class="ff-fb-lbl" data-k="usedlbl"></span><div class="ff-fb-chips" data-k="chips"></div></div>' +
  '<div class="ff-fb-actions"></div>';

const css = (c: number[]): string => `rgb(${Math.round(c[0] * 255)} ${Math.round(c[1] * 255)} ${Math.round(c[2] * 255)})`;

class RemoveDialog extends Dialog {
  private readonly name = el<HTMLElement>('<p class="ff-fb-dname"></p>');
  private readonly pick = el<HTMLSelectElement>('<select class="cv-input"></select>');
  private readonly row: HTMLElement;
  private readonly ok: HTMLButtonElement;
  private resolve: ((v: { reassign?: number } | null) => void) | null = null;

  constructor() {
    super('Delete fabric', 400);
    this.scrim.classList.add('ff-fb-dlg');
    this.body.appendChild(this.name);
    this.row = this.field('Reassign its panels to', this.pick);
    enhanceSelect(this.pick);
    this.button('Cancel', 'ghost', () => this.finish(null));
    this.ok = this.button('Delete', 'danger', () => this.finish(this.row.hidden ? {} : { reassign: Number(this.pick.value) }));
    this.onClose = () => this.finish(null);
  }

  ask(f: FabricInfo, others: FabricInfo[]): Promise<{ reassign?: number } | null> {
    setText(this.name, f.name);
    this.pick.replaceChildren(...others.map((o) => { const op = document.createElement('option'); op.value = String(o.id); op.textContent = o.name; return op; }));
    this.pick.value = String((others.find((o) => o.is_default) ?? others[0]).id);
    setHidden(this.row, f.panels.length === 0);
    this.open(this.ok);
    return new Promise((resolve) => { this.resolve = resolve; });
  }

  private finish(v: { reassign?: number } | null): void {
    const done = this.resolve;
    this.resolve = null;
    if (this.isOpen) this.close();
    done?.(v);
  }
}

export class FabricsSection {
  private readonly list: HTMLElement;
  private readonly chips: HTMLElement;
  private readonly note: HTMLElement;
  private readonly empty: HTMLElement;
  private readonly select: HTMLSelectElement;
  private readonly btn: Record<string, HTMLButtonElement> = {};
  private readonly count: HTMLElement;
  private filled = 0;
  private removeDialog: RemoveDialog | null = null;

  constructor(host: HTMLElement, private readonly ctx: Ctx) {
    host.innerHTML = HTML;
    this.list = q(host, '[data-k="list"]');
    this.chips = q(host, '[data-k="chips"]');
    this.note = q(host, '[data-k="note"]');
    this.empty = q(host, '[data-k="empty"]');
    emptyLabel(this.empty, 'No fabrics yet. Add one with New from preset.');
    label(q(host, '[data-k="usedlbl"]'), 'Used by');

    const tools = q(host, '.ff-fb-tools');
    this.select = el<HTMLSelectElement>('<select class="cv-input ff-fb-preset"></select>');
    const first = document.createElement('option');
    first.value = '';
    label(first, 'New from preset...');
    this.select.appendChild(first);
    tools.appendChild(this.select);
    this.select.setAttribute('aria-label', t('New from preset...'));
    enhanceSelect(this.select, { onHover: (v) => this.showNote(v) });
    this.select.addEventListener('change', () => { const id = this.select.value; this.select.value = ''; if (id) void this.create(id); });

    const add = (id: string, ico: string, key: string, run: () => void, parent: HTMLElement = tools): void => {
      const b = el<HTMLButtonElement>(`<button class="cv-icon-btn" type="button">${icon(ico, 16)}</button>`);
      titled(b, key);
      b.addEventListener('click', (e) => { run(); if (e.detail > 0) b.blur(); });
      parent.appendChild(b);
      this.btn[id] = b;
    };
    add('dup', 'duplicate', 'Duplicate', () => void this.duplicate());
    add('def', 'makeDefault', 'Set default', () => void this.makeDefault());
    add('del', 'delete', 'Delete', () => void this.remove());
    add('imp', 'fileIn', 'Import fabric', () => void this.importFile());
    add('exp', 'fileOut', 'Export fabric', () => void this.exportFile());
    add('lab', 'flask', 'Fabric Lab', () => ctx.openLab());

    const actions = q(host, '.ff-fb-actions');
    const assign = el<HTMLButtonElement>(`<button class="ff-btn ff-fb-assign" type="button">${icon('assignPanels', 14)}<span></span><span class="ff-fb-n"></span></button>`);
    label(assign.children[1] as HTMLElement, 'Assign to selected panels');
    this.count = q(assign, '.ff-fb-n');
    assign.addEventListener('click', (e) => { void this.assign(false); if (e.detail > 0) assign.blur(); });
    actions.appendChild(assign);
    this.btn.assign = assign;
    add('free', 'unassign', 'Follow default fabric', () => void this.assign(true), actions);

    repaint(() => this.fillPresets(true));
  }

  private fillPresets(relabel: boolean): void {
    const presets = this.ctx.store.presets?.presets;
    if (!presets) return;
    const options = Array.from(this.select.options).slice(1);
    if (options.length !== presets.length) {
      for (const o of options) o.remove();
      for (const p of presets) {
        const o = document.createElement('option');
        o.value = p.id;
        this.select.appendChild(o);
      }
    } else if (!relabel) return;
    Array.from(this.select.options).slice(1).forEach((o, i) => {
      const p = presets[i];
      const text = p.provisional ? `${p.name} (${t('provisional')})` : p.name;
      if (o.textContent !== text) o.textContent = text;
    });
    this.filled = presets.length;
  }

  private showNote(id: string | null): void {
    const p = id ? this.ctx.store.presets?.presets.find((x) => x.id === id) : undefined;
    if (!p) { setText(this.note, ''); return; }
    setText(this.note, p.provisional ? `${p.description} ${t('Provisional preset: values are derived or estimated. Measure your cloth in Fabric Lab.')}` : p.description);
  }

  private async create(preset: string): Promise<void> {
    const r = await this.ctx.store.run<{ fabric?: FabricInfo }>('fabric.create', { preset });
    if (r?.fabric) this.ctx.select(r.fabric.id);
  }

  private async duplicate(): Promise<void> {
    const f = this.ctx.selected();
    if (!f) return;
    const r = await this.ctx.store.run<{ fabric?: FabricInfo }>('fabric.duplicate', { id: f.id });
    if (r?.fabric) this.ctx.select(r.fabric.id);
  }

  private async makeDefault(): Promise<void> {
    const f = this.ctx.selected();
    if (f) await this.ctx.store.run('fabric.set_default', { id: f.id });
  }

  private async remove(): Promise<void> {
    const f = this.ctx.selected();
    const fabrics = this.ctx.store.list.fabrics;
    if (!f || fabrics.length < 2) return;
    this.removeDialog ??= new RemoveDialog();
    const choice = await this.removeDialog.ask(f, fabrics.filter((o) => o.id !== f.id));
    if (!choice) return;
    await this.ctx.store.run('fabric.delete', { id: f.id, ...(choice.reassign ? { reassign: choice.reassign } : {}) });
    this.ctx.select(null);
  }

  private async importFile(): Promise<void> {
    const path = await pickFabricFile('open', '');
    if (!path) return;
    const r = await this.ctx.store.run<{ fabric?: FabricInfo }>('fabric.import', { path });
    if (r?.fabric) this.ctx.select(r.fabric.id);
  }

  private async exportFile(): Promise<void> {
    const f = this.ctx.selected();
    if (!f) return;
    const path = await pickFabricFile('save', `${f.name}.ffab`);
    if (!path) return;
    const r = await this.ctx.store.run('fabric.export', { fabric: f.id, path });
    if (r !== undefined) this.ctx.info(`${t('Exported')}: ${path}`);
  }

  private async assign(clear: boolean): Promise<void> {
    const f = this.ctx.selected();
    const panels = this.ctx.sel.list().filter((id) => this.ctx.doc.panel(id));
    if (!f || !panels.length) return;
    await this.ctx.store.run('fabric.assign', clear ? { panels, clear: true } : { panels, fabric: f.id });
  }

  private panelName(id: number): string {
    return this.ctx.doc.panel(id)?.name ?? `#${id}`;
  }

  update(): void {
    const { store } = this.ctx;
    if (store.presets && store.presets.presets.length !== this.filled) this.fillPresets(false);
    const f = this.ctx.selected();
    const all = store.list.fabrics;
    patchKeyed(this.list, all, {
      key: (x) => String(x.id),
      create: () => {
        const n = el('<div class="ff-fb-item" role="option" tabindex="0"><i class="ff-fb-sw"></i><span class="ff-fb-name"></span><span class="ff-fb-tag"></span><span class="ff-fb-cnt"></span></div>');
        const pick = (): void => { const id = Number(n.dataset.key); this.ctx.select(id); };
        n.addEventListener('click', pick);
        n.addEventListener('keydown', (e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); pick(); } });
        return n;
      },
      update: (n, x) => {
        setStyleVar(n.children[0] as HTMLElement, '--sw', css(x.color));
        setText(n.children[1], x.name);
        setText(n.children[2], x.is_default ? t('Default') : '');
        setText(n.children[3], x.panels.length ? String(x.panels.length) : '');
        const title = x.panels.map((id) => this.panelName(id)).join(', ');
        if (n.title !== title) n.title = title;
        const on = x.id === f?.id;
        setClass(n, 'is-sel', on);
        n.setAttribute('aria-selected', String(on));
      },
    });
    setHidden(this.empty, all.length > 0 || (!store.loaded && !engineDown()));
    const used = f ? f.panels : [];
    patchKeyed(this.chips, used, {
      key: (id) => String(id),
      create: () => el('<span class="ff-fb-chip"></span>'),
      update: (n, id) => { setText(n, this.panelName(id)); setClass(n, 'is-sel', this.ctx.sel.panels.has(id)); },
    });
    this.touch();
    const one = all.length < 2;
    setDisabled(this.btn.dup, !f);
    setDisabled(this.btn.exp, !f);
    setDisabled(this.btn.def, !f || f.is_default);
    setDisabled(this.btn.del, !f || one);
    setDisabled(this.btn.lab, !f);
  }

  touch(): void {
    const f = this.ctx.selected();
    const n = this.ctx.sel.list().filter((id) => this.ctx.doc.panel(id)).length;
    setText(this.count, n ? `(${n})` : '');
    setDisabled(this.btn.assign, !f || n === 0);
    setDisabled(this.btn.free, !f || n === 0);
    for (const c of Array.from(this.chips.children)) setClass(c, 'is-sel', this.ctx.sel.panels.has(Number((c as HTMLElement).dataset.key)));
  }
}
