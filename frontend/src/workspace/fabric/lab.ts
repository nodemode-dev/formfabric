import { Dialog } from '../../ui/dialogs';
import { el, q, setClass, setDisabled, setText } from '../../ui/dom';
import { enhanceSelect } from '../../ui/dropdown';
import { t } from '../../ui/i18n';
import type { Ctx } from './ctx';
import { fmtNum, label, later, patchKeyed, repaint } from './widgets';
import type { LabReply, PhysicalKey } from './types';

interface Cell { key: string; unit: string; hint?: string }

const ROWS: Array<[string, string, string, Cell[]]> = [
  ['Sample', 'Weigh a sample of known area and measure its thickness. Ounces per square yard times 33.906 gives grams per square metre.', 'Density', [{ key: 'density', unit: 'g/m²' }]],
  ['', '', 'Thickness', [{ key: 'thickness', unit: 'mm' }]],
  ['Cantilever bending length', 'Bending length c: half the overhang at which the strip tip drops to 41.5 degrees. Bending rigidity B = 9.807e-6 x W x c^3 (Peirce); bias defaults to the geometric mean of warp and weft.', 'Warp', [{ key: 'bl.warp', unit: 'cm' }]],
  ['', '', 'Weft', [{ key: 'bl.weft', unit: 'cm' }]],
  ['', '', 'Bias', [{ key: 'bl.bias', unit: 'cm' }]],
  ['Tensile strip', 'Force on a strip at a small strain, 1 to 2 percent. Stretch is force per width divided by strain. Both force and strain are needed.', 'Warp', [{ key: 'tw.force', unit: 'N', hint: 'Force' }, { key: 'tw.strain', unit: '%', hint: 'Strain' }, { key: 'tw.width', unit: 'mm', hint: 'Width' }]],
  ['', '', 'Weft', [{ key: 'tf.force', unit: 'N', hint: 'Force' }, { key: 'tf.strain', unit: '%', hint: 'Strain' }, { key: 'tf.width', unit: 'mm', hint: 'Width' }]],
  ['Shear', 'Picture frame test: force along a frame side at a shear angle. Shear stiffness is force per length divided by the tangent of the angle. Without a frame, the bias extension at 4.903 N/m gives 123 divided by EB5.', 'Picture frame', [{ key: 'sh.angle', unit: '°', hint: 'Angle' }, { key: 'sh.force', unit: 'N', hint: 'Force' }, { key: 'sh.length', unit: 'mm', hint: 'Length' }]],
  ['', '', 'Bias extension EB5', [{ key: 'eb5', unit: '%' }]],
  ['Other', 'Values you do not measure stay as the target fabric has them.', 'Friction', [{ key: 'friction', unit: '0-1' }]],
  ['', '', 'Damping', [{ key: 'damping', unit: '0-1' }]],
  ['', '', 'Buckling ratio', [{ key: 'buckling_ratio', unit: '0-1' }]],
  ['', '', 'Buckling stiffness', [{ key: 'buckling_stiffness', unit: '0-1' }]],
];

const OUT: Array<[PhysicalKey, string, string, string]> = [
  ['stretch_weft', 'Stretch', 'Weft', 'N/m'], ['stretch_warp', 'Stretch', 'Warp', 'N/m'], ['shear', 'Shear', '', 'N/m'],
  ['bending_weft', 'Bending', 'Weft', 'µN·m'], ['bending_warp', 'Bending', 'Warp', 'µN·m'], ['bending_bias', 'Bending', 'Bias', 'µN·m'],
  ['buckling_ratio', 'Buckling', 'Ratio', '0-1'], ['buckling_stiffness', 'Buckling', 'Stiffness', '0-1'],
  ['density', 'Density', '', 'g/m²'], ['friction', 'Friction', '', '0-1'], ['thickness', 'Thickness', '', 'mm'], ['damping', 'Damping', '', '0-1'],
];

const HTML =
  '<div class="ff-lab">' +
  '<div class="ff-lab-in" data-k="in"></div>' +
  '<div class="ff-lab-out">' +
  '<div class="ff-lab-target"><span data-k="targetl"></span><select class="cv-input" data-k="target"></select></div>' +
  '<div class="ff-lab-table" data-k="table"><div class="ff-lab-th"><span></span><span data-k="thnow"></span><span data-k="thnew"></span><span></span></div></div>' +
  '<p class="ff-lab-msg" data-k="msg"></p>' +
  '<h5 class="ff-fb-sub" data-k="fl"></h5>' +
  '<div class="ff-lab-formulas" data-k="formulas"></div>' +
  '<h5 class="ff-fb-sub" data-k="wl"></h5>' +
  '<div class="ff-lab-warn" data-k="warn"></div>' +
  '</div></div>';

export class FabricLab extends Dialog {
  private readonly inputs = new Map<string, HTMLInputElement>();
  private readonly cells = new Map<PhysicalKey, { now: HTMLElement; next: HTMLElement; row: HTMLElement }>();
  private readonly target: HTMLSelectElement;
  private readonly msg: HTMLElement;
  private readonly formulas: HTMLElement;
  private readonly warn: HTMLElement;
  private readonly apply: HTMLButtonElement;
  private readonly wait = later(260);
  private seq = 0;
  private ready = false;

  constructor(private readonly ctx: Ctx) {
    super('Fabric Lab', 820);
    this.body.innerHTML = HTML;
    const form = q(this.body, '[data-k="in"]');
    for (const [title, help, name, cells] of ROWS) {
      if (title) {
        const h = el('<h5 class="ff-fb-sub"></h5>');
        label(h, title);
        const p = el('<p class="ff-lab-help"></p>');
        label(p, help);
        form.append(h, p);
      }
      const row = el('<div class="ff-lab-row"><span class="ff-lab-l"></span></div>');
      label(q(row, 'span'), name);
      row.style.setProperty('--n', String(cells.length));
      for (const c of cells) {
        const cell = el('<span class="ff-lab-cell"><input class="cv-input ff-fb-num" type="number" step="any"><i></i></span>');
        const input = q<HTMLInputElement>(cell, 'input');
        setText(q(cell, 'i'), c.unit);
        if (c.hint) repaint(() => { const s = t(c.hint ?? ''); input.placeholder = s; input.title = s; });
        input.addEventListener('input', () => this.schedule());
        input.addEventListener('change', () => this.schedule());
        this.inputs.set(c.key, input);
        row.appendChild(cell);
      }
      form.appendChild(row);
    }
    label(q(this.body, '[data-k="targetl"]'), 'Target fabric');
    label(q(this.body, '[data-k="thnow"]'), 'Current');
    label(q(this.body, '[data-k="thnew"]'), 'Result');
    label(q(this.body, '[data-k="fl"]'), 'Formulas');
    label(q(this.body, '[data-k="wl"]'), 'Warnings');
    const table = q(this.body, '[data-k="table"]');
    for (const [key, group, axis, unit] of OUT) {
      const row = el('<div class="ff-lab-tr"><span class="ff-lab-name"></span><span class="ff-lab-now"></span><span class="ff-lab-new"></span><span class="ff-lab-unit"></span></div>');
      repaint(() => setText(q(row, '.ff-lab-name'), axis ? `${t(group)} ${t(axis)}` : t(group)));
      setText(q(row, '.ff-lab-unit'), unit);
      table.appendChild(row);
      this.cells.set(key, { now: q(row, '.ff-lab-now'), next: q(row, '.ff-lab-new'), row });
    }
    this.target = q<HTMLSelectElement>(this.body, '[data-k="target"]');
    enhanceSelect(this.target);
    this.target.addEventListener('change', () => { this.showCurrent(); this.schedule(); });
    this.msg = q(this.body, '[data-k="msg"]');
    this.formulas = q(this.body, '[data-k="formulas"]');
    this.warn = q(this.body, '[data-k="warn"]');
    this.button('Close', 'ghost', () => this.close());
    this.apply = this.button('Apply to fabric', 'primary', () => void this.commit());
    this.apply.disabled = true;
  }

  show(fabricId: number | null): void {
    const fabrics = this.ctx.store.list.fabrics;
    this.target.replaceChildren(...fabrics.map((f) => { const o = document.createElement('option'); o.value = String(f.id); o.textContent = f.name; return o; }));
    const pick = fabricId !== null && fabrics.some((f) => f.id === fabricId) ? fabricId : fabrics[0]?.id;
    if (pick !== undefined) this.target.value = String(pick);
    this.ready = true;
    this.showCurrent();
    this.open(this.inputs.get('density'));
    this.schedule();
  }

  sync(): void {
    if (this.isOpen) this.showCurrent();
  }

  private targetId(): number | null {
    const n = Number(this.target.value);
    return Number.isFinite(n) && this.target.value !== '' ? n : null;
  }

  private showCurrent(): void {
    const f = this.ctx.store.fabric(this.targetId());
    for (const [key, c] of this.cells) setText(c.now, f ? fmtNum(f.physical[key]) : '');
  }

  private num(key: string): number | undefined {
    const input = this.inputs.get(key);
    if (!input || input.value.trim() === '') return undefined;
    const v = Number(input.value);
    return Number.isFinite(v) ? v : undefined;
  }

  private args(): Record<string, unknown> {
    const out: Record<string, unknown> = {};
    const plain = (k: string, name: string): void => { const v = this.num(k); if (v !== undefined) out[name] = v; };
    plain('density', 'density');
    plain('thickness', 'thickness');
    plain('friction', 'friction');
    plain('damping', 'damping');
    plain('buckling_ratio', 'buckling_ratio');
    plain('buckling_stiffness', 'buckling_stiffness');
    plain('eb5', 'bias_extension');
    const bl: Record<string, number> = {};
    for (const k of ['warp', 'weft', 'bias']) { const v = this.num(`bl.${k}`); if (v !== undefined) bl[k] = v; }
    if (Object.keys(bl).length) out.bending_length = bl;
    const tensile: Record<string, unknown> = {};
    for (const [k, p] of [['warp', 'tw'], ['weft', 'tf']] as const) {
      const force = this.num(`${p}.force`);
      const strain = this.num(`${p}.strain`);
      const width = this.num(`${p}.width`);
      if (force !== undefined && strain !== undefined) tensile[k] = { force, strain, ...(width !== undefined ? { width } : {}) };
    }
    if (Object.keys(tensile).length) out.tensile = tensile;
    const angle = this.num('sh.angle');
    const force = this.num('sh.force');
    const length = this.num('sh.length');
    if (angle !== undefined && force !== undefined) out.shear = { angle, force, ...(length !== undefined ? { length } : {}) };
    return out;
  }

  private schedule(): void {
    if (!this.ready) return;
    this.wait(() => void this.preview());
  }

  private clearResult(message: string, bad: boolean): void {
    for (const c of this.cells.values()) { setText(c.next, ''); setClass(c.row, 'is-computed', false); }
    setText(this.msg, message);
    setClass(this.msg, 'is-bad', bad);
    patchKeyed(this.formulas, [], { key: () => '', create: () => el('<div></div>'), update: () => undefined });
    patchKeyed(this.warn, [], { key: () => '', create: () => el('<div></div>'), update: () => undefined });
    setDisabled(this.apply, true);
  }

  private async preview(): Promise<void> {
    const args = this.args();
    const id = this.targetId();
    const mine = ++this.seq;
    if (!Object.keys(args).length || id === null) { this.clearResult(t('Enter at least one measurement'), false); return; }
    const r = await this.ctx.store.preview<LabReply>('fabric.lab', { ...args, fabric: id });
    if (mine !== this.seq) return;
    if (!r.ok) { this.clearResult(r.message, true); return; }
    const lab = r.result;
    for (const [key, c] of this.cells) {
      setText(c.next, fmtNum(lab.physical[key]));
      setClass(c.row, 'is-computed', lab.computed.includes(key));
    }
    setText(this.msg, '');
    setClass(this.msg, 'is-bad', false);
    patchKeyed(this.formulas, lab.formulas, {
      key: (f) => f.field,
      create: () => el('<div class="ff-lab-formula"></div>'),
      update: (n, f) => setText(n, `${f.field}: ${f.expression} = ${fmtNum(f.value)} ${f.unit}${f.sources.length ? ` [${f.sources.join(', ')}]` : ''}`),
    });
    patchKeyed(this.warn, lab.warnings.map((w, i) => ({ w, i })), {
      key: (x) => String(x.i),
      create: () => el('<div class="ff-lab-warning"></div>'),
      update: (n, x) => setText(n, x.w),
    });
    setDisabled(this.apply, false);
  }

  private async commit(): Promise<void> {
    const args = this.args();
    const id = this.targetId();
    if (id === null || !Object.keys(args).length) return;
    const r = await this.ctx.store.attempt('fabric.lab', { ...args, fabric: id, apply: true });
    if (!r.ok) { this.ctx.fail(r.message); return; }
    this.ctx.select(id);
    this.ctx.info(t('Fabric Lab values applied'));
    this.close();
  }
}
