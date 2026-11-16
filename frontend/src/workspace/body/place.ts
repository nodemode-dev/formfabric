import { el, q, setDisabled, setHidden, setText } from '../../ui/dom';
import { icon } from '../../ui/icons';
import type { Ctx } from './ctx';
import { label, patchKeyed, selectRow, slideRow, titled, toggleField, type SelectRow, type SlideRow, type ToggleField } from './widgets';
import type { Proposal } from './types';

const numberOf = (v: unknown): number => (typeof v === 'number' && Number.isFinite(v) ? v : 0);

export class PlaceSection {
  private readonly point: SelectRow;
  private readonly orient: SlideRow;
  private readonly offset: SlideRow;
  private readonly flip: ToggleField;
  private readonly place: HTMLButtonElement;
  private readonly reset: HTMLButtonElement;
  private readonly count: HTMLElement;
  private readonly group: HTMLElement;
  private readonly seams: HTMLElement;
  private readonly list: HTMLElement;
  private readonly propose: HTMLButtonElement;
  private readonly applyAll: HTMLButtonElement;
  private keys = '';
  private proposed = false;
  private fetching = false;
  private proposals: Proposal[] = [];

  constructor(host: HTMLElement, private readonly ctx: Ctx) {
    this.group = el('<div class="ff-bd-group"></div>');
    host.appendChild(this.group);
    this.point = selectRow('Point', [['', '-']], false, (v) => this.pointChanged(v));
    this.orient = slideRow('Orientation', -360, 360, 15, () => undefined);
    this.offset = slideRow('Offset', -200, 1000, 1, () => undefined);
    this.flip = toggleField('Flip', () => undefined);
    this.group.append(this.point.el, this.offset.el, this.orient.el, this.flip.el);
    const actions = el('<div class="ff-bd-actions"></div>');
    this.place = el<HTMLButtonElement>(`<button class="ff-btn ff-bd-place" type="button" disabled>${icon('bodyPin', 14)}<span></span><span class="ff-bd-n"></span></button>`);
    label(this.place.children[1] as HTMLElement, 'Place selected panels');
    this.count = q(this.place, '.ff-bd-n');
    this.reset = el<HTMLButtonElement>(`<button class="ff-btn" type="button" disabled>${icon('undo', 14)}<span></span></button>`);
    label(this.reset.children[1] as HTMLElement, 'Reset placement');
    this.place.addEventListener('click', (e) => { void this.run(); if (e.detail > 0) this.place.blur(); });
    this.reset.addEventListener('click', (e) => { void this.clear(); if (e.detail > 0) this.reset.blur(); });
    actions.append(this.place, this.reset);
    this.group.appendChild(actions);

    this.seams = el('<div class="ff-bd-group"><p class="ff-bd-sub"></p><div class="ff-bd-list ff-bd-list-s" data-k="proposals"><p class="ff-bd-empty" data-k="none"></p></div><div class="ff-bd-actions"></div></div>');
    host.appendChild(this.seams);
    label(q(this.seams, '.ff-bd-sub'), 'Auto Seam');
    this.list = q(this.seams, '[data-k="proposals"]');
    label(q(this.seams, '[data-k="none"]'), 'No seam proposals.');
    const bar = q(this.seams, '.ff-bd-actions');
    this.propose = el<HTMLButtonElement>(`<button class="ff-btn" type="button">${icon('needle', 14)}<span></span></button>`);
    label(this.propose.children[1] as HTMLElement, 'Propose seams');
    this.applyAll = el<HTMLButtonElement>(`<button class="ff-btn" type="button" disabled>${icon('check', 14)}<span></span></button>`);
    label(this.applyAll.children[1] as HTMLElement, 'Apply all');
    this.propose.addEventListener('click', (e) => { this.proposed = true; void this.fetch(); if (e.detail > 0) this.propose.blur(); });
    this.applyAll.addEventListener('click', (e) => { void this.ctx.store.run('sew.auto', { apply: true }); if (e.detail > 0) this.applyAll.blur(); });
    bar.append(this.propose, this.applyAll);
  }

  available(): boolean {
    const s = this.ctx.store;
    return (s.has('place.points') && s.has('place.panel')) || s.has('sew.auto');
  }

  private panels(): number[] {
    return this.ctx.sel.list();
  }

  private pointChanged(v: string): void {
    const item = this.ctx.store.points.find((i) => i.key === v);
    if (item && item.offset !== undefined) { this.offset.set(item.offset); }
  }

  private async run(): Promise<void> {
    const panels = this.panels();
    const point = this.point.select.value;
    const body = this.ctx.selected();
    if (panels.length === 0 || !point || !body) return;
    const common: Record<string, unknown> = {
      body: body.id,
      point: /^\d+$/.test(point) ? Number(point) : point,
      offset: this.offset.range.valueAsNumber,
      orientation: this.orient.range.valueAsNumber,
      flip: this.flip.input.checked,
    };
    const group = panels.length > 1;
    if (group) await this.ctx.store.read('pattern.begin_drag', { label: 'Place Panels' });
    try {
      for (const panel of panels) {
        const r = await this.ctx.store.run('place.panel', { ...common, panel });
        if (r === undefined) break;
      }
    } finally {
      if (group) await this.ctx.store.read('pattern.end_drag', {});
    }
  }

  private async clear(): Promise<void> {
    const body = this.ctx.selected();
    const panels = this.panels();
    if (!body || !this.ctx.store.has('place.reset')) return;
    await this.ctx.store.run('place.reset', panels.length ? { body: body.id, panels } : { body: body.id });
  }

  private async fetch(): Promise<void> {
    if (this.fetching || !this.ctx.store.has('sew.auto')) return;
    this.fetching = true;
    const r = await this.ctx.store.read<{ proposals?: Array<Record<string, unknown>> }>('sew.auto', {});
    this.fetching = false;
    const out: Proposal[] = [];
    for (const p of r?.proposals ?? []) {
      const a = p.a as Record<string, unknown> | undefined;
      const b = p.b as Record<string, unknown> | undefined;
      if (!a || !b) continue;
      const args = { panel_a: a.panel, segment_a: a.segment, panel_b: b.panel, segment_b: b.segment, reversed: p.reversed !== false };
      out.push({ key: `${a.panel}.${a.segment}-${b.panel}.${b.segment}`, name: this.pairName(a, b), mismatch: numberOf(p.mismatch), args });
    }
    this.proposals = out;
    this.paintProposals();
  }

  private pairName(a: Record<string, unknown>, b: Record<string, unknown>): string {
    const name = (x: Record<string, unknown>): string => {
      const panel = this.ctx.doc.garment.panels.find((p) => p.id === x.panel);
      return `${panel?.name ?? `#${String(x.panel)}`} ${String(numberOf(x.segment) + 1)}`;
    };
    return `${name(a)} - ${name(b)}`;
  }

  private paintProposals(): void {
    patchKeyed<Proposal>(this.list, this.proposals, {
      key: (p) => p.key,
      create: (p) => {
        const node = el('<div class="ff-bd-item"><span class="ff-bd-name"></span><span class="ff-bd-tag"></span></div>');
        const go = el<HTMLButtonElement>(`<button class="cv-icon-btn ff-bd-add" type="button">${icon('check', 16)}</button>`);
        titled(go, 'Apply');
        go.addEventListener('click', (e) => { void this.ctx.store.run('sew.segment', p.args); if (e.detail > 0) go.blur(); });
        node.appendChild(go);
        return node;
      },
      update: (n, p) => {
        setText(q(n, '.ff-bd-name'), p.name);
        setText(q(n, '.ff-bd-tag'), `${p.mismatch > 0 ? '+' : ''}${p.mismatch.toFixed(1)} mm`);
      },
    });
    setHidden(q(this.list, '[data-k="none"]'), this.proposals.length > 0);
    setDisabled(this.applyAll, this.proposals.length === 0);
  }

  touch(): void {
    const n = this.panels().length;
    const body = this.ctx.selected();
    setText(this.count, n > 0 ? String(n) : '');
    setDisabled(this.place, !body || n === 0 || !this.point.select.value);
    setDisabled(this.reset, !body || !this.ctx.store.has('place.reset'));
  }

  update(): void {
    const { store } = this.ctx;
    const body = store.selected();
    const items = store.points;
    const sig = items.map((i) => i.key).join('|');
    if (sig !== this.keys) {
      this.keys = sig;
      this.point.options(items.length ? items.map((i) => [i.key, i.name] as const) : [['', '-']], false);
    }
    setHidden(this.group, !(store.has('place.points') && store.has('place.panel')));
    setHidden(this.seams, !store.has('sew.auto'));
    if (!items.some((i) => i.key === this.point.select.value)) {
      this.point.set(items[0]?.key ?? '');
      this.pointChanged(items[0]?.key ?? '');
    }
    const off = !body || items.length === 0;
    this.point.disable(off);
    this.orient.disable(off);
    this.offset.disable(off);
    this.flip.input.disabled = off;
    if (this.proposed) void this.fetch();
    this.touch();
  }
}
