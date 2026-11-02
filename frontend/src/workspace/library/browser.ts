import { confirmAction, promptName } from '../../ui/dialogs';
import { el, q, setClass, setDisabled, setHidden, setText, setTitle } from '../../ui/dom';
import { enhanceSelect } from '../../ui/dropdown';
import { icon } from '../../ui/icons';
import { tr } from './text';
import { toast } from '../../ui/toast';
import { hasCommand, onCommands, run } from '../viz/api';
import { KIND_LABEL, KINDS, cachedThumb, list, resolveThumb, roots, setRoots, type LibItem } from './api';
import { KIND_ICON } from './icons';
import { label, repaint, titled } from '../fabric/widgets';

const MIN_W = 104;
const LABEL_H = 38;
const BUFFER = 1;
const KIND_KEY = 'formfabric.library.kind';

const HTML =
  '<div class="ff-lib-row"><select class="cv-input" data-k="kind" aria-label="Kind"></select><select class="cv-input" data-k="tag" aria-label="Tag"></select></div>' +
  '<div class="ff-lib-row"><input class="cv-input" type="search" data-k="search" spellcheck="false"><button class="cv-icon-btn" type="button" data-k="refresh"></button></div>' +
  '<div class="ff-lib-roots" data-k="roots"></div>' +
  '<div class="ff-lib-grid" data-k="grid" tabindex="0" role="listbox"><div class="ff-lib-spacer" data-k="spacer"></div></div>' +
  '<p class="ff-lib-note" data-k="note" role="status"></p>' +
  '<div class="ff-lib-actions"><button class="cv-btn" type="button" data-k="insert"></button><button class="cv-btn" type="button" data-k="add"></button><button class="cv-btn cv-btn-ghost" type="button" data-k="remove"></button><button class="cv-btn cv-btn-ghost" type="button" data-k="setroots"></button></div>';

interface Cell { node: HTMLElement; img: HTMLImageElement; name: HTMLElement; kind: HTMLElement; ico: HTMLElement; at: number; x: number; y: number; key: string; src: string }

export class LibraryTab {
  readonly pane: HTMLElement;
  fromSelection: (kind: string) => Promise<Record<string, unknown> | null> = () => Promise.resolve({});
  private readonly kind: HTMLSelectElement;
  private readonly tag: HTMLSelectElement;
  private readonly search: HTMLInputElement;
  private readonly grid: HTMLElement;
  private readonly spacer: HTMLElement;
  private readonly note: HTMLElement;
  private readonly rootsEl: HTMLElement;
  private readonly insertBtn: HTMLButtonElement;
  private readonly addBtn: HTMLButtonElement;
  private readonly removeBtn: HTMLButtonElement;
  private readonly rootsBtn: HTMLButtonElement;
  private items: LibItem[] = [];
  private shown: LibItem[] = [];
  private cells: Cell[] = [];
  private selected: string | null = null;
  private cols = 1;
  private cellW = MIN_W;
  private cellH = 100;
  private queued = false;
  private loaded = false;
  private reqId = 0;
  private timer = 0;
  private rootList: string[] = [];

  constructor(parent: HTMLElement) {
    this.pane = el('<div class="ff-lib-pane" data-tab="library"></div>');
    this.pane.innerHTML = HTML;
    parent.appendChild(this.pane);
    const k = (n: string): HTMLElement => q(this.pane, `[data-k="${n}"]`);
    this.kind = k('kind') as HTMLSelectElement;
    this.tag = k('tag') as HTMLSelectElement;
    this.search = k('search') as HTMLInputElement;
    this.grid = k('grid');
    this.spacer = k('spacer');
    this.note = k('note');
    this.rootsEl = k('roots');
    this.insertBtn = k('insert') as HTMLButtonElement;
    this.addBtn = k('add') as HTMLButtonElement;
    this.removeBtn = k('remove') as HTMLButtonElement;
    this.rootsBtn = k('setroots') as HTMLButtonElement;
    const refresh = k('refresh');
    refresh.innerHTML = icon('refresh', 16);
    titled(refresh, 'Refresh');

    for (const v of ['all', ...KINDS]) this.kind.appendChild(el<HTMLOptionElement>(`<option value="${v}"></option>`));
    repaint(() => { for (const o of Array.from(this.kind.options)) setText(o, tr(KIND_LABEL[o.value] ?? o.value)); });
    const savedKind = localStorage.getItem(KIND_KEY) ?? 'all';
    this.kind.value = ['all', ...KINDS].includes(savedKind as never) ? savedKind : 'all';
    this.tag.appendChild(el<HTMLOptionElement>('<option value="">All tags</option>'));
    enhanceSelect(this.kind);
    enhanceSelect(this.tag);
    repaint(() => {
      this.search.placeholder = tr('Search the library');
      this.search.setAttribute('aria-label', tr('Search the library'));
      setText(this.tag.options[0], tr('All tags'));
    });
    label(this.insertBtn, 'Insert');
    label(this.addBtn, 'Add selected');
    label(this.removeBtn, 'Remove');
    label(this.rootsBtn, 'Roots...');

    this.kind.addEventListener('change', () => { localStorage.setItem(KIND_KEY, this.kind.value); void this.reload(); });
    this.tag.addEventListener('change', () => this.apply());
    this.search.addEventListener('input', () => { window.clearTimeout(this.timer); this.timer = window.setTimeout(() => void this.reload(), 180); });
    this.search.addEventListener('keydown', (e) => e.stopPropagation());
    refresh.addEventListener('click', () => void this.reload());
    this.grid.addEventListener('scroll', () => this.schedule(), { passive: true });
    new ResizeObserver(() => this.schedule()).observe(this.grid);
    this.grid.addEventListener('click', (e) => { const c = this.cellOf(e.target); if (c) this.select(c.key); });
    this.grid.addEventListener('dblclick', (e) => { const c = this.cellOf(e.target); if (c) void this.insert(c.key); });
    this.grid.addEventListener('keydown', (e) => this.key(e));
    this.grid.addEventListener('dragstart', (e) => {
      const c = this.cellOf(e.target);
      if (!c || !e.dataTransfer) return;
      this.select(c.key);
      e.dataTransfer.setData('application/x-ff-library', c.key);
      e.dataTransfer.setData('text/plain', c.key);
      e.dataTransfer.effectAllowed = 'copy';
    });
    this.insertBtn.addEventListener('click', () => { if (this.selected) void this.insert(this.selected); });
    this.addBtn.addEventListener('click', () => void this.add());
    this.removeBtn.addEventListener('click', () => void this.remove());
    this.rootsBtn.addEventListener('click', () => void this.editRoots());
    onCommands(() => this.gate());
    this.gate();
  }

  private cellOf(target: EventTarget | null): Cell | null {
    const n = target instanceof Element ? target.closest<HTMLElement>('.ff-lib-cell') : null;
    return n ? this.cells.find((c) => c.node === n) ?? null : null;
  }

  private gate(): void {
    const lib = hasCommand('library.list');
    const hint = lib ? '' : tr('The engine does not provide the library yet');
    setTitle(this.pane, hint);
    for (const [b, cmd] of [[this.insertBtn, 'library.insert'], [this.addBtn, 'library.add'], [this.removeBtn, 'library.remove'], [this.rootsBtn, 'library.set_roots']] as const) {
      const ok = hasCommand(cmd) && (b === this.rootsBtn || b === this.addBtn || !!this.selected);
      setDisabled(b, !ok);
      setTitle(b, hasCommand(cmd) ? '' : tr('The engine does not provide') + ' ' + cmd);
    }
    this.say(lib ? '' : hint, !lib);
    if (lib && !this.loaded) void this.reload();
  }

  private say(text: string, bad = false): void {
    setText(this.note, text);
    setClass(this.note, 'is-bad', bad);
  }

  show(): void {
    this.gate();
    if (hasCommand('library.list') && !this.loaded) void this.reload();
    this.schedule();
  }

  async reload(): Promise<void> {
    const id = ++this.reqId;
    const got = await list(this.kind.value, this.search.value.trim());
    if (id !== this.reqId) return;
    this.rootList = await roots();
    setText(this.rootsEl, this.rootList.join('  |  '));
    setTitle(this.rootsEl, this.rootList.join('\n'));
    if (got === undefined) { this.say(tr('The library could not be read'), true); return; }
    this.loaded = true;
    this.items = got;
    this.fillTags();
    this.apply();
  }

  private fillTags(): void {
    const have = new Set(Array.from(this.tag.options).map((o) => o.value));
    const fresh = new Set<string>();
    for (const it of this.items) for (const g of it.tags) if (!have.has(g)) fresh.add(g);
    for (const g of [...fresh].sort((a, b) => a.localeCompare(b))) { const o = document.createElement('option'); o.value = g; o.textContent = g; this.tag.appendChild(o); }
  }

  private apply(): void {
    const tag = this.tag.value;
    const needle = this.search.value.trim().toLowerCase();
    this.shown = this.items.filter((i) => (!tag || i.tags.includes(tag)) && (!needle || i.name.toLowerCase().includes(needle) || i.tags.some((g) => g.toLowerCase().includes(needle))));
    if (this.selected && !this.shown.some((i) => i.id === this.selected)) this.selected = null;
    this.say(this.shown.length ? `${this.shown.length} ${tr('items')}` : tr('Nothing in the library matches'));
    this.gate();
    this.schedule(true);
  }

  private schedule(force = false): void {
    if (force) this.cols = 0;
    if (this.queued) return;
    this.queued = true;
    requestAnimationFrame(() => { this.queued = false; this.paint(); });
  }

  private paint(): void {
    const w = this.grid.clientWidth, h = this.grid.clientHeight;
    if (!w || !h) return;
    const cols = Math.max(1, Math.floor(w / MIN_W));
    const cellW = Math.floor(w / cols);
    const cellH = Math.round((cellW - 8) * 0.75) + LABEL_H;
    this.cols = cols;
    this.cellW = cellW;
    this.cellH = cellH;
    const rows = Math.ceil(this.shown.length / cols);
    const total = rows * cellH;
    const sh = `${total}px`;
    if (this.spacer.style.height !== sh) this.spacer.style.height = sh;
    const first = Math.max(0, Math.floor(this.grid.scrollTop / cellH) - BUFFER);
    const last = Math.min(rows - 1, Math.ceil((this.grid.scrollTop + h) / cellH) + BUFFER);
    const need = Math.max(0, (last - first + 1) * cols);
    while (this.cells.length < Math.min(need, 400)) this.cells.push(this.makeCell());
    let used = 0;
    for (let r = first; r <= last; r++) {
      for (let c = 0; c < cols; c++) {
        const at = r * cols + c;
        if (at >= this.shown.length) break;
        const cell = this.cells[used++];
        if (!cell) break;
        this.fill(cell, this.shown[at], at, c * cellW, r * cellH);
      }
    }
    for (let i = used; i < this.cells.length; i++) if (!this.cells[i].node.hidden) this.cells[i].node.hidden = true;
  }

  private makeCell(): Cell {
    const node = el('<div class="ff-lib-cell" role="option" draggable="true" hidden><div class="ff-lib-thumb"><span class="ff-lib-ico"></span><img alt="" draggable="false" decoding="async"></div><span class="ff-lib-name"></span><span class="ff-lib-kind"></span></div>');
    this.grid.appendChild(node);
    q(node, '.ff-lib-ico').innerHTML = [...new Set([...Object.values(KIND_ICON), 'info'])].map((n) => `<span data-i="${n}" hidden>${icon(n, 22)}</span>`).join('');
    return { node, img: q(node, 'img'), name: q(node, '.ff-lib-name'), kind: q(node, '.ff-lib-kind'), ico: q(node, '.ff-lib-ico'), at: -1, x: -1, y: -1, key: '', src: '' };
  }

  private fill(c: Cell, it: LibItem, at: number, x: number, y: number): void {
    if (c.node.hidden) c.node.hidden = false;
    if (c.x !== x || c.y !== y) { c.x = x; c.y = y; c.node.style.transform = `translate(${x}px, ${y}px)`; }
    const w = `${this.cellW}px`, hh = `${this.cellH}px`;
    if (c.node.style.width !== w) c.node.style.width = w;
    if (c.node.style.height !== hh) c.node.style.height = hh;
    if (c.key !== it.id) {
      c.key = it.id;
      c.node.dataset.key = it.id;
      setText(c.name, it.name);
      setTitle(c.name, it.name);
      setText(c.kind, tr(KIND_LABEL[it.kind] ?? it.kind));
      const ico = KIND_ICON[it.kind] ?? 'info';
      if (c.ico.dataset.ico !== ico) {
        c.ico.dataset.ico = ico;
        for (const s of Array.from(c.ico.children) as HTMLElement[]) setHidden(s, s.dataset.i !== ico);
      }
    }
    if (c.src !== it.thumb) {
      c.src = it.thumb;
      const url = it.thumb ? cachedThumb(it.thumb) : undefined;
      if (url) c.img.src = url;
      else {
        c.img.removeAttribute('src');
        if (it.thumb) void resolveThumb(it.thumb).then((u) => { if (u && c.src === it.thumb) c.img.src = u; });
      }
    }
    c.at = at;
    setClass(c.node, 'is-sel', it.id === this.selected);
    c.node.setAttribute('aria-selected', String(it.id === this.selected));
  }

  private select(id: string): void {
    this.selected = id;
    for (const c of this.cells) setClass(c.node, 'is-sel', c.key === id && !c.node.hidden);
    this.gate();
  }

  private key(e: KeyboardEvent): void {
    const at = this.shown.findIndex((i) => i.id === this.selected);
    const step = e.key === 'ArrowRight' ? 1 : e.key === 'ArrowLeft' ? -1 : e.key === 'ArrowDown' ? this.cols : e.key === 'ArrowUp' ? -this.cols : 0;
    if (e.key === 'Enter' && this.selected) { e.preventDefault(); void this.insert(this.selected); return; }
    if (!step || !this.shown.length) return;
    e.preventDefault();
    const next = Math.max(0, Math.min(this.shown.length - 1, at < 0 ? 0 : at + step));
    this.select(this.shown[next].id);
    const top = Math.floor(next / this.cols) * this.cellH;
    if (top < this.grid.scrollTop) this.grid.scrollTop = top;
    else if (top + this.cellH > this.grid.scrollTop + this.grid.clientHeight) this.grid.scrollTop = top + this.cellH - this.grid.clientHeight;
  }

  async insert(id: string): Promise<void> {
    if (!hasCommand('library.insert')) return;
    const done = await run('library.insert', { id });
    if (done !== undefined) toast(`${tr('Inserted')} ${this.items.find((i) => i.id === id)?.name ?? id}`, 'info');
  }

  private async add(): Promise<void> {
    if (!hasCommand('library.add')) return;
    const kind = this.kind.value === 'all' ? 'garment' : this.kind.value;
    const name = await promptName(`${tr('Add to the library')}: ${tr(KIND_LABEL[kind] ?? kind)}`, '', 'Add');
    if (!name) return;
    const extra = await this.fromSelection(kind);
    if (extra === null) { toast(tr('Nothing is selected to add'), 'error'); return; }
    const done = await run('library.add', { kind, name, ...extra });
    if (done !== undefined) { toast(`${tr('Added')} ${name}`, 'info'); await this.reload(); }
  }

  private async remove(): Promise<void> {
    const it = this.items.find((i) => i.id === this.selected);
    if (!it || !hasCommand('library.remove')) return;
    if (!(await confirmAction('Remove from the library?', `${it.name}`, 'Remove', true))) return;
    const done = await run('library.remove', { id: it.id });
    if (done !== undefined) { this.selected = null; await this.reload(); }
  }

  private async editRoots(): Promise<void> {
    const text = await promptName(tr('Library folders (separated by ;)'), this.rootList.join('; '), 'Save');
    if (text === null) return;
    const paths = text.split(';').map((s) => s.trim()).filter(Boolean);
    if (await setRoots(paths) !== undefined) await this.reload();
  }

  get count(): number { return this.shown.length; }
}
