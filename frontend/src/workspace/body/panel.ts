import type { ShellDeps } from '../../shell/deps';
import { el, q, setClass, setHidden } from '../../ui/dom';
import { icon } from '../../ui/icons';
import type { DocStore } from '../doc';
import type { Selection } from '../state';
import { BodiesSection, ImportSection, LibrarySection } from './bodies';
import { CollisionSection } from './collide';
import type { Ctx } from './ctx';
import { EditorSection } from './editor';
import { PlaceSection } from './place';
import { PoseSection } from './pose';
import { BodyStore } from './store';
import type { BodyInfo } from './types';
import { label, repaint, titled } from './widgets';

const OPEN_KEY = 'formfabric.body.open';

const SECTIONS: Array<[string, string, string]> = [
  ['bodies', 'Bodies', 'body'],
  ['library', 'Add from library', 'folder'],
  ['import', 'Import Body', 'fileIn'],
  ['editor', 'Body Editor', 'ruler'],
  ['pose', 'Poses', 'pose'],
  ['collision', 'Collision', 'layers'],
  ['place', 'Placement', 'bodyPin'],
];

const HTML =
  '<header class="ff-bd-head"><span class="ff-bd-title" data-k="title"></span><button class="cv-icon-btn" type="button" data-k="close"></button></header>' +
  '<div class="ff-bd-scroll">' +
  SECTIONS.map(([k, , ico]) => `<details class="ff-bd-sec" open data-s="${k}"><summary><span class="ff-bd-chev"></span>${icon(ico, 14)}<span data-k="h-${k}"></span></summary><div class="ff-bd-body" data-k="${k}"></div></details>`).join('') +
  '</div>';

export class BodyPanel implements Ctx {
  readonly store: BodyStore;
  private readonly drawer: HTMLElement;
  private readonly toggle: HTMLButtonElement;
  private readonly bodies: BodiesSection;
  private readonly library: LibrarySection;
  private readonly editor: EditorSection;
  private readonly pose: PoseSection;
  private readonly collision: CollisionSection;
  private readonly place: PlaceSection;
  private readonly placeSec: HTMLElement;
  private isOpen = false;
  private touching = false;
  private yielding = false;

  constructor(root: HTMLElement, private readonly deps: ShellDeps, readonly doc: DocStore, readonly sel: Selection) {
    this.store = new BodyStore(deps.engine, (m) => deps.toast(m, 'error'));
    this.toggle = el<HTMLButtonElement>(`<button class="cv-icon-btn ff-bd-toggle" type="button" aria-expanded="false">${icon('body', 18)}</button>`);
    titled(this.toggle, 'Body');
    const zoom = q(root, '[data-k="zoom"]');
    zoom.parentElement?.insertBefore(this.toggle, zoom);

    this.drawer = el('<aside class="ff-body-drawer" data-k="body" hidden></aside>');
    this.drawer.innerHTML = HTML;
    q(root, '.ff-body').appendChild(this.drawer);
    for (const [k, text] of SECTIONS) label(q(this.drawer, `[data-k="h-${k}"]`), text);
    label(q(this.drawer, '[data-k="title"]'), 'Body');
    for (const c of this.drawer.querySelectorAll<HTMLElement>('.ff-bd-chev')) c.innerHTML = icon('chevronDown', 12);
    const close = q<HTMLButtonElement>(this.drawer, '[data-k="close"]');
    close.innerHTML = icon('close', 16);
    titled(close, 'Close');
    close.addEventListener('click', () => { void this.setOpen(false); });
    this.toggle.addEventListener('click', () => { void this.setOpen(!this.isOpen); this.toggle.blur(); });
    root.addEventListener('click', (e) => {
      if (this.isOpen && !this.yielding && e.target instanceof Element && e.target.closest('.ff-fb-toggle')) void this.setOpen(false);
    }, true);

    const part = (k: string): HTMLElement => q(this.drawer, `[data-k="${k}"].ff-bd-body`);
    this.bodies = new BodiesSection(part('bodies'), this);
    this.library = new LibrarySection(part('library'), this);
    new ImportSection(part('import'), this);
    this.editor = new EditorSection(part('editor'), this);
    this.pose = new PoseSection(part('pose'), this);
    this.collision = new CollisionSection(part('collision'), this);
    this.place = new PlaceSection(part('place'), this);
    this.placeSec = q(this.drawer, '[data-s="place"]');
    setHidden(this.placeSec, true);

    this.store.onChange(() => this.render());
    doc.onChange(() => { if (this.isOpen) this.place.touch(); });
    deps.engine.onEngineChanged((e) => {
      if (e.name === 'view.units') return;
      if (this.isOpen) void this.store.refresh(); else this.store.stale = true;
    });
    const touch = (): void => {
      if (this.touching || !this.isOpen) return;
      this.touching = true;
      requestAnimationFrame(() => { this.touching = false; this.place.touch(); });
    };
    document.addEventListener('pointerup', touch, true);
    document.addEventListener('keyup', touch, true);
    repaint(() => this.render());

    if (!deps.commands.get('app.body')) {
      deps.commands.add({ id: 'app.body', label: 'Body Panel', group: 'Body', icon: 'body', run: () => { void this.setOpen(!this.isOpen); }, checked: () => this.isOpen });
    }
    if (localStorage.getItem(OPEN_KEY) === '1') void this.setOpen(true);
    else setClass(this.toggle, 'is-active', false);
  }

  selected(): BodyInfo | undefined {
    return this.store.selected();
  }

  info(message: string): void {
    this.deps.toast(message, 'info');
  }

  fail(message: string): void {
    this.deps.toast(message, 'error');
  }

  async setOpen(on: boolean): Promise<void> {
    if (on === this.isOpen) return;
    this.isOpen = on;
    localStorage.setItem(OPEN_KEY, on ? '1' : '0');
    setClass(this.toggle, 'is-active', on);
    this.toggle.setAttribute('aria-expanded', String(on));
    if (on) {
      const other = document.querySelector<HTMLElement>('.ff-fb-toggle.is-active');
      if (other) {
        this.yielding = true;
        other.click();
        this.yielding = false;
      }
      if (this.store.stale || !this.store.loaded) await this.store.refresh();
      if (!this.isOpen) return;
      this.render();
    }
    setHidden(this.drawer, !on);
  }

  render(): void {
    if (!this.isOpen) return;
    this.bodies.update();
    this.library.update();
    this.editor.update();
    this.pose.update();
    this.collision.update();
    setHidden(this.placeSec, !this.place.available());
    this.place.update();
  }
}
