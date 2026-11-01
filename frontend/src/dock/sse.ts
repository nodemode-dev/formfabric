export interface StreamHandlers {
  open(): void;
  message(data: string): void;
  error(refused: boolean): void;
}

export interface Stream {
  close(): void;
}

const RETRY_MS = 1500;

export function openStream(url: string, handlers: StreamHandlers): Stream {
  let stopped = false;
  let controller: AbortController | null = null;
  let timer = 0;

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
    const reader = response.body.getReader();
    const decoder = new TextDecoder();
    let buffer = '';
    try {
      for (;;) {
        const { done, value } = await reader.read();
        if (done) break;
        buffer += decoder.decode(value, { stream: true });
        let cut = buffer.indexOf('\n\n');
        while (cut >= 0) {
          const block = buffer.slice(0, cut);
          buffer = buffer.slice(cut + 2);
          const data = block.split('\n').filter((line) => line.startsWith('data:')).map((line) => line.slice(5).replace(/^ /, '')).join('\n');
          if (data) handlers.message(data);
          cut = buffer.indexOf('\n\n');
        }
      }
    } catch { retry(false); return; }
    retry(false);
  };

  void run();

  return {
    close(): void {
      stopped = true;
      window.clearTimeout(timer);
      controller?.abort();
    },
  };
}
