import { setClass, setText } from '../../ui/dom';
import { onLanguage, t } from '../../ui/i18n';
import { icon } from '../../ui/icons';

const open = new Set<Pop>();
let wired = false;

function wire(): void {
  if (wired) return;
  wired = true;
  document.addEventListener('pointerdown', (e) => {
    for (const p of open) if (!p.slot.contains(e.target as Node)) p.close();
  }, true);
  document.addEventListener('keydown', (e) => {
    if (e.key !== 'Escape' || open.size === 0) return;
    e.preventDefault();
    for (const p of [...open]) p.close();
  }, true);
}

export class Pop {
  readonly slot: HTMLDivElement;
  readonly button: HTMLButtonElement;
  readonly panel: HTMLDivElement;
  private readonly label: HTMLSpanElement;
  private readonly keys: Array<[HTMLElement, string]> = [];
  onOpen: (() => void) | null = null;

  constructor(parent: HTMLElement, iconName: string, private readonly english: string, private readonly caret = true) {
    wire();
    this.slot = document.createElement('div');
    this.slot.className = 'ffz-slot';
    this.button = document.createElement('button');
    this.button.type = 'button';
    this.button.className = 'ff3-btn';
    this.button.innerHTML = `${icon(iconName, 14)}<span></span>${caret ? icon('chevronDown', 10) : ''}`;
    this.label = this.button.querySelector('span:not(.icon)') as HTMLSpanElement;
    this.panel = document.createElement('div');
    this.panel.className = 'ffz-pop';
    this.panel.hidden = true;
    this.slot.append(this.button, this.panel);
    parent.appendChild(this.slot);
    this.button.addEventListener('click', (e) => {
      if (this.panel.hidden) this.show(); else this.close();
      if (e.detail > 0) this.button.blur();
    });
    this.relabel();
    onLanguage(() => this.relabel());
  }

  get isOpen(): boolean {
    return !this.panel.hidden;
  }

  show(): void {
    if (!this.panel.hidden) return;
    for (const p of [...open]) if (p !== this) p.close();
    this.panel.hidden = false;
    open.add(this);
    setClass(this.button, 'is-on', true);
    this.onOpen?.();
  }

  close(): void {
    if (this.panel.hidden) return;
    if (this.panel.contains(document.activeElement)) this.button.focus();
    this.panel.hidden = true;
    open.delete(this);
    setClass(this.button, 'is-on', false);
  }

  setLabel(english: string): void {
    this.label.dataset.en = english;
    setText(this.label, t(english));
  }

  setActive(on: boolean): void {
    setClass(this.button, 'is-on', on || this.isOpen);
  }

  text(node: HTMLElement, english: string): void {
    node.dataset.en = english;
    setText(node, t(english));
    this.keys.push([node, english]);
  }

  item(english: string, run: () => void, iconName?: string): HTMLButtonElement {
    const b = document.createElement('button');
    b.type = 'button';
    b.className = 'ffz-item';
    b.innerHTML = `${iconName ? icon(iconName, 14) : ''}<span class="ffz-grow"></span>`;
    this.text(b.querySelector('.ffz-grow') as HTMLElement, english);
    b.addEventListener('click', (e) => { run(); if (e.detail > 0) b.blur(); });
    this.panel.appendChild(b);
    return b;
  }

  separator(): void {
    const s = document.createElement('div');
    s.className = 'ffz-sep';
    this.panel.appendChild(s);
  }

  note(english: string): HTMLDivElement {
    const n = document.createElement('div');
    n.className = 'ffz-note';
    this.text(n, english);
    this.panel.appendChild(n);
    return n;
  }

  private relabel(): void {
    const en = this.label.dataset.en ?? this.english;
    this.label.dataset.en = en;
    setText(this.label, t(en));
    for (const [node, english] of this.keys) setText(node, t(english));
  }
}
