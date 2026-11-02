import '../trims/trims.css';
import { el, setHidden } from '../../ui/dom';
import { onLanguage, t } from '../../ui/i18n';
import type { DocStore } from '../doc';
import { label } from '../fabric/widgets';
import { action, numRow, selRow, toggleRow } from '../trims/widgets';
import { styleLines, type LineMode, type StyleLineInfo } from './store';

const MODES: Array<[string, string]> = [['none', 'None'], ['stiffen', 'Stiffen'], ['fold', 'Fold']];

export class StyleLineSection {
  private readonly host = el<HTMLElement>('<div class="ff-tr-host"></div>');
  private readonly head = el<HTMLElement>('<header class="ff-ins-sub" hidden><span></span></header>');
  private readonly body = el<HTMLElement>('<div class="ff-tr-col" hidden></div>');
  private readonly pick;
  private readonly mode;
  private readonly closed;
  private readonly stiffen;
  private readonly width;
  private readonly angle;
  private readonly strength;
  private readonly remove;
  private shown = '';

  constructor(root: HTMLElement, private readonly doc: DocStore) {
    styleLines.attach(doc);
    label(this.head.firstElementChild as HTMLElement, 'Style Line');
    this.pick = selRow('Style Line', [], false, (v) => styleLines.select(Number(v) || null));
    this.mode = selRow('Mode', MODES, true, (v) => this.send({ mode: v }));
    this.closed = toggleRow('Closed', (on) => this.send({ closed: on }));
    this.stiffen = numRow('Stiffness', 'x', { min: 1, max: 1000, step: 1 }, (v) => this.send({ stiffen: Math.min(1000, Math.max(1, v)) }));
    this.width = numRow('Width', 'mm', { min: 0.5, max: 500, step: 0.5 }, (v) => this.send({ width: Math.min(500, Math.max(0.5, v)) }));
    this.angle = numRow('Fold angle', 'deg', { min: 0, max: 360, step: 1 }, (v) => this.send({ fold_angle: Math.min(360, Math.max(0, v)) }));
    this.strength = numRow('Fold strength', '', { min: 0, max: 1, step: 0.05 }, (v) => this.send({ fold_strength: Math.min(1, Math.max(0, v)) }));
    this.remove = action('Delete', 'delete', () => {
      const id = styleLines.selected;
      if (id !== null) void this.doc.run('styleline.remove', { id });
    });
    this.pick.disable(false);
    for (const r of [this.stiffen, this.width, this.angle, this.strength]) r.disable(false);
    this.body.append(this.pick.el, this.mode.el, this.closed.el, this.stiffen.el, this.width.el, this.angle.el, this.strength.el, this.remove.el);
    this.host.append(this.head, this.body);
    root.appendChild(this.host);
    styleLines.onChange(() => this.update());
    onLanguage(() => { this.shown = ''; this.update(); });
    this.update();
  }

  private send(args: Record<string, unknown>): void {
    const id = styleLines.selected;
    if (id === null) return;
    void this.doc.run('styleline.set', { id, ...args });
  }

  update(): void {
    const lines = styleLines.lines;
    setHidden(this.head, lines.length === 0);
    setHidden(this.body, lines.length === 0);
    if (!lines.length) return;
    const key = JSON.stringify([lines, styleLines.selected]);
    if (key === this.shown) return;
    this.shown = key;
    const items: Array<[string, string]> = [['', '-'], ...lines.map((l): [string, string] => [String(l.id), `${t('Style Line')} ${l.id}`])];
    this.pick.options(items, false);
    const cur: StyleLineInfo | null = styleLines.current;
    this.pick.set(cur ? String(cur.id) : '');
    const on = cur !== null;
    for (const c of [this.mode, this.closed]) c.disable(!on);
    this.remove.set(!on);
    const mode: LineMode = cur?.mode ?? 'none';
    this.mode.set(mode);
    this.closed.set(cur?.closed === true);
    this.stiffen.set(cur ? cur.stiffen : null);
    this.width.set(cur ? cur.width : null);
    this.angle.set(cur ? cur.fold_angle : null);
    this.strength.set(cur ? cur.fold_strength : null);
    this.stiffen.show(on && mode === 'stiffen');
    this.width.show(on && mode !== 'none');
    this.angle.show(on && mode === 'fold');
    this.strength.show(on && mode === 'fold');
  }
}
