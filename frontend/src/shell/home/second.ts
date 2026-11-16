import { onEngineLink, serviceUrl } from '../../engine/service';
import { t } from '../../ui/i18n';
import { toast } from '../../ui/toast';

async function check(): Promise<void> {
  try {
    const response = await fetch(serviceUrl('/api/recovery'), { cache: 'no-store' });
    if (!response.ok) return;
    const o = (await response.json()) as { autosave_active?: unknown; autosave_off_reason?: unknown };
    if (o.autosave_active !== false || o.autosave_off_reason !== 'second_service') return;
    toast(t('Another FormFabric window is using the same data folder. This one keeps no autosave and no crash recovery: save your work by hand.'), 'error', { ms: 0 });
  } catch { return; }
}

export function watchSecondService(): void {
  void check();
  onEngineLink((state) => { if (state === 'up') void check(); });
}
