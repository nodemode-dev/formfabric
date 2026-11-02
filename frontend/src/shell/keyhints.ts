import { SCOPE_LABEL, formatKeys, scopeOf, type Command, type Commands, type Scope } from '../ui/commands';
import { anyDialogOpen } from '../ui/dialogs';
import { el, escapeHtml, isMac, q, setHidden, setHtml, setText } from '../ui/dom';
import { language, onLanguage, t } from '../ui/i18n';
import { loadPrefs } from '../ui/prefs';
import { tr } from '../workspace/library/text';

const HOLD_MS = 600;
const PER_GROUP = 14;

const HTML = '<div class="ff-keyhints-card"><div class="ff-keyhints-head"><span data-k="title"></span><span data-k="hint"></span></div><div class="ff-keyhints-cols" data-k="cols"></div></div>';

const heading = (group: string | undefined): string => {
  const g = (group ?? 'general').toLowerCase();
  return g.charAt(0).toUpperCase() + g.slice(1);
};

export class KeyHints {
  private readonly root: HTMLElement;
  private readonly cols: HTMLElement;
  private timer = 0;
  private holding = false;
  sticky = false;

  constructor(private readonly commands: Commands) {
    this.root = el('<div class="ff-keyhints" role="dialog" aria-label="Key hints" hidden></div>');
    this.root.innerHTML = HTML;
    (document.getElementById('overlay-root') ?? document.body).appendChild(this.root);
    this.cols = q(this.root, '[data-k="cols"]');
    const mod = isMac ? 'Meta' : 'Control';
    window.addEventListener('keydown', (e) => {
      if (e.key === mod && !e.repeat && !e.shiftKey && !e.altKey) { this.arm(); return; }
      if (e.key === 'Escape' && this.sticky) { this.setSticky(false); return; }
      this.cancel();
    }, true);
    window.addEventListener('keyup', (e) => { if (e.key === mod) this.cancel(); }, true);
    window.addEventListener('blur', () => this.cancel());
    window.addEventListener('pointerdown', () => this.cancel(), true);
    onLanguage(() => { if (!this.root.hidden) this.fill(); });
  }

  private arm(): void {
    this.cancel();
    if (!loadPrefs().keyHintsOnHold || anyDialogOpen()) return;
    this.timer = window.setTimeout(() => { this.timer = 0; this.holding = true; this.fill(); this.sync(); }, HOLD_MS);
  }

  private cancel(): void {
    if (this.timer) { window.clearTimeout(this.timer); this.timer = 0; }
    if (this.holding) { this.holding = false; this.sync(); }
  }

  setSticky(on: boolean): void {
    this.sticky = on;
    if (on) this.fill();
    this.sync();
  }

  private sync(): void { setHidden(this.root, !(this.holding || this.sticky)); }

  get visible(): boolean { return !this.root.hidden; }

  private fill(): void {
    const scope: Scope = this.commands.scopeNow();
    const tool = document.querySelector<HTMLElement>('.ff-tool.is-on');
    const toolKey = scope === '3d' ? '' : (tool?.querySelector('kbd')?.textContent ?? '').replace('⇧', 'Shift+');
    const toolName = scope === '3d' ? '' : (tool?.dataset.label ?? '');
    const all = this.commands.all().filter((c) => (c.keys?.length ?? 0) > 0 && this.commands.label(c) && (scopeOf(c) === 'global' || scopeOf(c) === scope));
    const groups = new Map<string, Command[]>();
    for (const c of all) {
      const name = scopeOf(c) === 'global' && (c.group ?? '').toLowerCase() === 'window' ? 'App' : heading(c.group);
      const list = groups.get(name) ?? [];
      list.push(c);
      groups.set(name, list);
    }
    const key = `${language()}|${scope}|${toolKey}|${all.map((c) => c.id + (c.keys ?? []).join(',')).join(';')}`;
    setText(q(this.root, '[data-k="title"]'), `${tr('Key hints')}: ${t(SCOPE_LABEL[scope])}${toolName ? `, ${tr('tool')} ${t(toolName)}` : ''}`);
    setText(q(this.root, '[data-k="hint"]'), tr('Release to close'));
    const live = (c: Command): boolean => !!toolKey && (c.keys ?? []).some((k) => k.replace('Mod+', '') === toolKey || k === toolKey);
    const html = [...groups.entries()].map(([name, cmds]) => {
      const rows = cmds.slice(0, PER_GROUP).map((c) => `<div class="ff-keyhints-row${live(c) ? ' is-live' : ''}"><span>${escapeHtml(t(this.commands.label(c)))}</span><kbd>${escapeHtml(formatKeys((c.keys ?? [])[0]))}</kbd></div>`).join('');
      return `<div class="ff-keyhints-grp"><p class="ff-keyhints-gh">${escapeHtml(t(name))}</p>${rows}</div>`;
    }).join('');
    setHtml(this.cols, html, key);
  }
}
