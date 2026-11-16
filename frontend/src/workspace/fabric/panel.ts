import type { ShellDeps } from '../../shell/deps';
import { el, q, setClass, setHidden } from '../../ui/dom';
import { icon } from '../../ui/icons';
import type { DocStore } from '../doc';
import type { Selection } from '../state';
import { ColorwaySection } from './colorways';
import type { Ctx } from './ctx';
import { FabricLab } from './lab';
import { FabricsSection } from './list';
import { LookSection } from './look';
import { PhysicalSection } from './physical';
import { FabricStore } from './store';
import type { FabricInfo } from './types';
import { label, repaint, titled } from './widgets';

const OPEN_KEY = 'formfabric.fabric.open';

const SECTIONS: Array<[string, string, string]> = [
  ['fabrics', 'Fabrics', 'fabric'],
  ['physical', 'Physical properties', 'pfPerformance'],
  ['look', 'Look', 'eye'],
  ['colorways', 'Colorways', 'colorway'],
];

const HTML =
  '<header class="ff-fb-head"><span class="ff-fb-title" data-k="title"></span><button class="cv-icon-btn" type="button" data-k="close"></button></header>' +
  '<div class="ff-fb-scroll">' +
  SECTIONS.map(([k, , ico]) => `<details class="ff-fb-sec" open data-s="${k}"><summary><span class="ff-fb-chev"></span>${icon(ico, 14)}<span data-k="h-${k}"></span></summary><div class="ff-fb-body" data-k="${k}"></div></details>`).join('') +
  '</div>';

export class FabricPanel implements Ctx {
  selId: number | null = null;
  readonly store: FabricStore;
  private readonly drawer: HTMLElement;
  private readonly toggle: HTMLButtonElement;
  private readonly fabrics: FabricsSection;
  private readonly physical: PhysicalSection;
  private readonly look: LookSection;
  private readonly colorways: ColorwaySection;
  private lab: FabricLab | null = null;
  private isOpen = false;
  private touching = false;

  constructor(root: HTMLElement, private readonly deps: ShellDeps, readonly doc: DocStore, readonly sel: Selection) {
    this.store = new FabricStore(deps.engine, (m) => deps.toast(m, 'error'));
    this.toggle = el<HTMLButtonElement>(`<button class="cv-icon-btn ff-fb-toggle" type="button" aria-expanded="false">${icon('fabric', 18)}</button>`);
    titled(this.toggle, 'Fabric');
    const zoom = q(root, '[data-k="zoom"]');
    zoom.parentElement?.insertBefore(this.toggle, zoom);

    this.drawer = el('<aside class="ff-fabric" data-k="fabric" hidden></aside>');
    this.drawer.innerHTML = HTML;
    q(root, '.ff-body').appendChild(this.drawer);
    for (const [k, text] of SECTIONS) label(q(this.drawer, `[data-k="h-${k}"]`), text);
    label(q(this.drawer, '[data-k="title"]'), 'Fabric');
    for (const c of this.drawer.querySelectorAll<HTMLElement>('.ff-fb-chev')) c.innerHTML = icon('chevronDown', 12);
    const close = q<HTMLButtonElement>(this.drawer, '[data-k="close"]');
    close.innerHTML = icon('close', 16);
    titled(close, 'Close');
    close.addEventListener('click', () => this.setOpen(false));
    this.toggle.addEventListener('click', (e) => { this.setOpen(!this.isOpen); if (e.detail > 0) this.toggle.blur(); });

    const part = (k: string): HTMLElement => q(this.drawer, `[data-k="${k}"].ff-fb-body`);
    this.fabrics = new FabricsSection(part('fabrics'), this);
    this.physical = new PhysicalSection(part('physical'), this);
    this.look = new LookSection(part('look'), this);
    this.colorways = new ColorwaySection(part('colorways'), this);

    this.store.onChange(() => this.render());
    doc.onChange(() => this.render());
    deps.engine.onEngineChanged((e) => {
      if (e.name === 'view.units') return;
      if (this.isOpen) void this.store.refresh(); else this.store.stale = true;
    });
    const touch = (): void => {
      if (this.touching || !this.isOpen) return;
      this.touching = true;
      requestAnimationFrame(() => { this.touching = false; this.fabrics.touch(); });
    };
    document.addEventListener('pointerup', touch, true);
    document.addEventListener('keyup', touch, true);
    repaint(() => this.render());

    void this.store.loadPresets();
    if (localStorage.getItem(OPEN_KEY) === '1') void this.setOpen(true);
    else setClass(this.toggle, 'is-active', false);
  }

  selected(): FabricInfo | undefined {
    const { list } = this.store;
    return this.store.fabric(this.selId) ?? this.store.fabric(list.default_fabric) ?? list.fabrics[0];
  }

  select(id: number | null): void {
    this.selId = id;
    this.render();
  }

  info(message: string): void {
    this.deps.toast(message, 'info');
  }

  fail(message: string): void {
    this.deps.toast(message, 'error');
  }

  openLab(): void {
    this.lab ??= new FabricLab(this);
    this.lab.show(this.selected()?.id ?? null);
  }

  async setOpen(on: boolean): Promise<void> {
    if (on === this.isOpen) return;
    this.isOpen = on;
    localStorage.setItem(OPEN_KEY, on ? '1' : '0');
    setClass(this.toggle, 'is-active', on);
    this.toggle.setAttribute('aria-expanded', String(on));
    if (on) {
      for (const other of document.querySelectorAll<HTMLElement>('.ff-body > aside[data-k]:not([hidden])')) {
        if (other !== this.drawer) other.querySelector<HTMLButtonElement>('[data-k="close"]')?.click();
      }
      void this.store.loadPresets();
      if (this.store.stale || !this.store.loaded) await this.store.refresh();
      if (!this.isOpen) return;
      this.render();
    }
    setHidden(this.drawer, !on);
  }

  render(): void {
    if (!this.isOpen) return;
    this.fabrics.update();
    this.physical.update();
    this.look.update();
    this.colorways.update();
    this.lab?.sync();
  }
}
