import { Dialog } from '../../ui/dialogs';
import { el, q, setClass, setHidden, setText } from '../../ui/dom';
import { t } from '../../ui/i18n';
import { icon } from '../../ui/icons';
import { browseSave, canBrowse, recall, remember, swapExt } from './files';
import type { OutputStore } from './store';
import type { FormatInfo, OptSpec } from './types';
import { bar, button, numRow, pairRow, selectRow, textRow, titled, toggleRow, type Row } from './widgets';

type Value = boolean | number | string | [number, number];

interface Built {
  info: FormatInfo;
  box: HTMLElement;
  rows: Map<string, { spec: OptSpec; row: Row<never> }>;
}

const START = /^(start|first|frame_start|start_frame)$/;
const END = /^(end|last|frame_end|end_frame)$/;

export class ExportDialog extends Dialog {
  private readonly store: OutputStore;
  private readonly notify: (text: string, kind?: 'info' | 'error') => void;
  private readonly format = selectRow('Format', []);
  private readonly path = textRow('Path');
  private readonly browse = el<HTMLButtonElement>(`<button class="cv-icon-btn" type="button">${icon('folderOpen', 16)}</button>`);
  private readonly host = el('<div class="ou-fmts"></div>');
  private readonly weldBox = el('<div class="ou-weldpick is-off" hidden><span class="ou-weldpick-title"></span><div class="ou-weldpick-list"><span class="ou-weldpick-empty"></span></div></div>');
  private readonly pickRows = new Map<number, { el: HTMLElement; input: HTMLInputElement; name: HTMLElement }>();
  private readonly chosen = new Set<number>(recall<number[]>('export.weldPanels', []));
  private readonly note = el('<p class="cv-note ou-note"></p>');
  private readonly status = el('<p class="ou-status"></p>');
  private readonly progress = bar();
  private readonly go: HTMLButtonElement;
  private readonly built = new Map<string, Built>();
  private signature = '';
  private busy = false;
  private current = '';

  constructor(store: OutputStore, notify: (text: string, kind?: 'info' | 'error') => void) {
    super('Export', 520);
    this.store = store;
    this.notify = notify;
    const pathLine = el('<div class="ou-pathline"></div>');
    pathLine.append(this.path.el, this.browse);
    titled(this.browse, 'Browse');
    setHidden(this.browse, !canBrowse());
    setText(q(this.weldBox, '.ou-weldpick-title'), t('Panels to weld'));
    setText(q(this.weldBox, '.ou-weldpick-empty'), t('No panels to list yet.'));
    this.body.append(this.format.el, pathLine, this.host, this.weldBox, this.note, this.progress.el, this.status);
    this.button('Close', 'ghost', () => this.close());
    this.go = this.button('Export', 'primary', () => { void this.run(); });
    this.format.onInput(() => this.select(this.format.get()));
    this.browse.addEventListener('click', () => { void this.pick(); });
    this.path.input.addEventListener('keydown', (e) => { if (e.key === 'Enter') { e.preventDefault(); this.go.click(); } });
    this.path.onInput(() => { if (this.current) remember(`path.${this.current}`, this.path.get()); });
  }

  private rebuild(): void {
    const formats = this.store.formats;
    const sig = JSON.stringify(formats);
    if (sig === this.signature) return;
    this.signature = sig;
    this.host.replaceChildren();
    this.built.clear();
    const select = this.format.select;
    const keep = select.value;
    const have = new Map(Array.from(select.options).map((o) => [o.value, o] as const));
    const ids = new Set(formats.map((f) => f.id));
    for (const [id, o] of have) if (!ids.has(id)) o.remove();
    formats.forEach((f, at) => {
      let o = have.get(f.id);
      if (!o) { o = document.createElement('option'); o.value = f.id; }
      if (o.textContent !== f.label) setText(o, f.label);
      if (select.options[at] !== o) select.insertBefore(o, select.options[at] ?? null);
    });
    if (keep && ids.has(keep) && select.value !== keep) select.value = keep;
    for (const f of formats) {
      const box = el('<div class="ou-fmt" hidden></div>');
      const rows = new Map<string, { spec: OptSpec; row: Row<never> }>();
      const saved = recall<Record<string, Value>>(`export.${f.id}`, {});
      for (const spec of f.opts) {
        const row = this.makeRow(spec);
        if (spec.hint) row.el.title = spec.hint;
        rows.set(spec.key, { spec, row: row as unknown as Row<never> });
        const v = saved[spec.key] ?? spec.def;
        (row as Row<Value>).set(v as Value);
        row.onInput(() => this.save(f.id));
        if (spec.key === 'weld') row.onInput(() => this.paintPick());
        box.appendChild(row.el);
      }
      this.host.appendChild(box);
      this.built.set(f.id, { info: f, box, rows });
    }
  }

  private makeRow(spec: OptSpec): Row<Value> {
    switch (spec.kind) {
      case 'boolean': return toggleRow(spec.label) as unknown as Row<Value>;
      case 'enum': return selectRow(spec.label, spec.choices) as unknown as Row<Value>;
      case 'integer': return numRow(spec.label, '', '1', spec.min, spec.max) as unknown as Row<Value>;
      case 'number': return numRow(spec.label, '', spec.max !== null && spec.max <= 10 ? '0.01' : '1', spec.min, spec.max) as unknown as Row<Value>;
      case 'pair': return pairRow(spec.label, '1') as unknown as Row<Value>;
      default: return textRow(spec.label) as unknown as Row<Value>;
    }
  }

  private values(id: string): Record<string, Value> {
    const out: Record<string, Value> = {};
    const b = this.built.get(id);
    if (!b) return out;
    for (const [key, { row }] of b.rows) out[key] = (row as unknown as Row<Value>).get();
    return out;
  }

  private save(id: string): void {
    remember(`export.${id}`, this.values(id));
  }

  private applyRange(b: Built): void {
    const r = this.store.range;
    if (!r) return;
    for (const [key, { spec, row }] of b.rows) {
      const rw = row as unknown as Row<Value>;
      if (spec.kind === 'pair' && key === 'range') rw.set([0, r.end - r.start]);
      else if (spec.kind !== 'pair' && START.test(key)) rw.set(0);
      else if (spec.kind !== 'pair' && END.test(key)) rw.set(r.end - r.start);
    }
  }

  private select(id: string): void {
    const b = this.built.get(id);
    if (!b) return;
    if (this.current && this.current !== id) {
      const prev = this.built.get(this.current);
      if (prev) {
        setHidden(prev.box, true);
        const was = this.path.get();
        const next = swapExt(was, prev.info.ext, b.info.ext);
        if (next !== was) this.path.set(next);
      }
    }
    this.current = id;
    this.format.set(id);
    setHidden(b.box, false);
    if (!this.path.get() || this.path.get() === '') this.path.set(recall<string>(`path.${id}`, `Garment.${b.info.ext}`));
    else if (!new RegExp(`\\.${b.info.ext}$`, 'i').test(this.path.get())) this.path.set(recall<string>(`path.${id}`, `Garment.${b.info.ext}`));
    this.applyRange(b);
    this.paint();
  }

  private syncPanels(): void {
    const seen = new Set<number>();
    const empty = q(this.weldBox, '.ou-weldpick-empty');
    const list = q(this.weldBox, '.ou-weldpick-list');
    for (const island of this.store.uv.islands) {
      seen.add(island.panel);
      let row = this.pickRows.get(island.panel);
      if (!row) {
        const node = el('<label class="ou-weldrow"><input type="checkbox"><span></span></label>');
        row = { el: node, input: q<HTMLInputElement>(node, 'input'), name: q(node, 'span') };
        const id = island.panel;
        row.input.addEventListener('change', () => {
          if (row?.input.checked) this.chosen.add(id); else this.chosen.delete(id);
          remember('export.weldPanels', [...this.chosen]);
        });
        this.pickRows.set(id, row);
        list.appendChild(node);
      }
      setText(row.name, island.name);
      if (row.input.checked !== this.chosen.has(island.panel)) row.input.checked = this.chosen.has(island.panel);
    }
    for (const [id, row] of [...this.pickRows]) {
      if (seen.has(id)) continue;
      row.el.remove();
      this.pickRows.delete(id);
    }
    setHidden(empty, this.pickRows.size > 0);
  }

  private paintPick(): void {
    const b = this.built.get(this.current);
    const has = this.store.formats.some((f) => f.opts.some((o) => o.key === 'weld'));
    setHidden(this.weldBox, !has);
    const weld = b ? (b.rows.get('weld')?.row as unknown as Row<string> | undefined)?.get() : undefined;
    const on = weld === 'weld_selected';
    setClass(this.weldBox, 'is-off', !on);
    for (const row of this.pickRows.values()) if (row.input.disabled !== !on) row.input.disabled = !on;
  }

  private paint(): void {
    this.paintPick();
    const have = this.store.has('export.run');
    const formats = this.store.formats.length > 0;
    const b = this.built.get(this.current);
    const range = this.store.range;
    let hint = '';
    if (!this.store.has('export.formats') || !have) hint = t('Not available in this engine build');
    else if (!formats) hint = t('No export formats are available.');
    else if (b && range && [...b.rows.values()].some((r) => START.test(r.spec.key) || END.test(r.spec.key) || r.spec.key === 'range')) hint = `${t('Timeline cache frames')} 0 - ${range.end - range.start}`;
    setText(this.note, hint);
    setHidden(this.note, hint === '');
    const off = !have || !formats || this.busy;
    if (this.go.disabled !== off) this.go.disabled = off;
    if (this.format.select.disabled !== (!formats || this.busy)) this.format.disable(!formats || this.busy);
  }

  private async pick(): Promise<void> {
    const b = this.built.get(this.current);
    if (!b) return;
    const chosen = await browseSave(b.info.ext, this.path.get(), b.info.label);
    if (chosen) { this.path.set(chosen); remember(`path.${b.info.id}`, chosen); }
  }

  async show(formatId?: string, thickness?: boolean): Promise<void> {
    await Promise.all([this.store.loadFormats(), this.store.loadRange(), this.store.loadUv()]);
    this.rebuild();
    if (thickness !== undefined) for (const b of this.built.values()) (b.rows.get('thickness')?.row as unknown as Row<boolean> | undefined)?.set(thickness);
    this.syncPanels();
    const want = formatId && this.built.has(formatId) ? formatId : this.current && this.built.has(this.current) ? this.current : this.store.formats[0]?.id ?? '';
    if (this.current && this.current !== want) {
      const prev = this.built.get(this.current);
      if (prev) setHidden(prev.box, true);
      this.current = '';
    }
    if (want) {
      if (this.current === want) { const b = this.built.get(want); if (b) this.applyRange(b); }
      else this.select(want);
    }
    setText(this.status, '');
    this.progress.show(false);
    this.paint();
    this.open(this.path.input);
  }

  private async run(): Promise<void> {
    const b = this.built.get(this.current);
    const path = this.path.get().trim();
    if (!b || this.busy) return;
    if (!path) { setText(this.status, t('Choose a path first.')); this.path.input.focus(); return; }
    this.busy = true;
    this.save(b.info.id);
    remember(`path.${b.info.id}`, path);
    this.progress.set(null);
    this.progress.show(true);
    setText(this.status, t('Exporting...'));
    this.paint();
    const opts = this.values(b.info.id);
    if (opts.animation !== true) for (const k of ['cache', 'start', 'end', 'step']) delete opts[k];
    else if (!(Number(opts.cache) >= 1)) delete opts.cache;
    const args: Record<string, unknown> = { ...opts, format: b.info.id, path };
    if (opts.weld === 'weld_selected') {
      const ids = [...this.chosen].filter((id) => this.pickRows.has(id));
      if (ids.length < 2) { this.busy = false; this.progress.show(false); this.paint(); setText(this.status, t('Pick at least two panels to weld.')); return; }
      args.panels = ids;
    }
    const r = await this.store.attempt<Record<string, unknown>>('export.run', args);
    this.busy = false;
    this.progress.show(false);
    this.paint();
    if (!r.ok) {
      setText(this.status, r.message);
      this.notify(r.message, 'error');
      return;
    }
    const out = typeof r.result?.path === 'string' ? r.result.path : path;
    const files = Array.isArray(r.result?.files) ? (r.result.files as Array<{ bytes?: unknown }>) : [];
    const sum = files.reduce((n, f) => n + (typeof f.bytes === 'number' ? f.bytes : 0), 0);
    const bytes = sum > 0 ? sum : typeof r.result?.bytes === 'number' ? r.result.bytes : null;
    const size = bytes === null ? '' : ` (${bytes >= 1048576 ? `${(bytes / 1048576).toFixed(1)} MB` : `${Math.max(1, Math.round(bytes / 1024))} KB`})`;
    setText(this.status, '');
    this.notify(`${t('Exported')} ${b.info.label}: ${out}${size}`, 'info');
    this.close();
  }

  get currentFormat(): string {
    return this.current;
  }

  q<T extends Element = HTMLElement>(sel: string): T {
    return q<T>(this.scrim, sel);
  }
}
