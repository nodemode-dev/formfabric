import { el, patchList, q, setClass, setText } from '../../ui/dom';
import { onLanguage, t } from '../../ui/i18n';
import type { DocStore } from '../doc';
import type { Selection } from '../state';
import { mmText } from './render-sew';
import type { Seam } from './sew-types';

const KEY = 'formfabric.seamcheck';

interface Row { seam: Seam; mismatch: number }

export class SeamCheck {
  readonly node: HTMLElement;
  private on: boolean;
  private readonly list: HTMLElement;
  private readonly count: HTMLElement;
  private readonly note: HTMLElement;
  private readonly title: HTMLElement;

  constructor(host: HTMLElement, private readonly doc: DocStore, private readonly sel: Selection, private readonly picked: () => void, private readonly changed: (on: boolean) => void) {
    this.on = localStorage.getItem(KEY) === '1';
    this.node = el('<div class="ff-sewcheck" data-k="sewcheck"><header class="ff-sewcheck-head"><span data-k="title"></span><output data-k="count"></output></header><div class="ff-sewcheck-list" data-k="list"></div><p class="ff-sewcheck-note" data-k="note"></p></div>');
    host.appendChild(this.node);
    this.list = q(this.node, '[data-k="list"]');
    this.count = q(this.node, '[data-k="count"]');
    this.note = q(this.node, '[data-k="note"]');
    this.title = q(this.node, '[data-k="title"]');
    this.list.addEventListener('click', (e) => {
      const row = e.target instanceof Element ? e.target.closest<HTMLElement>('.ff-sewcheck-row') : null;
      const id = Number(row?.dataset.key);
      if (!row || !Number.isFinite(id)) return;
      this.sel.seam = id;
      this.sel.tack = null;
      this.picked();
    });
    onLanguage(() => this.update());
    this.update();
  }

  get open(): boolean { return this.on; }

  toggle(): void {
    this.on = !this.on;
    localStorage.setItem(KEY, this.on ? '1' : '0');
    this.update();
    this.changed(this.on);
  }

  update(): void {
    setClass(this.node, 'is-open', this.on);
    setText(this.title, t('Seam Check'));
    const rows: Row[] = [];
    for (const s of this.doc.sew.seams) {
      const m = this.doc.sew.bad.get(s.id);
      if (m !== undefined) rows.push({ seam: s, mismatch: m });
    }
    setText(this.count, String(rows.length));
    setText(this.note, this.doc.sew.seams.length === 0 ? t('No seams') : rows.length === 0 ? t('All seams match') : '');
    const name = (id: number | undefined): string => (id === undefined ? '-' : this.doc.panel(id)?.name ?? String(id));
    patchList(this.list, rows, {
      key: (r) => String(r.seam.id),
      create: () => el('<button class="ff-sewcheck-row" type="button"><span data-k="n"></span><output data-k="m"></output></button>'),
      update: (node, r) => {
        setText(q(node, '[data-k="n"]'), `#${r.seam.id}  ${name(r.seam.a[0]?.panel)} / ${name(r.seam.b[0]?.panel)}`);
        setText(q(node, '[data-k="m"]'), mmText(r.mismatch));
        setClass(node, 'is-on', this.sel.seam === r.seam.id);
      },
    });
  }
}
