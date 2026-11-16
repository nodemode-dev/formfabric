import { promptName } from '../../ui/dialogs';
import { el, q, setClass, setDisabled, setStyleVar, setText } from '../../ui/dom';
import { icon } from '../../ui/icons';
import { t } from '../../ui/i18n';
import type { Ctx } from './ctx';
import { fromHex, toHex, type FabricInfo, type Rgba } from './types';
import { label, patchKeyed, titled } from './widgets';

const css = (c: Rgba): string => `rgb(${Math.round(c[0] * 255)} ${Math.round(c[1] * 255)} ${Math.round(c[2] * 255)})`;

const HTML =
  '<div class="ff-fb-tools" data-k="tools"></div>' +
  '<div class="ff-fb-list ff-fb-list-s" data-k="list" role="listbox"></div>' +
  '<h5 class="ff-fb-sub" data-k="colorsl"></h5>' +
  '<div class="ff-fb-cwcolors" data-k="colors"></div>';

export class ColorwaySection {
  private cw: number | null = null;
  private seeded = false;
  private readonly list: HTMLElement;
  private readonly colors: HTMLElement;
  private readonly btn: Record<string, HTMLButtonElement> = {};

  constructor(host: HTMLElement, private readonly ctx: Ctx) {
    host.innerHTML = HTML;
    this.list = q(host, '[data-k="list"]');
    this.colors = q(host, '[data-k="colors"]');
    label(q(host, '[data-k="colorsl"]'), 'Color of each fabric');
    const tools = q(host, '[data-k="tools"]');
    const add = (id: string, ico: string, key: string, run: () => void): void => {
      const b = el<HTMLButtonElement>(`<button class="cv-icon-btn" type="button">${icon(ico, 16)}</button>`);
      titled(b, key);
      b.addEventListener('click', (e) => { run(); if (e.detail > 0) b.blur(); });
      tools.appendChild(b);
      this.btn[id] = b;
    };
    add('new', 'plus', 'New colorway', () => void this.create());
    add('ren', 'pen', 'Rename', () => void this.rename());
    add('del', 'delete', 'Delete', () => void this.remove());
    add('act', 'check', 'Activate', () => void this.activate());
  }

  private async create(): Promise<void> {
    const n = this.ctx.store.list.colorways.length + 1;
    const name = await promptName(t('New colorway'), `${t('Colorway')} ${n}`, t('Create'));
    if (!name) return;
    const from = this.ctx.store.list.active_colorway;
    const r = await this.ctx.store.run<{ colorway?: { id: number } }>('colorway.create', { name, ...(from !== null ? { from } : {}) });
    if (!r?.colorway) return;
    await this.ctx.store.refresh();
    this.cw = r.colorway.id;
    this.update();
  }

  private async rename(): Promise<void> {
    const c = this.ctx.store.list.colorways.find((x) => x.id === this.cw);
    if (!c) return;
    const name = await promptName(t('Rename'), c.name, t('Rename'));
    if (name && name !== c.name) await this.ctx.store.run('colorway.update', { id: c.id, name });
  }

  private async remove(): Promise<void> {
    if (this.cw === null) return;
    const id = this.cw;
    this.cw = null;
    await this.ctx.store.run('colorway.delete', { id });
  }

  private async activate(): Promise<void> {
    await this.ctx.store.run('colorway.activate', this.cw === null ? {} : { id: this.cw });
  }

  private async setColor(f: FabricInfo, hex: string): Promise<void> {
    const alpha = f.color[3];
    const rgba = fromHex(hex, alpha);
    if (this.cw === null) await this.ctx.store.run('fabric.update', { id: f.id, look: { front: { color: rgba } } });
    else await this.ctx.store.run('colorway.update', { id: this.cw, fabric_colors: { [String(f.id)]: rgba } });
  }

  private colorOf(f: FabricInfo): Rgba {
    if (this.cw === null) return f.look.front.color;
    const c = this.ctx.store.list.colorways.find((x) => x.id === this.cw);
    return c?.fabric_colors[String(f.id)] ?? f.look.front.color;
  }

  update(): void {
    const { list } = this.ctx.store;
    if (!this.seeded && this.ctx.store.loaded) { this.seeded = true; this.cw = list.active_colorway; }
    if (this.cw !== null && !list.colorways.some((c) => c.id === this.cw)) this.cw = null;
    const rows: Array<{ id: number | null; name: string }> = [{ id: null, name: '' }, ...list.colorways.map((c) => ({ id: c.id as number | null, name: c.name }))];
    patchKeyed(this.list, rows, {
      key: (r) => (r.id === null ? 'none' : String(r.id)),
      create: (r) => {
        const n = el('<div class="ff-fb-item" role="option" tabindex="0"><i class="ff-fb-dot"></i><span class="ff-fb-name"></span><span class="ff-fb-tag"></span></div>');
        const pick = (): void => { this.cw = r.id; this.update(); };
        n.addEventListener('click', pick);
        n.addEventListener('dblclick', () => { pick(); void this.activate(); });
        n.addEventListener('keydown', (e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); pick(); } });
        return n;
      },
      update: (n, r) => {
        setText(n.children[1], r.id === null ? t('Original colors') : r.name);
        setText(n.children[2], list.active_colorway === r.id ? t('Active') : '');
        setClass(n, 'is-sel', this.cw === r.id);
        setClass(n, 'is-live', list.active_colorway === r.id);
        n.setAttribute('aria-selected', String(this.cw === r.id));
      },
    });
    patchKeyed(this.colors, list.fabrics, {
      key: (f) => String(f.id),
      create: (f) => {
        const n = el('<label class="ff-fb-cwrow"><i class="ff-fb-sw"></i><span class="ff-fb-name"></span><input class="cv-input cv-color" type="color"><button class="cv-icon-btn" type="button" data-k="reset"></button></label>');
        const input = q<HTMLInputElement>(n, 'input');
        const reset = q<HTMLButtonElement>(n, '[data-k="reset"]');
        reset.innerHTML = icon('refresh', 14);
        titled(reset, 'Reset color');
        input.addEventListener('change', () => { const cur = this.ctx.store.fabric(f.id); if (cur) void this.setColor(cur, input.value); });
        reset.addEventListener('click', (e) => {
          e.preventDefault();
          if (this.cw !== null) void this.ctx.store.run('colorway.update', { id: this.cw, fabric_colors: { [String(f.id)]: null } });
          if (e.detail > 0) reset.blur();
        });
        return n;
      },
      update: (n, f) => {
        const c = this.colorOf(f);
        setStyleVar(n.children[0] as HTMLElement, '--sw', css(c));
        setText(n.children[1], f.name);
        const input = n.children[2] as HTMLInputElement;
        const hex = toHex(c);
        if (input.value !== hex && document.activeElement !== input) input.value = hex;
        const own = this.cw !== null && this.ctx.store.list.colorways.find((x) => x.id === this.cw)?.fabric_colors[String(f.id)] !== undefined;
        setDisabled(n.children[3] as HTMLButtonElement, !own);
      },
    });
    const has = this.cw !== null;
    setDisabled(this.btn.ren, !has);
    setDisabled(this.btn.del, !has);
    setDisabled(this.btn.act, has ? list.active_colorway === this.cw : list.active_colorway === null);
  }
}
