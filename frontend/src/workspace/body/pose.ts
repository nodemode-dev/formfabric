import { el, q, setDisabled } from '../../ui/dom';
import { icon } from '../../ui/icons';
import type { Ctx } from './ctx';
import { MAIN_JOINTS, type JointInfo } from './types';
import { label, selectRow, slideRow, type SelectRow, type SlideRow } from './widgets';

const POSES: Array<[string, string]> = [['rest', 'Rest pose'], ['A', 'A pose'], ['T', 'T pose']];
const AXES = ['X', 'Y', 'Z'];
const DEG = 180 / Math.PI;

export function eulerOf(q4: [number, number, number, number]): [number, number, number] {
  const [x, y, z, w] = q4;
  const m20 = 2 * (x * z - w * y);
  const yaw = Math.asin(Math.max(-1, Math.min(1, -m20)));
  let ex: number;
  let ez: number;
  if (Math.abs(m20) > 0.99999) {
    ex = 0;
    ez = Math.atan2(-2 * (x * y - w * z), 1 - 2 * (x * x + z * z));
  } else {
    ex = Math.atan2(2 * (y * z + w * x), 1 - 2 * (x * x + y * y));
    ez = Math.atan2(2 * (x * y + w * z), 1 - 2 * (y * y + z * z));
  }
  const r = (v: number): number => Math.round(v * DEG * 10) / 10 + 0;
  return [r(ex), r(yaw), r(ez)];
}

export class PoseSection {
  private readonly buttons = new Map<string, HTMLButtonElement>();
  private readonly joint: SelectRow;
  private readonly sliders: SlideRow[] = [];
  private readonly reset: HTMLButtonElement;
  private names: string[] = [];
  private current = '';

  constructor(host: HTMLElement, private readonly ctx: Ctx) {
    host.innerHTML = '<div class="ff-bd-tabs" data-k="tabs"></div>';
    const tabs = q(host, '[data-k="tabs"]');
    for (const [name, text] of POSES) {
      const b = el<HTMLButtonElement>('<button class="ff-bd-tab" type="button" disabled></button>');
      label(b, text);
      b.addEventListener('click', (e) => { void this.ctx.store.run('body.pose', { id: this.ctx.selected()?.id, name }); if (e.detail > 0) b.blur(); });
      tabs.appendChild(b);
      this.buttons.set(name, b);
    }
    this.joint = selectRow('Joint', [['', '-']], false, (v) => { this.current = v; this.paint(); });
    host.appendChild(this.joint.el);
    for (const axis of AXES) {
      const row = slideRow(axis, -180, 180, 1, () => this.commit());
      row.el.classList.add('ff-bd-axis');
      this.sliders.push(row);
      host.appendChild(row.el);
    }
    const actions = el('<div class="ff-bd-actions"></div>');
    this.reset = el<HTMLButtonElement>(`<button class="ff-btn" type="button" disabled>${icon('undo', 14)}<span></span></button>`);
    label(this.reset.children[1] as HTMLElement, 'Reset');
    this.reset.addEventListener('click', (e) => { this.zero(); if (e.detail > 0) this.reset.blur(); });
    actions.appendChild(this.reset);
    host.appendChild(actions);
  }

  private zero(): void {
    for (const s of this.sliders) { s.range.value = '0'; s.input.value = '0'; }
    this.commit();
  }

  private commit(): void {
    const body = this.ctx.selected();
    if (!body || !this.current) return;
    const euler = this.sliders.map((s) => s.range.valueAsNumber);
    void this.ctx.store.run('body.pose', { id: body.id, joints: { [this.current]: { euler } } });
  }

  private info(): JointInfo | undefined {
    return this.ctx.store.joints.find((j) => j.name === this.current);
  }

  private paint(): void {
    const j = this.info();
    const e = j ? eulerOf(j.pose) : null;
    this.sliders.forEach((s, i) => s.set(e ? e[i] : null));
  }

  update(): void {
    const body = this.ctx.selected();
    const joints = this.ctx.store.joints;
    const have = new Set(joints.map((j) => j.name));
    let names = MAIN_JOINTS.filter((n) => have.has(n));
    if (names.length === 0) names = joints.slice(0, 24).map((j) => j.name);
    if (names.join('|') !== this.names.join('|')) {
      this.names = names;
      this.joint.options(names.length ? names.map((n) => [n, n] as const) : [['', '-']], false);
    }
    if (!names.includes(this.current)) this.current = names[0] ?? '';
    this.joint.set(this.current);
    const off = !body || names.length === 0;
    for (const b of this.buttons.values()) setDisabled(b, !body);
    this.joint.disable(off);
    for (const s of this.sliders) s.disable(off);
    setDisabled(this.reset, off);
    this.paint();
  }
}
