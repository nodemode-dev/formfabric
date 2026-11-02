import '../workspace/library/library.css';
import { Dialog } from '../ui/dialogs';
import { el, q, setText } from '../ui/dom';
import { toast } from '../ui/toast';
import { recoveryAct, recoveryInfo, type Recovery } from '../workspace/library/api';
import { tr } from '../workspace/library/text';

class RecoveryDialog extends Dialog {
  private readonly meta = el('<p class="ff-rec-meta" data-k="meta"></p>');
  private readonly say = el('<p class="ff-lib-note" role="status" data-k="say"></p>');
  private restoreBtn: HTMLButtonElement;
  private discardBtn: HTMLButtonElement;

  constructor(private readonly done: () => void) {
    super('Recover your work', 420);
    this.body.append(el('<p class="ff-rec-meta" data-k="lead"></p>'), this.meta, this.say);
    setText(q(this.body, '[data-k="lead"]'), tr('FormFabric did not close normally. The last autosave and the commands after it can be restored.'));
    this.discardBtn = this.button('Discard', 'ghost', () => void this.act('discard'));
    this.restoreBtn = this.button('Restore', 'primary', () => void this.act('restore'));
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
    const ok = await recoveryAct(action);
    this.restoreBtn.disabled = false;
    this.discardBtn.disabled = false;
    if (!ok) { setText(this.say, tr('The request failed')); toast(tr('The request failed'), 'error'); return; }
    this.close();
    if (action === 'restore') {
      toast(tr('Your work was restored'), 'info');
      document.dispatchEvent(new CustomEvent('ff:recovered'));
      this.done();
    }
  }
}

let offered = false;
let asking = false;

export async function offerRecovery(refreshDocument: () => void): Promise<boolean> {
  if (offered || asking) return false;
  asking = true;
  const info = await recoveryInfo().finally(() => { asking = false; });
  if (!info) return false;
  offered = true;
  if (!info.present) return false;
  new RecoveryDialog(refreshDocument).show(info);
  return true;
}
