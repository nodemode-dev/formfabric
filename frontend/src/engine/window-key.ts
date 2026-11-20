const STORE = 'ffb.windowKey';

function take(): string {
  const holder = window as unknown as { __FFB_WINDOW__?: unknown };
  const found = holder.__FFB_WINDOW__;
  try { delete holder.__FFB_WINDOW__; } catch { holder.__FFB_WINDOW__ = undefined; }
  return typeof found === 'string' ? found : '';
}

let key = take();

export function windowKey(): string {
  return key;
}

function stored(): string {
  try { return sessionStorage.getItem(STORE) ?? ''; } catch { return ''; }
}

function keep(value: string): void {
  try { sessionStorage.setItem(STORE, value); } catch { return; }
}

export async function claimWindowKey(base: string): Promise<string> {
  if (key) return key;
  key = stored();
  if (key) return key;
  try {
    const response = await fetch(`${base}/api/window/key`, { method: 'POST', cache: 'no-store' });
    if (!response.ok) return key;
    const body = (await response.json()) as { key?: unknown };
    if (typeof body.key === 'string' && body.key) {
      key = body.key;
      keep(key);
    }
  } catch { return key; }
  return key;
}
