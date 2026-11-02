import type { ShellDeps } from '../../shell/deps';
import type { Command } from '../../ui/commands';
import { patchList, el, q, setClass, setHidden, setText } from '../../ui/dom';
import { onLanguage, t } from '../../ui/i18n';
import { icon } from '../../ui/icons';
import type { DocStore } from '../doc';
import type { PatternView } from '../pattern-view/view';
import type { Selection } from '../state';
import { BakeDialog, GameReadyDialog } from './bake';
import { ExportDialog } from './export-dialog';
import { recall, remember } from './files';
import { GuideLayer } from './guides';
import { OutputStore } from './store';
import { UvEditor } from './uv';
import { bar, button, label, selectRow, slideRow, titled, toggleRow } from './widgets';
import type { Metric } from './types';

const OPEN_KEY = 'formfabric.output.open';

const SECTIONS: Array<[string, string, string]> = [
  ['export', 'Export', 'exportFile'],
  ['quad', 'Quad Mesh', 'quadMesh'],
  ['thick', 'Thickness', 'layers'],
  ['tools', 'UV and maps', 'uvEditor'],
];

const HTML =
  '<header class="ff-fb-head"><span class="ff-fb-title" data-k="title"></span><button class="cv-icon-btn" type="button" data-k="close"></button></header>' +
  '<div class="ff-fb-scroll">' +
  SECTIONS.map(([k, , ico]) => `<details class="ff-fb-sec" open data-s="${k}"><summary><span class="ff-fb-chev"></span>${icon(ico, 14)}<span data-k="h-${k}"></span></summary><div class="ff-fb-body" data-k="${k}"></div></details>`).join('') +
  '</div>';

const STATIC_FORMATS: Array<[string, string]> = [
  ['obj', 'OBJ'], ['gltf', 'glTF'], ['glb', 'GLB'], ['fbx', 'FBX'], ['usd', 'USD'], ['abc', 'Alembic'], ['mdd', 'MDD'], ['pc2', 'PC2'],
];

export class OutputPanel {
  readonly store: OutputStore;
  private readonly drawer: HTMLElement;
  private readonly toggle: HTMLButtonElement;
  private readonly uvToggle: HTMLButtonElement;
  private readonly uv: UvEditor;
  private readonly guides: GuideLayer;
  private exportDialog: ExportDialog | null = null;
  private bakeDialog: BakeDialog | null = null;
  private gameDialog: GameReadyDialog | null = null;
  private isOpen = false;
  private remeshing = false;

  private readonly format = selectRow('Format', [['obj', 'OBJ']]);
  private readonly exportBtn = button('Export...', 'primary');
  private readonly exportHint = el('<p class="ou-hint"></p>');

  private readonly density = slideRow('Density', 0.05, 1, 0.05, (v) => { remember('density', v); });
  private readonly remesh = button('Remesh', 'primary');
  private readonly clear = button('Clear');
  private readonly draw = button('Draw guides');
  private readonly clearGuides = button('Clear guides');
  private readonly progress = bar();
  private readonly status = el('<p class="ou-status"></p>');
  private readonly metrics = el('<div class="ou-metrics"></div>');
  private readonly quadHint = el('<p class="ou-hint"></p>');
  private readonly guideCount = el('<span class="ou-count"></span>');

  private readonly thickOn = toggleRow('Thickness');
  private readonly thickAmount = slideRow('Amount', 0.1, 10, 0.1, () => this.commitThickness());
  private readonly thickSides = toggleRow('Both sides');
  private readonly thickHint = el('<p class="ou-hint"></p>');

  private readonly uvBtn = button('UV Editor');
  private readonly bakeBtn = button('Bake...');
  private readonly gameBtn = button('Game-Ready...');
  private readonly toolsHint = el('<p class="ou-hint"></p>');

  constructor(private readonly root: HTMLElement, private readonly deps: ShellDeps, private readonly doc: DocStore, private readonly sel: Selection, view: PatternView) {
    this.store = new OutputStore(deps.engine, (m) => deps.toast(m, 'error'));
    const zoom = q(root, '[data-k="zoom"]');
    const make = (ico: string, key: string): HTMLButtonElement => {
      const b = el<HTMLButtonElement>(`<button class="cv-icon-btn ou-toggle-btn" type="button" aria-expanded="false">${icon(ico, 18)}</button>`);
      titled(b, key);
      zoom.parentElement?.insertBefore(b, zoom);
      return b;
    };
    this.toggle = make('exportFile', 'Output');
    this.uvToggle = make('uvEditor', 'UV Editor');

    this.drawer = el('<aside class="ff-fabric ou-drawer" data-k="output" hidden></aside>');
    this.drawer.innerHTML = HTML;
    q(root, '.ff-body').appendChild(this.drawer);
    for (const [k, text] of SECTIONS) label(q(this.drawer, `[data-k="h-${k}"]`), text);
    label(q(this.drawer, '[data-k="title"]'), 'Output');
    for (const c of this.drawer.querySelectorAll<HTMLElement>('.ff-fb-chev')) c.innerHTML = icon('chevronDown', 12);
    const close = q<HTMLButtonElement>(this.drawer, '[data-k="close"]');
    close.innerHTML = icon('close', 16);
    titled(close, 'Close');
    close.addEventListener('click', () => this.setOpen(false));
    this.toggle.addEventListener('click', () => { this.setOpen(!this.isOpen); this.toggle.blur(); });

    const part = (k: string): HTMLElement => q(this.drawer, `[data-k="${k}"].ff-fb-body`);
    part('export').append(this.format.el, this.exportBtn, this.exportHint);
    const guideRow = el('<div class="ou-btnrow"></div>');
    guideRow.append(this.draw, this.clearGuides, this.guideCount);
    const runRow = el('<div class="ou-btnrow"></div>');
    runRow.append(this.remesh, this.clear);
    part('quad').append(this.density.el, runRow, guideRow, this.progress.el, this.status, this.metrics, this.quadHint);
    part('thick').append(this.thickOn.el, this.thickAmount.el, this.thickSides.el, this.thickHint);
    const toolsRow = el('<div class="ou-btnrow ou-col"></div>');
    toolsRow.append(this.uvBtn, this.bakeBtn, this.gameBtn);
    part('tools').append(toolsRow, this.toolsHint);

    this.guides = new GuideLayer(q(root, '[data-k="pane2d"]'), view, doc, this.store, (on) => this.paint(on));
    this.uv = new UvEditor(q(root, '[data-k="pane2d"]'), deps, this.store, this.uvToggle, (on) => { if (on) this.guides.set(false); this.paint(); });

    this.exportBtn.addEventListener('click', () => { void this.showExport(this.format.get()); });
    this.remesh.addEventListener('click', () => { void this.runRemesh(); });
    this.clear.addEventListener('click', () => { void this.runClear(); });
    this.draw.addEventListener('click', () => { this.guides.set(!this.guides.active); this.draw.blur(); });
    this.clearGuides.addEventListener('click', () => { void this.guides.submit([]); });
    this.uvBtn.addEventListener('click', () => { this.uv.setOpen(!this.uv.isOpen); this.uvBtn.blur(); });
    this.bakeBtn.addEventListener('click', () => { this.bakeDialog ??= new BakeDialog(this.store, (m, k) => deps.toast(m, k)); void this.bakeDialog.present(); });
    this.gameBtn.addEventListener('click', () => { this.gameDialog ??= new GameReadyDialog(this.store, (m, k) => deps.toast(m, k)); void this.gameDialog.present(); });
    this.thickOn.onInput(() => this.commitThickness());
    this.thickSides.onInput(() => this.commitThickness());
    this.format.onInput(() => remember('format', this.format.get()));

    const saved = recall<number>('density', 0.5);
    this.density.set(saved);
    this.thickAmount.set(1);

    this.store.onChange(() => this.paint());
    doc.onChange(() => this.paintThickness());
    deps.engine.onEngineChanged((e) => {
      if (e.name === 'view.units') return;
      if (this.isOpen || this.guides.active) void this.store.loadRetopo();
    });
    onLanguage(() => this.paint());
    this.registerCommands();
    void this.start();
    if (localStorage.getItem(OPEN_KEY) === '1') this.setOpen(true);
  }

  private async start(): Promise<void> {
    for (let n = 0; n < 40; n++) {
      if (await this.store.loadCaps()) break;
      await new Promise((r) => setTimeout(r, 1500));
    }
    await Promise.all([this.store.loadFormats(), this.store.loadRetopo(), this.store.loadRange()]);
    this.syncFormatCommands();
    this.paint();
    if (this.store.retopo.density > 0 && this.store.has('retopo.get')) this.density.set(this.store.retopo.density);
  }

  private registerCommands(): void {
    const { commands } = this.deps;
    const list: Command[] = [
      { id: 'app.output.panel', label: 'Output Panel', icon: 'exportFile', checked: () => this.isOpen, run: () => this.setOpen(!this.isOpen) },
      { id: 'app.output.uv', label: 'UV Editor', icon: 'uvEditor', checked: () => this.uv.isOpen, run: () => this.uv.setOpen(!this.uv.isOpen) },
      { id: 'app.output.guides', label: 'Retopo Guides', enabled: () => this.store.has('retopo.set'), checked: () => this.guides.active, run: () => { this.setOpen(true); this.guides.set(!this.guides.active); } },
      { id: 'app.output.bake', label: 'Bake Maps...', enabled: () => this.store.has('bake.run'), run: () => { this.bakeBtn.click(); } },
      { id: 'app.output.gameready', label: 'Game-Ready...', enabled: () => this.store.has('gameready.run'), run: () => { this.gameBtn.click(); } },
    ];
    for (const [id, text] of STATIC_FORMATS) list.push(this.formatCommand(id, text));
    commands.add(...list);
  }

  private formatCommand(id: string, text: string): Command {
    return { id: `app.export.${id}`, label: `${text}...`, group: 'Export', enabled: () => this.store.has('export.run') && this.store.formats.some((f) => f.id === id), run: () => this.showExport(id) };
  }

  private syncFormatCommands(): void {
    for (const f of this.store.formats) {
      if (!this.deps.commands.get(`app.export.${f.id}`)) this.deps.commands.add(this.formatCommand(f.id, f.label));
    }
    this.deps.commands.onChange?.();
  }

  private async showExport(id?: string): Promise<void> {
    this.exportDialog ??= new ExportDialog(this.store, (m, k) => this.deps.toast(m, k));
    await this.exportDialog.show(id);
    if (id) remember('format', id);
  }

  setOpen(on: boolean): void {
    if (on === this.isOpen) return;
    this.isOpen = on;
    localStorage.setItem(OPEN_KEY, on ? '1' : '0');
    setClass(this.toggle, 'is-active', on);
    this.toggle.setAttribute('aria-expanded', String(on));
    setHidden(this.drawer, !on);
    if (on) { void this.store.loadRetopo(); this.paintThickness(); this.paint(); }
  }

  private selectedPanels(): number[] {
    const ids = this.sel.list();
    return ids.length ? ids : this.doc.garment.panels.map((p) => p.id);
  }

  private async runRemesh(): Promise<void> {
    if (this.remeshing) return;
    this.remeshing = true;
    this.progress.set(null);
    this.progress.show(true);
    setText(this.status, t('Remeshing...'));
    this.paint();
    const r = await this.store.attempt<Record<string, unknown>>('quad.remesh', { panels: this.selectedPanels(), density: this.density.range.valueAsNumber });
    this.remeshing = false;
    this.progress.show(false);
    if (!r.ok) { setText(this.status, r.message); this.deps.toast(r.message, 'error'); }
    else setText(this.status, t('Remeshed.'));
    await this.store.loadRetopo();
    this.paint();
  }

  private async runClear(): Promise<void> {
    const r = await this.store.attempt('quad.clear', { panels: this.selectedPanels() });
    if (!r.ok) this.deps.toast(r.message, 'error');
    else setText(this.status, t('Quad mesh cleared.'));
    await this.store.loadRetopo();
  }

  private thick = { enabled: false, amount: 1, sides: true };

  private paintThickness(): void {
    const out = (this.doc.garment as unknown as { output?: { thickness?: { enabled?: boolean; amount?: number; sides?: boolean } } }).output?.thickness;
    if (out) {
      this.thick = { enabled: out.enabled ?? this.thick.enabled, amount: out.amount ?? this.thick.amount, sides: out.sides ?? this.thick.sides };
    }
    this.thickOn.set(this.thick.enabled);
    this.thickAmount.set(this.thick.amount);
    this.thickSides.set(this.thick.sides);
  }

  private commitThickness(): void {
    this.thick = { enabled: this.thickOn.get(), amount: this.thickAmount.range.valueAsNumber, sides: this.thickSides.get() };
    void this.store.run('thickness.set', this.thick);
  }

  private paint(guiding = this.guides.active): void {
    if (!this.isOpen) return;
    const s = this.store;
    const miss = (ids: string[]): string => {
      const gone = ids.filter((id) => !s.has(id));
      return s.known && gone.length ? `${t('Not available in this engine build')}: ${gone.join(', ')}` : '';
    };
    const hint = (node: HTMLElement, text: string): void => { setText(node, text); setHidden(node, text === ''); };

    const known = s.formats.map((f) => [f.id, f.label] as const);
    const sel = this.format.select;
    const want = known.map((k) => k.join(':')).join('|');
    if (sel.dataset.sig !== want) {
      sel.dataset.sig = want;
      sel.replaceChildren();
      for (const [id, text] of known) { const o = document.createElement('option'); o.value = id; setText(o, text); sel.appendChild(o); }
      const last = recall<string>('format', 'obj');
      if (known.some(([id]) => id === last)) this.format.set(last);
    }
    const canExport = s.has('export.run') && known.length > 0;
    this.format.disable(!canExport);
    if (this.exportBtn.disabled !== !canExport) this.exportBtn.disabled = !canExport;
    hint(this.exportHint, miss(['export.formats', 'export.run']));

    const canRemesh = s.has('quad.remesh');
    if (this.remesh.disabled !== (!canRemesh || this.remeshing)) this.remesh.disabled = !canRemesh || this.remeshing;
    if (this.clear.disabled !== !s.has('quad.clear')) this.clear.disabled = !s.has('quad.clear');
    const canGuide = s.has('retopo.set');
    if (this.draw.disabled !== !canGuide) this.draw.disabled = !canGuide;
    if (this.clearGuides.disabled !== (!canGuide || s.retopo.guides.length === 0)) this.clearGuides.disabled = !canGuide || s.retopo.guides.length === 0;
    setClass(this.draw, 'is-on', guiding);
    this.density.disable(!canRemesh);
    setText(this.guideCount, s.retopo.guides.length ? String(s.retopo.guides.length) : '');
    hint(this.quadHint, miss(['quad.remesh', 'quad.clear', 'retopo.get', 'retopo.set']));
    patchList<Metric>(this.metrics, s.retopo.metrics, {
      key: (m) => m.key,
      create: () => el('<div class="ou-metric"><span></span><b></b></div>'),
      update: (n, m) => { setText(n.firstElementChild, m.label); setText(n.lastElementChild, m.value); },
    });
    setHidden(this.metrics, s.retopo.metrics.length === 0);

    const canThick = s.has('thickness.set');
    this.thickOn.disable(!canThick);
    this.thickAmount.disable(!canThick || !this.thickOn.get());
    this.thickSides.disable(!canThick || !this.thickOn.get());
    hint(this.thickHint, miss(['thickness.set']));

    this.uvBtn.disabled = false;
    setClass(this.uvBtn, 'is-on', this.uv.isOpen);
    if (this.bakeBtn.disabled !== !s.has('bake.run')) this.bakeBtn.disabled = !s.has('bake.run');
    if (this.gameBtn.disabled !== !s.has('gameready.run')) this.gameBtn.disabled = !s.has('gameready.run');
    hint(this.toolsHint, miss(['uv.get', 'uv.layout', 'uv.island.set', 'bake.run', 'gameready.run']));
  }
}
