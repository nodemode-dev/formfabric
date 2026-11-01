import './trims.css';
import { el, q, setHidden, setText } from '../../ui/dom';
import { onLanguage, t } from '../../ui/i18n';
import type { DocStore } from '../doc';
import { decimals, fromUnit, toUnit, type Selection } from '../state';
import type { Panel, Vec } from '../types';
import { fmtNum } from '../fabric/widgets';
import { itemLabel, removeTrim } from './actions';
import { localRuns, walkOf } from './scene';
import { trimHooks, trimState } from './state';
import {
  findPrint, num as numParam, printsOf, styleOf, trimDoc,
  type ItemKind, type PrintSide, type TrimItem,
} from './types';
import { action, label, numRow, outRow, pairRow, selRow, textRow, type Row } from './widgets';
import type { SeamEnd } from '../pattern-view/sew-types';

const SIDES: Array<[string, string]> = [['auto', 'Auto'], ['front', 'Front'], ['back', 'Back']];

function reverse(ends: SeamEnd[]): SeamEnd[] {
  return ends.slice().reverse().map((e) => ({ ...e, t0: e.t1, t1: e.t0 }));
}

export class TrimSection {
  private readonly rows: Array<[Row, ItemKind[]]> = [];
  private readonly printsHead: HTMLElement;
  private readonly printsBody: HTMLElement;
  private readonly itemHead: HTMLElement;
  private readonly itemBody: HTMLElement;
  private readonly title: HTMLElement;
  private readonly printPick;
  private readonly addPrint;
  private readonly style;
  private readonly styleOut;
  private readonly rot;
  private readonly pos;
  private readonly panelOut;
  private readonly fastOut;
  private readonly closed;
  private readonly lenA;
  private readonly lenB;
  private readonly mismatch;
  private readonly offsetOut;
  private readonly finishKind;
  private readonly finishWidth;
  private readonly finishRatio;
  private readonly finishStrength;
  private readonly finishGen;
  private readonly pname;
  private readonly pimage;
  private readonly psize;
  private readonly pside;
  private readonly pemboss;
  private readonly reverseBtn;
  private readonly unfastenBtn;
  private readonly deleteBtn;

  constructor(root: HTMLElement, private readonly doc: DocStore, private readonly sel: Selection, private readonly unit: () => string, private readonly redraw: () => void) {
    this.printsHead = el('<header class="ff-ins-sub"><span></span></header>');
    label(q(this.printsHead, 'span'), 'Prints');
    this.printsBody = el('<div class="ff-tr-col"></div>');
    this.itemHead = el('<header class="ff-ins-sub"><span></span></header>');
    this.title = q(this.itemHead, 'span');
    this.itemBody = el('<div class="ff-tr-col"></div>');
    root.append(this.printsHead, this.printsBody, this.itemHead, this.itemBody);

    this.printPick = selRow('Print', [['', '-']], false, (v) => {
      const id = Number(v);
      trimState.select(id > 0 ? { kind: 'print', id } : null);
    });
    this.addPrint = action('Add print', 'tPrint', () => {
      const id = this.single();
      if (id !== null) trimHooks.addPrint(id);
    });
    this.printsBody.append(this.printPick.el, this.addPrint.el);

    const add = <R extends Row>(row: R, kinds: ItemKind[]): R => {
      this.itemBody.appendChild(row.el);
      for (const c of row.el.querySelectorAll<HTMLInputElement | HTMLSelectElement>('input, select')) c.disabled = false;
      this.rows.push([row, kinds]);
      return row;
    };
    const run = (command: string, args: Record<string, unknown>): void => { void this.doc.run(command, args).then(() => this.redraw()); };
    const item = (): TrimItem | null => trimState.item;
    const idOf = (): number => item()?.id ?? 0;

    this.styleOut = add(outRow('Style'), ['hardware', 'stitch']);
    this.style = add(selRow('Style', [], false, (v) => run('zipper.set', { zipper: idOf(), style: Number(v) })), ['zipper']);
    this.rot = add(numRow('Rotation', '°', { step: 1 }, (v) => {
      const it = item();
      if (it?.kind === 'hardware') run('hardware.move', { hardware: it.id, rotation: v });
      else if (it?.kind === 'print') run('print.set', { print: it.id, rotation: v });
    }), ['hardware', 'print']);
    this.pos = add(pairRow('Position', 'mm', (x, y) => {
      const it = item();
      const u = this.unit();
      const point = [fromUnit(x, u), fromUnit(y, u)];
      if (it?.kind === 'hardware') run('hardware.move', { hardware: it.id, point, space: 'local' });
      else if (it?.kind === 'print') run('print.set', { print: it.id, position: point, space: 'local' });
    }), ['hardware', 'print']);
    this.panelOut = add(outRow('Panel'), ['hardware']);
    this.fastOut = add(outRow('Fastened to'), ['hardware']);
    this.closed = add(numRow('Closed', '', { min: 0, max: 1, step: 0.05 }, (v) => run('zipper.set', { zipper: idOf(), closed: Math.min(1, Math.max(0, v)) })), ['zipper']);
    this.lenA = add(outRow('Length A'), ['zipper']);
    this.lenB = add(outRow('Length B'), ['zipper']);
    this.mismatch = add(outRow('Mismatch'), ['zipper']);
    this.offsetOut = add(numRow('Offset', '', { step: 0.5 }, (v) => run('stitch.set', { stitch: idOf(), offset: Math.min(50, Math.max(-50, fromUnit(v, this.unit()))) })), ['stitch']);
    this.finishKind = add(outRow('Kind'), ['finish']);
    this.finishWidth = add(numRow('Width', '', { min: 0, step: 0.5 }, (v) => run('finish.set', { finish: idOf(), width: Math.min(100, Math.max(1, fromUnit(v, this.unit()))) })), ['finish']);
    this.finishRatio = add(numRow('Ratio', '', { min: 0.2, max: 2, step: 0.05 }, (v) => run('finish.set', { finish: idOf(), ratio: Math.min(2, Math.max(0.2, v)) })), ['finish']);
    this.finishStrength = add(numRow('Strength', '', { min: 0, max: 20, step: 0.5 }, (v) => run('finish.set', { finish: idOf(), strength: Math.min(20, Math.max(0, v)) })), ['finish']);
    this.finishGen = add(outRow('Generated panels'), ['finish']);
    this.pname = add(textRow('Name', (v) => { if (v) run('print.set', { print: idOf(), name: v }); }), ['print']);
    this.pimage = add(textRow('Image', (v) => { if (v) run('print.set', { print: idOf(), image: v }); }), ['print']);
    this.psize = add(pairRow('Size', 'mm', (w, h) => {
      const u = this.unit();
      if (w > 0 && h > 0) run('print.set', { print: idOf(), size: [fromUnit(w, u), fromUnit(h, u)] });
    }), ['print']);
    this.pside = add(selRow('Side', SIDES, true, (v) => run('print.set', { print: idOf(), side: v })), ['print']);
    this.pemboss = add(numRow('Emboss', 'mm', { min: -5, max: 5, step: 0.1 }, (v) => run('print.set', { print: idOf(), emboss: Math.min(5, Math.max(-5, v)) })), ['print']);

    const buttons = el('<div class="ff-tr-actions"></div>');
    this.reverseBtn = action('Reverse', null, () => { void this.reverseZipper(); });
    this.unfastenBtn = action('Unfasten', null, () => {
      const it = item();
      const h = it ? trimDoc(this.doc.garment).hardware.find((x) => x.id === it.id) : undefined;
      if (!h || h.fastened_to === null) return;
      const isButton = styleOf(trimDoc(this.doc.garment), h.style)?.kind === 'button';
      run('button.fasten', { button: isButton ? h.id : h.fastened_to, unfasten: true });
    });
    this.deleteBtn = action('Delete', 'delete', () => {
      const it = item();
      if (it) void removeTrim(this.doc, it).then(() => this.redraw());
    });
    buttons.append(this.reverseBtn.el, this.unfastenBtn.el, this.deleteBtn.el);
    this.itemBody.appendChild(buttons);

    trimState.onChange(() => this.update());
    onLanguage(() => this.update());
    this.update();
  }

  private single(): number | null {
    const ids = this.sel.list().filter((id) => this.doc.panel(id));
    return ids.length === 1 ? ids[0] : null;
  }

  private async reverseZipper(): Promise<void> {
    const it = trimState.item;
    if (!it || it.kind !== 'zipper') return;
    const z = trimDoc(this.doc.garment).zippers.find((x) => x.id === it.id);
    if (!z) return;
    const spans = (ends: SeamEnd[]): unknown[] => ends.map((e) => ({ panel: e.panel, ...(e.inner !== null ? { inner: e.inner } : {}), segment: e.segment, t0: e.t0, t1: e.t1 }));
    const ok = await this.doc.commit([
      ['zipper.delete', { zipper: z.id }],
      ['zipper.add', { style: z.style, a: spans(reverse(z.a)), b: spans(reverse(z.b)), closed: z.closed }],
    ], 'Reverse Zipper');
    if (ok) {
      const list = trimDoc(this.doc.garment).zippers;
      const last = list[list.length - 1];
      trimState.select(last ? { kind: 'zipper', id: last.id } : null);
    }
    this.redraw();
  }

  private lengthOf(ends: SeamEnd[]): number {
    const panels = new Map<number, Panel>(this.doc.garment.panels.map((p) => [p.id, p]));
    let total = 0;
    for (const r of localRuns(panels, ends)) total += walkOf(r.pts).total;
    return total;
  }

  update(): void {
    const g = this.doc.garment;
    const d = trimDoc(g);
    const u = this.unit();
    const dec = decimals(u) + 1;
    const len = (mm: number): string => `${toUnit(mm, u).toFixed(dec)} ${u}`;
    const pid = this.single();
    const panel = pid === null ? undefined : this.doc.panel(pid);
    setHidden(this.printsHead, !panel);
    setHidden(this.printsBody, !panel);
    if (panel) {
      const list = printsOf(panel);
      this.printPick.options([['', t('None')], ...list.map((p): [string, string] => [String(p.id), p.name ? `${p.id} · ${p.name}` : String(p.id)])], false);
      const cur = trimState.item;
      this.printPick.set(cur && cur.kind === 'print' && list.some((p) => p.id === cur.id) ? String(cur.id) : '');
    }

    const it = trimState.item;
    setHidden(this.itemHead, !it);
    setHidden(this.itemBody, !it);
    if (!it) return;
    setText(this.title, `${t(itemLabel(g, it))} ${it.id}`);
    for (const [row, kinds] of this.rows) row.show(kinds.includes(it.kind));
    this.reverseBtn.show(it.kind === 'zipper');
    this.unfastenBtn.show(it.kind === 'hardware');
    this.pos.unit(u);
    this.psize.unit(u);

    if (it.kind === 'hardware') {
      const h = d.hardware.find((x) => x.id === it.id);
      if (!h) return;
      const st = styleOf(d, h.style);
      this.styleOut.set(st ? st.name : '-');
      this.rot.set(h.rotation);
      this.rot.disable(false);
      this.pos.set([toUnit(h.anchor.point[0], u), toUnit(h.anchor.point[1], u)]);
      this.pos.disable(false);
      this.panelOut.set(this.doc.panel(h.anchor.panel)?.name ?? String(h.anchor.panel));
      this.fastOut.set(h.fastened_to === null ? '-' : `#${h.fastened_to}`);
      this.unfastenBtn.set(h.fastened_to === null);
    } else if (it.kind === 'zipper') {
      const z = d.zippers.find((x) => x.id === it.id);
      if (!z) return;
      const zs = d.styles.filter((s) => s.kind === 'zipper');
      this.style.options(zs.map((s): [string, string] => [String(s.id), s.name]), false);
      this.style.set(String(z.style));
      this.closed.set(z.closed);
      const la = this.lengthOf(z.a);
      const lb = this.lengthOf(z.b);
      this.lenA.set(len(la));
      this.lenB.set(len(lb));
      this.mismatch.set(`${lb - la > 0 ? '+' : ''}${toUnit(lb - la, u).toFixed(dec)} ${u}`);
    } else if (it.kind === 'stitch') {
      const s = d.stitches.find((x) => x.id === it.id);
      if (!s) return;
      const st = styleOf(d, s.style);
      this.styleOut.set(st ? st.name : '-');
      const fray = numParam(st?.params, 'fray', 0);
      this.offsetOut.unit(u);
      this.offsetOut.set(toUnit(fray > 0 ? fray : s.offset, u));
      this.offsetOut.disable(fray > 0);
    } else if (it.kind === 'finish') {
      const f = d.finishes.find((x) => x.id === it.id);
      if (!f) return;
      this.finishKind.set(t(f.kind.charAt(0).toUpperCase() + f.kind.slice(1)));
      this.finishWidth.unit(u);
      this.finishWidth.set(toUnit(f.width, u));
      this.finishRatio.show(f.kind === 'elastic');
      this.finishRatio.set(f.ratio);
      this.finishStrength.set(f.strength);
      this.finishGen.set(f.generated.length ? f.generated.map((id) => this.doc.panel(id)?.name ?? String(id)).join(', ') : '-');
    } else {
      const found = findPrint(g, it.id);
      if (!found) return;
      const p = found.print;
      this.pname.set(p.name);
      this.pimage.set(p.image);
      this.rot.set(p.rotation);
      this.rot.disable(false);
      this.pos.set([toUnit(p.position[0], u), toUnit(p.position[1], u)] as Vec);
      this.pos.disable(false);
      this.psize.set([toUnit(p.size[0], u), toUnit(p.size[1], u)]);
      this.pside.set(p.side satisfies PrintSide);
      this.pemboss.set(p.emboss);
    }
  }
}
