export interface DockDeps {
  t: (key: string) => string;
  icons: { icon: (name: string, size?: number) => string };
  serviceUrl: (path: string) => string;
  language?: () => string;
  onFail?: (err: unknown) => void;
  context?: () => string;
  note?: () => string;
}

export interface EngineEvent {
  type: string;
  seq?: number;
  [key: string]: unknown;
}

type Handler = (e: EngineEvent) => void;

export class ServiceError extends Error {
  constructor(message: string, readonly status: number) { super(message); }
}

export class Link {
  private source: EventSource | null = null;
  private readonly handlers = new Map<string, Set<Handler>>();

  constructor(private readonly serviceUrl: (path: string) => string) {}

  url(path: string): string { return this.serviceUrl(path); }

  socketUrl(path: string): string {
    const u = new URL(this.serviceUrl(path), location.href);
    u.protocol = u.protocol === 'https:' ? 'wss:' : 'ws:';
    const token = this.token();
    if (token) u.searchParams.set('ffb_window', token);
    return u.toString();
  }

  private token(): string {
    const found = (window as unknown as { __FFB_WINDOW__?: unknown }).__FFB_WINDOW__;
    return typeof found === 'string' ? found : '';
  }

  async cmd<T = unknown>(id: string, params: Record<string, unknown> = {}): Promise<T> {
    const headers: Record<string, string> = { 'content-type': 'application/json' };
    const token = this.token();
    if (token) headers['x-ffb-window'] = token;
    const res = await fetch(this.serviceUrl(`/api/commands/${encodeURIComponent(id)}`), {
      method: 'POST', headers, body: JSON.stringify(params),
    });
    const body = await res.json().catch(() => ({})) as { result?: T; detail?: string; error?: string };
    if (!res.ok) throw new ServiceError(body.detail || body.error || `${id} failed (${res.status})`, res.status);
    return body.result as T;
  }

  start(): void {
    if (this.source) return;
    const src = new EventSource(this.serviceUrl('/api/events'));
    this.source = src;
    src.onmessage = (m) => {
      let e: EngineEvent;
      try { e = JSON.parse(m.data) as EngineEvent; } catch { return; }
      for (const fn of this.handlers.get(e.type) ?? []) fn(e);
    };
  }

  stop(): void {
    this.source?.close();
    this.source = null;
  }

  on(type: string, fn: Handler): () => void {
    let set = this.handlers.get(type);
    if (!set) { set = new Set(); this.handlers.set(type, set); }
    set.add(fn);
    return () => set?.delete(fn);
  }
}
