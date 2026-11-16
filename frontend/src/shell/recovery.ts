import '../workspace/library/library.css';
import { Dialog } from '../ui/dialogs';
import { el, q, reserve, setHidden, setText } from '../ui/dom';
import { t } from '../ui/i18n';
import { toast } from '../ui/toast';
import { recoveryAct, recoveryInfo, type Recovery } from '../workspace/library/api';
import { tr } from '../workspace/library/text';

class RecoveryDialog extends Dialog {
  private readonly meta = el('<p class="ff-rec-meta" data-k="meta"></p>');
  private readonly say = el('<p class="ff-lib-note" role="status" data-k="say"></p>');
  private restoreBtn: HTMLButtonElement;
  private discardBtn: HTMLButtonElement;
  private reported = false;

  constructor(private readonly done: (path: string | null) => void) {
    super('Recover your work', 420);
    this.body.append(el('<p class="ff-rec-meta" data-k="lead"></p>'), this.meta, this.say);
    setText(q(this.body, '[data-k="lead"]'), tr('FormFabric did not close normally. The last autosave and the commands after it can be restored.'));
    this.discardBtn = this.button('Discard', 'ghost', () => void this.act('discard'));
    this.restoreBtn = this.button('Restore', 'primary', () => { if (this.reported) this.close(); else void this.act('restore'); });
    this.scrim.dataset.k = 'recovery';
  }

  show(info: Recovery): void {
    const when = info.at ? new Date(info.at).toLocaleString() : tr('unknown time');
    setText(this.meta, `${tr('Saved')} ${when}. ${info.commands} ${tr('commands logged since the autosave')}.`);
    this.open(this.restoreBtn);
  }

  private async act(action: 'restore' | 'discard'): Promise<void> {
    this.restoreBtn.disabled = true;
    this.discardBtn.disabled = true;
    const got = await recoveryAct(action);
    this.restoreBtn.disabled = false;
    this.discardBtn.disabled = false;
    if (!got) { setText(this.say, tr('The request failed')); toast(tr('The request failed'), 'error'); return; }
    if (action !== 'restore') { this.close(); return; }
    const r = report(got);
    if (r.missing || r.full) this.showReport(r);
    else { this.close(); toast(`${t('Your work was restored')}. ${r.back}`, 'info'); }
    document.dispatchEvent(new CustomEvent('ff:recovered'));
    this.done(typeof got.path === 'string' && got.path ? got.path : null);
  }

  private showReport(r: Report): void {
    this.reported = true;
    const frame = q<HTMLElement>(this.scrim, '.cv-dialog'), before = frame.offsetHeight;
    reserve(this.body, { minHeight: parseFloat(getComputedStyle(this.body).height) || 0 });
    setText(q(this.body, '[data-k="lead"]'), r.back);
    setText(this.meta, r.missing);
    setText(this.say, r.full);
    setHidden(this.discardBtn, true);
    setText(this.restoreBtn, t('Close'));
    this.restoreBtn.dataset.en = 'Close';
    this.restoreBtn.dataset.shown = this.restoreBtn.textContent ?? '';
    const grew = frame.offsetHeight - before;
    if (grew > 0) frame.style.marginTop = `${grew}px`;
    this.restoreBtn.focus();
  }
}

interface Report { back: string; missing: string; full: string }

const count = (v: unknown): number => (typeof v === 'number' && Number.isFinite(v) ? v : 0);

function reason(text: string): string {
  if (text.startsWith('it took the positions of the running simulation')) return t('it used the positions of the running simulation, which the crash journal does not keep');
  if (text.startsWith('changes to recorded simulation caches')) return t('recorded simulation caches are not kept in the crash journal');
  if (text.includes('display choice of the 3D View')) return t('it was a display choice of the 3D View, not part of the garment');
  if (text.startsWith('it wrote or read a file') || text.startsWith('it opened, saved or replaced the document')) return t('it opened, saved, read or wrote a file');
  if (text.startsWith('it undid or redid a step')) return t('it undid or redid a step that was not repeated');
  if (text.startsWith('its arguments were too large')) return t('its arguments were too large to keep in the journal');
  if (text.startsWith('this line of the journal')) return t('this line of the journal could not be read');
  if (text.startsWith('this command is not repeated')) return t('this command is not repeated after a crash');
  return text;
}

function reasons(rows: unknown, field: 'reason' | 'error'): string {
  const list = Array.isArray(rows) ? rows : [];
  const said = list.map((x) => (x && typeof x === 'object' ? (x as Record<string, unknown>)[field] : null)).filter((x): x is string => typeof x === 'string' && x.length > 0);
  return Array.from(new Set(said.map(reason))).join('; ');
}

function report(r: Record<string, unknown>): Report {
  const n = String(count(r.replayed)), total = String(count(r.logged)), skipped = count(r.skipped_count), failed = count(r.failed_count);
  const back = (r.base === 'autosave' ? t('The last autosave came back, with {n} of {total} commands logged after it.') : t('{n} of {total} logged commands came back on an empty garment.')).replace('{n}', n).replace('{total}', total);
  const parts: string[] = [];
  if (skipped) parts.push(t('Left out on purpose ({n}): {why}.').replace('{n}', String(skipped)).replace('{why}', reasons(r.skipped, 'reason')));
  if (failed) parts.push(t('Could not be repeated ({n}): {why}.').replace('{n}', String(failed)).replace('{why}', reasons(r.failed, 'error')));
  const full = r.truncated === true ? t('The journal was full before the crash, so the last work before it is missing.') : '';
  return { back, missing: parts.join(' '), full };
}

let offered = false;
let asking = false;

export async function offerRecovery(refreshDocument: (path: string | null) => void): Promise<boolean> {
  if (offered || asking) return false;
  asking = true;
  const info = await recoveryInfo().finally(() => { asking = false; });
  if (!info) return false;
  offered = true;
  if (!info.present) return false;
  new RecoveryDialog(refreshDocument).show(info);
  return true;
}
