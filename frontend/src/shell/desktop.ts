export const inDesktop = (): boolean => '__TAURI_INTERNALS__' in window;

export interface ShellHooks {
  open: (paths: string[]) => Promise<void>;
  exitRequested: () => void;
}

export async function listenShell(on: ShellHooks): Promise<void> {
  if (!inDesktop()) return;
  const [events, core] = await Promise.all([import('@tauri-apps/api/event'), import('@tauri-apps/api/core')]);
  let opening: Promise<void> = Promise.resolve();
  const queue = (paths: unknown): void => {
    const list = Array.isArray(paths) ? paths.filter((p): p is string => typeof p === 'string' && !!p) : [];
    if (list.length) opening = opening.then(() => on.open(list)).catch(() => undefined);
  };
  await events.listen<string[]>('formfabric://open-paths', (e) => queue(e.payload));
  await events.listen('formfabric://exit-requested', () => on.exitRequested());
  queue(await core.invoke<string[]>('take_pending_opens', {}));
}

export async function confirmExit(cancel = false): Promise<void> {
  const core = await import('@tauri-apps/api/core');
  await core.invoke<void>('confirm_exit', cancel ? { cancel: true } : {});
}
