import { fromHtml as el, q, setText } from './dom';
import type { DockDeps, Link } from './link';

interface WindowState {
  epoch: number;
  changed: boolean;
  consent: { tool: string; asked: number } | null;
}

export class AgentLink {
  private epoch = -1;
  private readonly dialog: HTMLElement;
  private readonly detail: HTMLElement;
  private open = false;
  private busy = false;
  private current = 0;
  private answered = 0;
  private timer = 0;
  onChange: (() => void) | null = null;
  shouldWait: () => boolean = () => false;

  constructor(overlay: HTMLElement, private readonly link: Link, private readonly deps: DockDeps) {
    const t = deps.t;
    this.dialog = el(`<div class="ff-consent" role="alertdialog" aria-modal="true" aria-labelledby="ff-consent-title" hidden>
<div class="ff-consent-card"><h2 id="ff-consent-title">${t('Allow an outside agent to edit patterns?')}</h2><p class="ff-consent-detail"></p>
<div class="ff-consent-actions"><button type="button" class="ff-btn ff-consent-deny">${t('Decline')}</button><button type="button" class="ff-btn is-primary ff-consent-allow">${t('Allow')}</button></div></div>
</div>`);
    this.detail = q(this.dialog, '.ff-consent-detail');
    q(this.dialog, '.ff-consent-deny').addEventListener('click', () => void this.answer(false));
    q(this.dialog, '.ff-consent-allow').addEventListener('click', () => void this.answer(true));
    overlay.append(this.dialog);
  }

  start(): void {
    if (this.timer) return;
    void this.tick();
    this.timer = window.setInterval(() => void this.tick(), 900);
  }

  stop(): void {
    window.clearInterval(this.timer);
    this.timer = 0;
    this.dialog.remove();
  }

  private async tick(): Promise<void> {
    if (this.busy) return;
    this.busy = true;
    try {
      const response = await fetch(this.link.url(`/api/mcp/window?since=${this.epoch}`), { method: 'POST', cache: 'no-store' });
      if (!response.ok) return;
      const state = (await response.json()) as WindowState;
      if (state.changed && !this.shouldWait()) {
        const first = this.epoch === -1;
        this.epoch = state.epoch;
        if (!first) this.onChange?.();
      }
      this.show(state.consent);
    } catch {
      return;
    } finally {
      this.busy = false;
    }
  }

  private show(consent: WindowState['consent']): void {
    if (consent && consent.asked === this.answered) return;
    if (consent && !this.open) {
      this.current = consent.asked;
      setText(this.detail, `${this.deps.t('An agent connected over MCP wants to run')} ${consent.tool} ${this.deps.t('on this garment. Every change it makes can be undone. Allow outside agents for this session?')}`);
      this.dialog.hidden = false;
      this.open = true;
    } else if (!consent && this.open) {
      this.dialog.hidden = true;
      this.open = false;
    }
  }

  private async answer(allow: boolean): Promise<void> {
    this.answered = this.current;
    this.dialog.hidden = true;
    this.open = false;
    await fetch(this.link.url('/api/mcp/consent'), { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ allow }) }).catch(() => undefined);
  }
}
