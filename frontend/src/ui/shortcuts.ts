import { comboOf, formatKeys, type Commands } from './commands';
import { Dialog } from './dialogs';
import { el, patchList, q, setClass, setText } from './dom';
import { t } from './i18n';
import { loadPrefs, updatePrefs } from './prefs';

export function applyShortcuts(commands: Commands): void {
  for (const [id, keys] of Object.entries(loadPrefs().shortcuts)) if (commands.get(id)) commands.setKeys(id, keys);
}

export class ShortcutsDialog extends Dialog {
  private readonly search = el<HTMLInputElement>('<input class="cv-input" type="search" placeholder="Search commands" aria-label="Search shortcuts" spellcheck="false">');
  private readonly list = el('<div class="cv-list cv-keys-list" aria-label="Keyboard shortcuts"></div>');
  private readonly note = el('<p class="cv-note" role="status"></p>');
  private capturing: string | null = null;
  private pending: { id: string; combo: string } | null = null;

  constructor(private readonly commands: Commands) {
    super('Keyboard Shortcuts', 660);
    this.body.append(this.search, this.note, this.list);
    this.search.addEventListener('input', () => this.render());
    this.search.addEventListener('keydown', (e) => e.stopPropagation());
    this.list.addEventListener('click', (e) => {
      const target = e.target instanceof Element ? e.target : null;
      const row = target?.closest<HTMLElement>('.cv-keys-row');
      const id = row?.dataset.key;
      if (!id) return;
      if (target?.closest('[data-clear]')) { this.assign(id, []); return; }
      this.capturing = id;
      this.pending = null;
      this.say(t('Press the new shortcut, or Escape to cancel.'));
      this.render();
    });
    this.button('Reset All', 'ghost', () => { this.commands.resetKeys(); this.save(); this.say(''); this.render(); });
    this.button('Close', 'primary', () => this.close());
    window.addEventListener('keydown', (e) => this.capture(e), true);
  }

  show(): void {
    this.capturing = null;
    this.pending = null;
    this.say('');
    this.search.value = '';
    this.render();
    this.open(this.search);
  }

  private save(): void { updatePrefs({ shortcuts: this.commands.overrides() }); }

  private say(text: string): void { setText(this.note, text); }

  private assign(id: string, keys: string[]): void {
    this.commands.setKeys(id, keys);
    this.save();
    this.capturing = null;
    this.pending = null;
    this.say('');
    this.render();
  }

  private capture(e: KeyboardEvent): void {
    if (!this.isOpen || !this.capturing) return;
    if (['Shift', 'Control', 'Alt', 'Meta'].includes(e.key)) return;
    e.preventDefault();
    e.stopPropagation();
    if (e.key === 'Escape') { this.capturing = null; this.pending = null; this.say(''); this.render(); return; }
    const combo = comboOf(e);
    const had = this.commands.forCombo(combo);
    if (had && had.id !== this.capturing && !(this.pending && this.pending.combo === combo)) {
      this.pending = { id: this.capturing, combo };
      this.say(`${combo.replace(/^Mod/, 'Ctrl')} ${t('is used by')} ${t(this.commands.label(had))}. ${t('Press it again to move it here.')}`);
      return;
    }
    this.assign(this.capturing, [combo]);
  }

  private render(): void {
    const needle = this.search.value.trim().toLowerCase();
    const rows = this.commands.all().filter((c) => { const label = t(this.commands.label(c)); return label && c.id !== 'app.none' && (!needle || label.toLowerCase().includes(needle) || c.id.includes(needle)); });
    patchList(this.list, rows, {
      key: (c) => c.id,
      create: (c) => {
        const n = el<HTMLElement>('<div class="cv-list-row cv-keys-row"><span class="cv-list-name"></span><span class="cv-list-meta"></span><button class="cv-btn cv-btn-ghost" type="button" data-clear>Clear</button></div>');
        this.fill(n, c.id);
        return n;
      },
      update: (n, c) => this.fill(n, c.id),
    });
  }

  private fill(n: HTMLElement, id: string): void {
    const c = this.commands.get(id);
    if (!c) return;
    n.dataset.key = id;
    setText(q(n, '.cv-list-name'), t(this.commands.label(c)));
    const keys = this.capturing === id ? t('Press keys...') : (c.keys ?? []).map(formatKeys).join('  ') || t('None');
    setText(q(n, '.cv-list-meta'), keys);
    setClass(n, 'is-selected', this.capturing === id);
  }
}
