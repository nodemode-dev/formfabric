import { el, q, setValue } from '../../ui/dom';
import type { Ctx } from './ctx';
import { flagBad, label, numField, type NumField } from './widgets';
import type { PhysicalKey } from './types';

const GROUPS: Array<[string, Array<[PhysicalKey, string, string, string]>]> = [
  ['Stretch', [['stretch_weft', 'Weft', 'N/m', '100'], ['stretch_warp', 'Warp', 'N/m', '100'], ['shear', 'Shear', 'N/m', '1']]],
  ['Bending', [['bending_weft', 'Weft', 'µN·m', '0.1'], ['bending_warp', 'Warp', 'µN·m', '0.1'], ['bending_bias', 'Bias', 'µN·m', '0.1']]],
  ['Buckling', [['buckling_ratio', 'Ratio', '0-1', '0.05'], ['buckling_stiffness', 'Stiffness', '0-1', '0.05']]],
  ['Mass and surface', [['density', 'Density', 'g/m²', '1'], ['friction', 'Friction', '0-1', '0.05'], ['thickness', 'Thickness', 'mm', '0.01'], ['damping', 'Damping', '0-1', '0.05']]],
];

export class PhysicalSection {
  private readonly fields = new Map<PhysicalKey, NumField>();
  private readonly name: HTMLInputElement;
  private hinted = false;

  constructor(host: HTMLElement, private readonly ctx: Ctx) {
    const nameRow = el('<label class="cv-field ff-fb-row ff-fb-name-row"><span></span><input class="cv-input" type="text" spellcheck="false" maxlength="64" disabled></label>');
    label(q(nameRow, 'span'), 'Name');
    this.name = q<HTMLInputElement>(nameRow, 'input');
    this.name.addEventListener('change', () => {
      const f = ctx.selected();
      const v = this.name.value.trim();
      if (f && v && v !== f.name) void ctx.store.run('fabric.update', { id: f.id, name: v });
      else if (f) this.name.value = f.name;
    });
    this.name.addEventListener('keydown', (e) => { if (e.key === 'Enter') this.name.blur(); });
    host.appendChild(nameRow);
    for (const [title, rows] of GROUPS) {
      const head = el('<h5 class="ff-fb-sub"></h5>');
      label(head, title);
      host.appendChild(head);
      for (const [key, text, unit, step] of rows) {
        const field = numField(text, unit, step, (v, input) => void this.commit(key, v, input));
        this.fields.set(key, field);
        host.appendChild(field.el);
      }
    }
  }

  private async commit(key: PhysicalKey, v: number, input: HTMLInputElement): Promise<void> {
    const f = this.ctx.selected();
    if (!f) return;
    if (v === f.physical[key]) return;
    const r = await this.ctx.store.attempt('fabric.update', { id: f.id, physical: { [key]: v } });
    if (r.ok) return;
    this.ctx.fail(r.message);
    flagBad(input, r.message, () => this.update());
  }

  update(): void {
    const f = this.ctx.selected();
    setValue(this.name, f?.name ?? '');
    if (this.name.disabled !== !f) this.name.disabled = !f;
    const defs = this.ctx.store.presets?.fields;
    for (const [key, field] of this.fields) {
      field.set(f ? f.physical[key] : null);
      field.disable(!f);
    }
    if (defs && !this.hinted) {
      this.hinted = true;
      for (const d of defs) {
        const field = this.fields.get(d.key);
        if (!field) continue;
        const tip = `${d.description} (${d.min} - ${d.max} ${d.unit})`;
        field.el.title = tip;
      }
    }
  }
}
