export interface StreamHandlers {
  open(): void;
  message(data: string): void;
  error(refused: boolean): void;
}

export interface Stream {
  close(): void;
}

const RETRY_MS = 1500;
const IDLE_MS = 45000;
const live = new Set<() => void>();

export function recheckStreams(): void {
  for (const again of Array.from(live)) again();
}

export function openStream(url: string, handlers: StreamHandlers): Stream {
  let stopped = false;
  let controller: AbortController | null = null;
  let timer = 0;
  let recheck = false;
  const again = (): void => { live.delete(again); recheck = true; controller?.abort(); };

  const retry = (refused: boolean): void => {
    if (stopped) return;
    handlers.error(refused);
    timer = window.setTimeout(run, RETRY_MS);
  };

  const run = async (): Promise<void> => {
    controller = new AbortController();
    let response: Response;
    try {
      response = await fetch(url, { cache: 'no-store', headers: { accept: 'text/event-stream' }, signal: controller.signal });
    } catch { retry(false); return; }
    if (!response.ok || !response.body) { retry(!response.ok); return; }
    handlers.open();
    live.add(again);
    const reader = response.body.getReader();
    const decoder = new TextDecoder();
    let buffer = '';
    const watch = (): void => { window.clearTimeout(idle); idle = window.setTimeout(() => controller?.abort(), IDLE_MS); };
    let idle = 0;
    try {
      for (;;) {
        watch();
        const { done, value } = await reader.read();
        if (done) break;
        buffer += decoder.decode(value, { stream: true });
        let from = 0;
        let cut = buffer.indexOf('\n\n', from);
        while (cut >= 0) {
          const block = buffer.slice(from, cut);
          from = cut + 2;
          if (block.startsWith('data:') && block.indexOf('\n') < 0) handlers.message(block.slice(5).replace(/^ /, ''));
          else {
            const data = block.split('\n').filter((line) => line.startsWith('data:')).map((line) => line.slice(5).replace(/^ /, '')).join('\n');
            if (data) handlers.message(data);
          }
          cut = buffer.indexOf('\n\n', from);
        }
        if (from) buffer = buffer.slice(from);
      }
    } catch {
      window.clearTimeout(idle);
      live.delete(again);
      if (recheck && !stopped) { recheck = false; void run(); return; }
      controller?.abort();
      retry(false);
      return;
    }
    window.clearTimeout(idle);
    live.delete(again);
    retry(false);
  };

  void run();

  return {
    close(): void {
      stopped = true;
      live.delete(again);
      window.clearTimeout(timer);
      controller?.abort();
    },
  };
}
