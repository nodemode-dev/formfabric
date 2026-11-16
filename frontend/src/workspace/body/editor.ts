import { el, q, setDisabled, setText, setValue } from '../../ui/dom';
import { icon } from '../../ui/icons';
import type { Ctx } from './ctx';
import { MEASURES } from './types';
import { label } from './widgets';

const head =
  '<div class="ff-bd-mrow ff-bd-mhead"><span data-k="h-m"></span><span data-k="h-now"></span><span data-k="h-target"></span><span></span></div>';

export class EditorSection {
  private readonly rows = new Map<string, { now: HTMLElement; input: HTMLInputElement }>();
  private readonly dirty = new Set<string>();
  private readonly apply: HTMLButtonElement;
  private readonly reset: HTMLButtonElement;
  private bodyId: number | null = null;
  private busy = false;

  constructor(host: HTMLElement, private readonly ctx: Ctx) {
    host.innerHTML = `<div class="ff-bd-table" data-k="table">${head}</div><div class="ff-bd-actions"></div>`;
    label(q(host, '[data-k="h-m"]'), 'Measurement');
    label(q(host, '[data-k="h-now"]'), 'Current');
    label(q(host, '[data-k="h-target"]'), 'Target');
    const table = q(host, '[data-k="table"]');
    for (const [key, text] of MEASURES) {
      const row = el(`<label class="ff-bd-mrow"><span data-k="n"></span><span class="ff-bd-now" data-k="now"></span><input class="cv-input ff-bd-num" type="number" min="100" max="5000" step="1" disabled><span class="ff-bd-unit">mm</span></label>`);
      label(q(row, '[data-k="n"]'), text);
      const input = q<HTMLInputElement>(row, 'input');
      input.addEventListener('change', () => {
        if (input.value.trim() === '' || !Number.isFinite(Number(input.value))) { this.dirty.delete(key); } else this.dirty.add(key);
        this.sync();
      });
      this.rows.set(key, { now: q(row, '[data-k="now"]'), input });
      table.appendChild(row);
    }
    const actions = q(host, '.ff-bd-actions');
    this.apply = el<HTMLButtonElement>(`<button class="ff-btn ff-bd-apply" type="button">${icon('check', 14)}<span></span></button>`);
    label(this.apply.children[1] as HTMLElement, 'Apply');
    this.reset = el<HTMLButtonElement>(`<button class="ff-btn" type="button">${icon('undo', 14)}<span></span></button>`);
    label(this.reset.children[1] as HTMLElement, 'Reset to original');
    this.apply.addEventListener('click', (e) => { void this.send(this.targets()); if (e.detail > 0) this.apply.blur(); });
    this.reset.addEventListener('click', (e) => { void this.restore(); if (e.detail > 0) this.reset.blur(); });
    actions.append(this.apply, this.reset);
  }

  private targets(): Record<string, number> {
    const out: Record<string, number> = {};
    for (const key of this.dirty) {
      const row = this.rows.get(key);
      const v = Number(row?.input.value);
      if (row && Number.isFinite(v)) out[key] = v;
    }
    return out;
  }

  private async send(targets: Record<string, number>): Promise<void> {
    const body = this.ctx.selected();
    if (!body || this.busy || Object.keys(targets).length === 0) return;
    this.busy = true;
    this.sync();
    const r = await this.ctx.store.run('body.resize', { id: body.id, ...targets });
    this.busy = false;
    if (r) this.dirty.clear();
    this.update();
  }

  private async restore(): Promise<void> {
    const body = this.ctx.selected();
    const original = body ? this.ctx.store.originals.get(body.id) : undefined;
    if (!original) return;
    const targets: Record<string, number> = {};
    for (const [key] of MEASURES) if (original[key] !== undefined) targets[key] = Math.round(original[key] * 10) / 10;
    await this.send(targets);
  }

  private sync(): void {
    const body = this.ctx.selected();
    const original = body ? this.ctx.store.originals.get(body.id) : undefined;
    const now = this.ctx.store.measures;
    const moved = !!original && MEASURES.some(([k]) => original[k] !== undefined && now[k] !== undefined && Math.abs(original[k] - now[k]) > 0.5);
    setDisabled(this.apply, !body || this.busy || this.dirty.size === 0);
    setDisabled(this.reset, !body || this.busy || !moved);
  }

  update(): void {
    const body = this.ctx.selected();
    const id = body?.id ?? null;
    if (id !== this.bodyId) { this.bodyId = id; this.dirty.clear(); }
    const now = this.ctx.store.measures;
    for (const [key, row] of this.rows) {
      const v = now[key];
      setText(row.now, v === undefined ? '-' : v.toFixed(1));
      if (row.input.disabled !== !body) row.input.disabled = !body;
      if (!this.dirty.has(key)) setValue(row.input, v === undefined ? '' : v.toFixed(1));
    }
    this.sync();
  }
}
