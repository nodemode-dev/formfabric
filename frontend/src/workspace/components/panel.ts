import { confirmAction, promptName } from '../../ui/dialogs';
import { el, patchList, q, setClass, setDisabled, setHidden, setText, setTitle } from '../../ui/dom';
import { enhanceSelect } from '../../ui/dropdown';
import { icon } from '../../ui/icons';
import { toast } from '../../ui/toast';
import { call } from '../../engine/service';
import { hasCommand, onCommands, run } from '../viz/api';
import { components, list, toCheck, type CheckResult, type CompItem, type LibItem } from '../library/api';
import { tr } from '../library/text';
import { label, repaint } from '../fabric/widgets';
import { OFFLINE, engineDown } from '../offline';
import type { PatternView } from '../pattern-view/view';
import type { DocStore } from '../doc';
import type { Selection } from '../state';
import { fmtLen } from '../state';
import { PortPicker } from './pick';

const HTML =
  '<p class="ff-lib-sub" data-k="h-new"></p>' +
  '<div class="ff-lib-row"><input class="cv-input" type="text" data-k="name" spellcheck="false"><input class="cv-input" type="number" min="0" step="0.5" value="5" data-k="tol" style="max-width:64px"></div>' +
  '<div class="ff-lib-row"><button class="cv-btn" type="button" data-k="pick"></button><button class="cv-btn cv-btn-ghost" type="button" data-k="addport"></button><button class="cv-btn" type="button" data-k="create"></button></div>' +
  '<div class="ff-lib-ports" data-k="drafts"></div>' +
  '<p class="ff-lib-note" data-k="note" role="status"></p>' +
  '<p class="ff-lib-sub" data-k="h-list"></p>' +
  '<div class="ff-lib-list" data-k="list" role="listbox"></div>' +
  '<div class="ff-lib-ports" data-k="ports"></div>' +
  '<div class="ff-lib-row"><select class="cv-input" data-k="with" aria-label="Library component"></select><button class="cv-btn cv-btn-ghost" type="button" data-k="check"></button><button class="cv-btn" type="button" data-k="swap"></button></div>' +
  '<label class="ff-lib-row"><input type="checkbox" data-k="ease"><span data-k="ease-l"></span></label>' +
  '<div class="ff-lib-checks" data-k="checks"></div>' +
  '<p class="ff-lib-note" data-k="result" role="status"></p>' +
  '<div class="ff-lib-row"><select class="cv-input" data-k="ja" aria-label="Port"></select><select class="cv-input" data-k="jb" aria-label="Port"></select><button class="cv-btn" type="button" data-k="join"></button></div>' +
  '<div class="ff-lib-row"><button class="cv-btn cv-btn-ghost" type="button" data-k="save"></button><button class="cv-btn cv-btn-ghost" type="button" data-k="remove"></button></div>';

const ROW = '<div class="ff-lib-item" tabindex="0" role="option"><div class="ff-lib-thumb"><span class="ff-lib-ico"></span></div><div class="ff-lib-item-text"><span class="ff-lib-name"></span><span class="ff-lib-kind"></span></div></div>';
const CHK = '<div class="ff-lib-chk"><span class="ff-lib-chk-n"></span><span data-k="a"></span><span data-k="b"></span><span class="ff-lib-chk-i"></span></div>';

export class ComponentsTab {
  readonly pane: HTMLElement;
  readonly picker: PortPicker;
  private readonly k: Record<string, HTMLElement>;
  private items: CompItem[] = [];
  private library: LibItem[] = [];
  private selected: string | null = null;
  private last: CheckResult | null = null;
  private loaded = false;
  private unit = (): string => 'mm';

  constructor(parent: HTMLElement, pane2d: HTMLElement, private readonly doc: DocStore, private readonly sel: Selection, view: PatternView, unit: () => string) {
    this.unit = unit;
    this.pane = el('<div class="ff-lib-pane" data-tab="components" hidden></div>');
    this.pane.innerHTML = HTML;
    parent.appendChild(this.pane);
    this.k = {};
    for (const n of this.pane.querySelectorAll<HTMLElement>('[data-k]')) this.k[n.dataset.k ?? ''] = n;
    this.picker = new PortPicker(pane2d, view, doc);
    this.picker.onChange = () => this.paintDraft();

    label(this.k['h-new'], 'New component from the selected panels');
    label(this.k['h-list'], 'Components');
    label(this.k.pick, 'Pick ports');
    label(this.k.addport, 'Add port');
    label(this.k.create, 'Create');
    label(this.k.check, 'Check');
    label(this.k.swap, 'Swap');
    label(this.k.join, 'Join ports');
    label(this.k['ease-l'], 'Sew a length mismatch with ease');
    label(this.k.save, 'Save to library');
    label(this.k.remove, 'Remove');
    repaint(() => {
      const name = this.k.name as HTMLInputElement;
      name.placeholder = tr('Component name');
      name.setAttribute('aria-label', tr('Component name'));
      const tol = this.k.tol as HTMLInputElement;
      tol.title = tr('Tolerance (mm)');
      tol.setAttribute('aria-label', tr('Tolerance (mm)'));
    });
    for (const n of ['with', 'ja', 'jb']) enhanceSelect(this.k[n] as HTMLSelectElement);
    for (const n of ['name', 'tol']) this.k[n].addEventListener('keydown', (e) => e.stopPropagation());
    this.k.pick.addEventListener('click', () => { if (this.picker.active) this.picker.stop(); else this.picker.start(); });
    this.k.addport.addEventListener('click', () => {
      const n = `Port ${this.picker.ports.length + 1}`;
      this.picker.commit(n);
    });
    this.k.create.addEventListener('click', () => void this.create());
    this.k.list.addEventListener('click', (e) => {
      const id = (e.target instanceof Element ? e.target.closest<HTMLElement>('.ff-lib-item') : null)?.dataset.key;
      if (id) this.select(id);
    });
    this.k.drafts.addEventListener('click', (e) => {
      const chip = e.target instanceof Element ? e.target.closest<HTMLElement>('.ff-lib-chip') : null;
      if (chip?.dataset.port) this.picker.removePort(chip.dataset.port);
    });
    this.k.check.addEventListener('click', () => void this.check());
    this.k.swap.addEventListener('click', () => void this.swap());
    this.k.join.addEventListener('click', () => void this.join());
    this.k.save.addEventListener('click', () => void this.saveToLibrary());
    this.k.remove.addEventListener('click', () => void this.remove());
    doc.onChange(() => { if (!this.pane.hidden) this.paintDraft(); });
    document.addEventListener('pointerup', () => { if (!this.pane.hidden) requestAnimationFrame(() => this.paintDraft()); }, true);
    this.k.with.addEventListener('change', () => { this.last = null; this.paintCheck(); });
    this.k.ease.addEventListener('change', () => this.paintDetail());
    onCommands(() => this.gate());
    this.gate();
  }

  show(): void {
    this.gate();
    void this.refresh();
    this.paintDraft();
  }

  hide(): void {
    this.picker.stop();
  }

  fromSelection(): string {
    return this.selected ?? 'selection';
  }

  private chosen(): CompItem | undefined {
    return this.items.find((i) => i.id === this.selected);
  }

  private gate(): void {
    const has = hasCommand('component.list');
    const hint = has ? '' : tr('The engine does not provide components yet');
    if (!has) this.say(hint, false);
    this.paintDraft();
    this.paintDetail();
    if (has && !this.loaded && !this.pane.hidden) void this.refresh();
  }

  private say(text: string, bad: boolean): void {
    setText(this.k.note, text);
    setClass(this.k.note, 'is-bad', bad);
  }

  async refresh(): Promise<void> {
    const got = await components();
    if (got === undefined) { if (!this.loaded) this.say(tr(engineDown() ? OFFLINE : 'The request failed'), true); return; }
    this.loaded = true;
    this.items = got;
    if (this.selected && !got.some((i) => i.id === this.selected)) this.selected = null;
    if (!this.selected && got[0]) this.selected = got[0].id;
    const lib = await list('component', '');
    this.library = lib ?? [];
    this.fillWith();
    this.paintList();
    this.paintDetail();
  }

  private fillWith(): void {
    const sel = this.k.with as HTMLSelectElement;
    const keep = sel.value;
    const want = this.library.map((i) => i.id);
    const have = Array.from(sel.options).map((o) => o.value);
    if (want.join('\n') === have.join('\n')) return;
    for (const o of Array.from(sel.options)) o.remove();
    for (const i of this.library) { const o = document.createElement('option'); o.value = i.id; o.textContent = i.name; sel.appendChild(o); }
    if (want.includes(keep)) sel.value = keep;
  }

  private paintList(): void {
    patchList(this.k.list, this.items, {
      key: (i) => i.id,
      create: () => { const n = el(ROW); q(n, '.ff-lib-ico').innerHTML = icon('libComponent', 20); return n; },
      update: (n, i) => {
        setText(q(n, '.ff-lib-name'), i.name);
        setText(q(n, '.ff-lib-kind'), `${i.panels.length} ${tr('panels')}, ${i.ports.length} ${tr('ports')}`);
        setClass(n, 'is-sel', i.id === this.selected);
      },
    });
  }

  private select(id: string): void {
    this.selected = id;
    this.last = null;
    for (const n of Array.from(this.k.list.children) as HTMLElement[]) setClass(n, 'is-sel', n.dataset.key === id);
    this.paintDetail();
    this.paintCheck();
  }

  private portOptions(): Array<[string, string]> {
    const out: Array<[string, string]> = [];
    for (const c of this.items) for (const p of c.ports) out.push([`${c.id}:${p.name}`, `${c.name}: ${p.name}`]);
    return out;
  }

  private paintDetail(): void {
    const c = this.chosen();
    const u = this.unit();
    patchList(this.k.ports, c?.ports ?? [], {
      key: (p) => p.name,
      create: () => el('<span class="ff-lib-chip"><span class="ff-lib-icoport"></span><span class="ff-lib-pn"></span></span>'),
      update: (n, p) => {
        if (!n.firstElementChild?.innerHTML) q(n, '.ff-lib-icoport').innerHTML = icon('libPort', 12);
        setText(q(n, '.ff-lib-pn'), p.length !== null ? `${p.name}  ${fmtLen(p.length, u)}` : p.name);
        setClass(n, 'is-bad', !p.valid);
      },
    });
    const opts = this.portOptions();
    for (const key of ['ja', 'jb']) {
      const sel = this.k[key] as HTMLSelectElement;
      const want = opts.map((o) => o[0]);
      const have = Array.from(sel.options).map((o) => o.value);
      if (want.join('\n') !== have.join('\n')) {
        const keep = sel.value;
        for (const o of Array.from(sel.options)) o.remove();
        for (const [v, text] of opts) { const o = document.createElement('option'); o.value = v; o.textContent = text; sel.appendChild(o); }
        sel.value = want.includes(keep) ? keep : (want[key === 'jb' ? 1 : 0] ?? '');
      }
    }
    const cmd = (id: string): boolean => hasCommand(id);
    const hasWith = !!(this.k.with as HTMLSelectElement).value;
    this.gateButton(this.k.check, 'component.check', !!c && hasWith);
    this.gateButton(this.k.swap, 'component.swap', !!c && hasWith && !(this.last?.refused && !(this.k.ease as HTMLInputElement).checked));
    this.gateButton(this.k.join, 'component.join', opts.length >= 2 && (this.k.ja as HTMLSelectElement).value !== (this.k.jb as HTMLSelectElement).value);
    this.gateButton(this.k.save, 'component.save', !!c);
    this.gateButton(this.k.remove, 'component.remove', !!c);
    setHidden(this.k.with.parentElement as HTMLElement, !cmd('component.swap') && !cmd('component.check'));
  }

  private gateButton(b: HTMLElement, command: string, ok: boolean): void {
    const present = hasCommand(command);
    setDisabled(b as HTMLButtonElement, !present || !ok);
    setTitle(b, present ? '' : `${tr('The engine does not provide')} ${command}`);
  }

  private paintDraft(): void {
    const p = this.picker;
    const u = this.unit();
    setClass(this.k.pick, 'is-active', p.active);
    const panels = this.sel.list();
    const hasName = true;
    this.gateButton(this.k.create, 'component.create', panels.length > 0 && p.ports.length > 0 && hasName);
    this.gateButton(this.k.pick, 'component.create', panels.length > 0);
    setDisabled(this.k.addport as HTMLButtonElement, !p.current.length);
    const drafts = p.ports.map((x) => ({ name: x.name, text: `${x.name}  ${fmtLen(p.lengthOf(x.spans), u)}` }));
    patchList(this.k.drafts, drafts, {
      key: (d) => d.name,
      create: (d) => { const n = el('<span class="ff-lib-chip" title="Remove"></span>'); n.dataset.port = d.name; return n; },
      update: (n, d) => setText(n, d.text),
    });
    if (hasCommand('component.create')) {
      if (p.active) this.say(`${tr('Click panel edges to pick a port')} (${p.current.length} ${tr('edges')}, ${fmtLen(p.lengthOf(p.current), u)}). ${tr('Add port keeps them; Escape stops.')}`, false);
      else if (!panels.length) this.say(tr('Select panels to make a component'), false);
      else this.say(`${panels.length} ${tr('panels selected')}`, false);
    }
  }

  private async create(): Promise<void> {
    const panels = this.sel.list();
    const p = this.picker;
    if (!panels.length || !p.ports.length) return;
    const name = (this.k.name as HTMLInputElement).value.trim() || `Component ${this.items.length + 1}`;
    const tolerance = Number((this.k.tol as HTMLInputElement).value) || 0;
    const ports = p.ports.map((x) => ({ name: x.name, spans: x.spans, tolerance }));
    const done = await run<unknown>('component.create', { name, panels, ports });
    if (done === undefined) return;
    toast(`${tr('Created component')} ${name}`, 'info');
    (this.k.name as HTMLInputElement).value = '';
    p.stop();
    p.reset();
    await this.refresh();
  }

  private paintCheck(): void {
    const r = this.last;
    const u = this.unit();
    const rows = r?.rows ?? [];
    patchList(this.k.checks, rows, {
      key: (x) => x.name || `row${rows.indexOf(x)}`,
      create: () => { const n = el(CHK); return n; },
      update: (n, x) => {
        setText(q(n, '.ff-lib-chk-n'), x.name);
        setText(q(n, '[data-k="a"]'), x.a !== null ? fmtLen(x.a, u, false) : '');
        setText(q(n, '[data-k="b"]'), x.b !== null ? fmtLen(x.b, u, false) : '');
        const mark = q(n, '.ff-lib-chk-i');
        const ico = x.ok ? 'check' : 'warning';
        if (mark.dataset.ico !== ico) { mark.dataset.ico = ico; mark.innerHTML = icon(ico, 12); }
        setClass(n, 'is-bad', !x.ok);
        setTitle(n, x.note);
      },
    });
    if (!r) { setText(this.k.result, ''); setClass(this.k.result, 'is-bad', false); return; }
    const bits: string[] = [r.ok ? tr('The ports match') : tr('The ports do not match')];
    if (r.mismatches.length) bits.push(`${tr('Mismatches')}: ${r.mismatches.join(', ')}`);
    if (r.eased) bits.push(tr('Sewn with ease'));
    if (r.refused) bits.push(tr('The swap was refused'));
    setText(this.k.result, bits.join('. '));
    setClass(this.k.result, 'is-bad', !r.ok);
  }

  private async check(): Promise<void> {
    const c = this.chosen();
    const w = (this.k.with as HTMLSelectElement).value;
    if (!c || !w) return;
    const r = await run<unknown>('component.check', { id: c.id, with: w });
    if (r === undefined) return;
    this.last = toCheck(r);
    this.paintCheck();
    this.paintDetail();
  }

  private async swap(): Promise<void> {
    const c = this.chosen();
    const w = (this.k.with as HTMLSelectElement).value;
    if (!c || !w) return;
    const ease = (this.k.ease as HTMLInputElement).checked;
    let r: unknown;
    try {
      r = await call<unknown>('component.swap', { id: c.id, with: w, ...(ease ? { ease: true } : {}) });
    } catch (err) {
      const checked = await run<unknown>('component.check', { id: c.id, with: w });
      this.last = checked !== undefined ? { ...toCheck(checked), refused: true } : null;
      this.paintCheck();
      this.paintDetail();
      toast(`${tr('The swap was refused')}: ${err instanceof Error ? err.message : String(err)}`, 'error');
      return;
    }
    const res = toCheck(r);
    this.last = { ...res, eased: ease && !res.ok ? true : res.eased };
    this.paintCheck();
    toast(tr('Component swapped'), 'info');
    await this.refresh();
  }

  private async join(): Promise<void> {
    const a = (this.k.ja as HTMLSelectElement).value, b = (this.k.jb as HTMLSelectElement).value;
    if (!a || !b || a === b) return;
    if (await run('component.join', { a_port: a, b_port: b }) !== undefined) toast(tr('Ports joined'), 'info');
  }

  private async saveToLibrary(): Promise<void> {
    const c = this.chosen();
    if (!c) return;
    const name = await promptName(tr('Library name'), c.name, 'Save');
    if (name && await run('component.save', { id: c.id, name }) !== undefined) { toast(`${tr('Saved')} ${name}`, 'info'); await this.refresh(); }
  }

  private async remove(): Promise<void> {
    const c = this.chosen();
    if (!c || !(await confirmAction('Remove the component?', c.name, 'Remove', true))) return;
    if (await run('component.remove', { id: c.id }) !== undefined) await this.refresh();
  }
}
