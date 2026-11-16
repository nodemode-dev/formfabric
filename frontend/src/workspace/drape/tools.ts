import type { CatalogueCommand } from '../../engine/service';
import { anyDialogOpen } from '../../ui/dialogs';
import { setClass, setDisabled, setTitle } from '../../ui/dom';
import { onLanguage, t } from '../../ui/i18n';
import { icon } from '../../ui/icons';
import { textFieldFocused } from '../../ui/native-menu';
import { comboOf, formatKeys, normalizeCombo } from '../../ui/commands';
import type { Tool, ToolOpts } from './types';
import { ensureStyle, numberField, selectField } from './ui';

interface Def {
  tool: Exclude<Tool, 'none'>;
  command: string;
  icon: string;
  label: string;
  hint: string;
}

const DEFS: Def[] = [
  { tool: 'pinch', command: 'sim.pinch', icon: 'pinchTool', label: 'Pinch', hint: 'Pull the cloth while it drapes' },
  { tool: 'box', command: 'pin.box', icon: 'pinBox', label: 'Pin (Box)', hint: 'Pin the vertices inside a dragged box' },
  { tool: 'lasso', command: 'pin.lasso', icon: 'pinLasso', label: 'Pin (Lasso)', hint: 'Pin the vertices inside a drawn outline' },
  { tool: 'fold', command: 'fold.place', icon: 'foldTool', label: 'Fold', hint: 'Drag a fold line across a panel' },
  { tool: 'tape', command: 'body.tape', icon: 'tapeTool', label: 'Body tape', hint: 'Click the cloth to hold it to the body' },
];

export class Tools {
  readonly opts: ToolOpts = { radius: 50, falloff: 'bump', power: 0, distance: 'straight', angle: 90, side: 'left', length: 0 };
  readonly root: HTMLElement[] = [];
  private current: Tool = 'none';
  private readonly buttons = new Map<Tool, HTMLButtonElement>();
  private readonly groups = new Map<Tool, HTMLElement>();
  private readonly keys = new Map<string, Tool>();
  private readonly shown = new Map<Tool, string>();
  private readonly have = new Set<Tool>();

  constructor(private readonly change: (tool: Tool) => void, private readonly usable: () => boolean) {
    ensureStyle();
    const bar = document.createElement('div');
    bar.className = 'ffd-group';
    for (const d of DEFS) {
      const b = document.createElement('button');
      b.type = 'button';
      b.className = 'ff3-btn';
      b.innerHTML = icon(d.icon, 14);
      b.disabled = true;
      b.addEventListener('click', (e) => { this.set(this.current === d.tool ? 'none' : d.tool); if (e.detail > 0) b.blur(); });
      bar.appendChild(b);
      this.buttons.set(d.tool, b);
    }
    this.root.push(bar);
    const pinch = this.group('pinch');
    pinch.append(
      numberField('Radius', 1, 500, 1, this.opts.radius, (v) => { this.opts.radius = v; }).root,
      selectField('Falloff', [['bump', 'Bump'], ['linear', 'Linear']], this.opts.falloff, (v) => { this.opts.falloff = v; }).root,
      numberField('Power', -2, 2, 0.1, this.opts.power, (v) => { this.opts.power = v; }).root,
      selectField('Distance', [['straight', 'Straight'], ['surface', 'Surface']], this.opts.distance, (v) => { this.opts.distance = v; }).root,
    );
    const fold = this.group('fold');
    fold.append(
      numberField('Angle', -180, 180, 5, this.opts.angle, (v) => { this.opts.angle = v; }).root,
      selectField('Side', [['left', 'Left'], ['right', 'Right']], this.opts.side, (v) => { this.opts.side = v; }).root,
    );
    const tape = this.group('tape');
    tape.append(numberField('Length', 0, 1000, 1, this.opts.length, (v) => { this.opts.length = v; }).root);
    document.addEventListener('keydown', (e) => this.onKey(e), true);
    this.labels();
    onLanguage(() => this.labels());
  }

  get active(): Tool { return this.current; }

  private group(tool: Tool): HTMLElement {
    const g = document.createElement('div');
    g.className = 'ffd-opts';
    g.hidden = true;
    this.groups.set(tool, g);
    this.root.push(g);
    return g;
  }

  set(tool: Tool): void {
    if (tool !== 'none' && !this.have.has(tool)) tool = 'none';
    if (tool === this.current) return;
    this.current = tool;
    for (const [id, b] of this.buttons) setClass(b, 'is-on', id === tool);
    for (const [id, g] of this.groups) if (g.hidden === (id === tool)) g.hidden = id !== tool;
    this.change(tool);
  }

  configure(commands: CatalogueCommand[]): void {
    this.keys.clear();
    this.have.clear();
    for (const d of DEFS) {
      const c = commands.find((x) => x.id === d.command);
      if (c) this.have.add(d.tool);
      this.shown.set(d.tool, c && c.keys.length ? formatKeys(normalizeCombo(c.keys[0])) : '');
      for (const k of c?.keys ?? []) this.keys.set(normalizeCombo(k), d.tool);
      setDisabled(this.buttons.get(d.tool) as HTMLButtonElement, !c);
    }
    if (this.current !== 'none' && !this.have.has(this.current)) this.set('none');
    this.labels();
  }

  private labels(): void {
    for (const d of DEFS) {
      const key = this.shown.get(d.tool) ?? '';
      setTitle(this.buttons.get(d.tool) as HTMLButtonElement, `${t(d.label)} - ${t(d.hint)}${key ? ` (${key})` : ''}`);
    }
  }

  private onKey(e: KeyboardEvent): void {
    if (e.defaultPrevented || this.keys.size === 0 || !this.usable()) return;
    if (anyDialogOpen() || textFieldFocused() || document.querySelector('.cv-home:not([hidden])')) return;
    const tool = this.keys.get(comboOf(e));
    if (!tool) return;
    e.preventDefault();
    e.stopPropagation();
    this.set(this.current === tool ? 'none' : tool);
  }
}
