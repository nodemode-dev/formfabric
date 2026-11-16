import { el, q, setClass, setDisabled, setText, setTitle } from '../../ui/dom';
import { icon } from '../../ui/icons';
import { toast } from '../../ui/toast';
import { patchList } from '../../ui/dom';
import { hasCommand, onCommands, quiet, run } from '../viz/api';
import { snapshots, thumbFromView, type SnapItem } from '../library/api';
import { tr } from '../library/text';
import { label, repaint, titled } from '../fabric/widgets';
import { OFFLINE, engineDown } from '../offline';

const HTML =
  '<div class="ff-lib-row"><input class="cv-input" type="text" data-k="name" spellcheck="false"><button class="cv-btn" type="button" data-k="save"></button></div>' +
  '<p class="ff-lib-note" data-k="note" role="status"></p>' +
  '<div class="ff-lib-list" data-k="list" role="listbox"></div>';

const ROW = '<div class="ff-lib-item" tabindex="0" role="option"><div class="ff-lib-thumb"><span class="ff-lib-ico"></span><img alt="" decoding="async"></div><div class="ff-lib-item-text"><span class="ff-lib-name"></span><span class="ff-lib-kind"></span></div><button class="cv-icon-btn" type="button" data-a="compare"></button><button class="cv-icon-btn" type="button" data-a="restore"></button><button class="cv-icon-btn" type="button" data-a="remove"></button></div>';

export class SnapshotsTab {
  readonly pane: HTMLElement;
  private readonly name: HTMLInputElement;
  private readonly save: HTMLButtonElement;
  private readonly list: HTMLElement;
  private readonly note: HTMLElement;
  private items: SnapItem[] = [];
  private selected: string | null = null;
  private comparing: string | null = null;
  private busy = false;
  private loaded = false;

  constructor(parent: HTMLElement) {
    this.pane = el('<div class="ff-lib-pane" data-tab="snapshots" hidden></div>');
    this.pane.innerHTML = HTML;
    parent.appendChild(this.pane);
    this.name = q(this.pane, '[data-k="name"]');
    this.save = q(this.pane, '[data-k="save"]');
    this.list = q(this.pane, '[data-k="list"]');
    this.note = q(this.pane, '[data-k="note"]');
    label(this.save, 'Save snapshot');
    repaint(() => { this.name.placeholder = tr('Snapshot name'); this.name.setAttribute('aria-label', tr('Snapshot name')); });
    this.name.addEventListener('keydown', (e) => { e.stopPropagation(); if (e.key === 'Enter') void this.saveNow(); });
    this.save.addEventListener('click', () => void this.saveNow());
    this.list.addEventListener('click', (e) => this.click(e));
    this.list.addEventListener('keydown', (e) => {
      if (e.key !== 'Enter' && e.key !== ' ') return;
      const id = (e.target instanceof Element ? e.target.closest<HTMLElement>('.ff-lib-item') : null)?.dataset.key;
      if (id && e.target instanceof HTMLElement && e.target.classList.contains('ff-lib-item')) { e.preventDefault(); this.select(id); }
    });
    onCommands(() => this.gate());
    this.gate();
  }

  private gate(): void {
    const ok = hasCommand('snapshot.save');
    setDisabled(this.save, !ok || this.busy);
    setTitle(this.save, ok ? '' : `${tr('The engine does not provide')} snapshot.save`);
    setText(this.note, hasCommand('snapshot.list') ? this.note.textContent : tr('The engine does not provide snapshots yet'));
    if (hasCommand('snapshot.list') && !this.loaded && !this.pane.hidden) void this.refresh();
  }

  show(): void {
    this.gate();
    void this.refresh();
  }

  async refresh(): Promise<void> {
    const got = await snapshots();
    if (got === undefined) { if (!this.loaded) setText(this.note, tr(engineDown() ? OFFLINE : 'The request failed')); return; }
    this.loaded = true;
    this.items = got.sort((a, b) => b.at - a.at);
    if (this.selected && !this.items.some((i) => i.id === this.selected)) this.selected = null;
    setText(this.note, this.items.length ? `${this.items.length} ${tr('snapshots')}` : tr('No snapshots yet'));
    this.paint();
  }

  private paint(): void {
    patchList(this.list, this.items, {
      key: (i) => i.id,
      create: () => {
        const n = el(ROW);
        for (const [a, ico, key] of [['compare', 'libCompare', 'Compare with the current state'], ['restore', 'libRestore', 'Restore'], ['remove', 'delete', 'Remove']] as const) {
          const b = q(n, `[data-a="${a}"]`);
          b.innerHTML = icon(ico, 14);
          titled(b, key);
        }
        q(n, '.ff-lib-ico').innerHTML = icon('libSnapshot', 20);
        return n;
      },
      update: (n, i) => {
        setText(q(n, '.ff-lib-name'), i.name);
        setText(q(n, '.ff-lib-kind'), `${i.at ? new Date(i.at).toLocaleString() : ''}${i.count !== null ? `  ${i.count} ${tr('panels')}` : ''}`);
        const img = q<HTMLImageElement>(n, 'img');
        if (i.thumb) { if (img.getAttribute('src') !== i.thumb) img.src = i.thumb; } else if (img.hasAttribute('src')) img.removeAttribute('src');
        setClass(n, 'is-sel', i.id === this.selected);
        setClass(q(n, '[data-a="compare"]'), 'is-active', i.id === this.comparing);
        setDisabled(q<HTMLButtonElement>(n, '[data-a="restore"]'), !hasCommand('snapshot.restore'));
        setDisabled(q<HTMLButtonElement>(n, '[data-a="remove"]'), !hasCommand('snapshot.remove'));
        setDisabled(q<HTMLButtonElement>(n, '[data-a="compare"]'), !hasCommand('compare.set'));
      },
    });
  }

  private select(id: string): void {
    this.selected = id;
    for (const n of Array.from(this.list.children) as HTMLElement[]) setClass(n, 'is-sel', n.dataset.key === id);
  }

  private click(e: MouseEvent): void {
    const target = e.target instanceof Element ? e.target : null;
    const row = target?.closest<HTMLElement>('.ff-lib-item');
    const id = row?.dataset.key;
    const item = this.items.find((i) => i.id === id);
    if (!row || !item) return;
    this.select(item.id);
    const act = target?.closest<HTMLElement>('[data-a]')?.dataset.a;
    if (act === 'restore') void this.restore(item);
    else if (act === 'remove') void this.remove(item);
    else if (act === 'compare') void this.compare(item);
  }

  private nextName(): string {
    const used = new Set(this.items.map((i) => i.name));
    for (let n = this.items.length + 1; ; n++) if (!used.has(`Snapshot ${n}`)) return `Snapshot ${n}`;
  }

  async saveNow(): Promise<void> {
    if (this.busy || !hasCommand('snapshot.save')) return;
    this.busy = true;
    this.gate();
    try {
      const name = this.name.value.trim() || this.nextName();
      const thumbnail = await thumbFromView(240, 180);
      const done = await run('snapshot.save', { name, ...(thumbnail ? { thumbnail } : {}) });
      if (done === undefined) return;
      if (hasCommand('compare.snapshot')) await quiet('compare.snapshot', { name: `snap:${name}` });
      this.name.value = '';
      toast(`${tr('Saved snapshot')} ${name}`, 'info');
      await this.refresh();
    } finally {
      this.busy = false;
      this.gate();
    }
  }

  private async restore(i: SnapItem): Promise<void> {
    if (await run('snapshot.restore', { id: i.id }) !== undefined) toast(`${tr('Restored')} ${i.name}`, 'info');
  }

  private async remove(i: SnapItem): Promise<void> {
    if (await run('snapshot.remove', { id: i.id }) === undefined) return;
    if (this.comparing === i.id) { this.comparing = null; await quiet('compare.clear', {}); }
    await this.refresh();
  }

  private async compare(i: SnapItem): Promise<void> {
    if (this.comparing === i.id) {
      this.comparing = null;
      await quiet('compare.clear', {});
    } else if (await run('compare.set', { a: 'live', b: `snap:${i.name}` }) !== undefined) this.comparing = i.id;
    this.paint();
  }

  get count(): number { return this.items.length; }
}
