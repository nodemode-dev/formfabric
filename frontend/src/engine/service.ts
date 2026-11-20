import { recheckStreams } from '../agent/sse';
import { onEngineChanged, onEngineLink, type EngineChanged, type Link } from './events';

export { onEngineChanged, onEngineLink };
export type { EngineChanged, Link };

export class EngineError extends Error {
  constructor(readonly code: string, message: string) {
    super(message);
    this.name = 'EngineError';
  }
}

export interface JsonSchema {
  type?: string | string[];
  description?: string;
  properties?: Record<string, JsonSchema>;
  required?: string[];
  enum?: ReadonlyArray<string | number>;
  items?: JsonSchema;
  minimum?: number;
  maximum?: number;
  default?: unknown;
  unit?: string;
}

export interface CatalogueCommand {
  id: string;
  name: string;
  group: string;
  label: string;
  description: string;
  keys: string[];
  annotations: { readOnlyHint: boolean; destructiveHint: boolean; idempotentHint: boolean; openWorldHint: boolean };
  inputSchema: JsonSchema;
}

export interface Catalogue { engine: string; version: string; commands: CatalogueCommand[] }

export interface Version { app: string; version: string }

type Reply<T> = { ok: true; result: T } | { ok: false; error: { code: string; message: string } };

function base(): string {
  const injected = (window as unknown as { __FFB_ENGINE__?: string }).__FFB_ENGINE__;
  return injected ? injected.replace(/\/$/, '') : '';
}

async function failure(response: Response): Promise<EngineError> {
  let message = response.statusText;
  try {
    const text = await response.text();
    message = text || message;
    const detail = (JSON.parse(text) as { detail?: unknown }).detail;
    if (typeof detail === 'string') message = detail;
  } catch { return new EngineError(`http_${response.status}`, message); }
  return new EngineError(`http_${response.status}`, message);
}

export async function call<T>(name: string, args: object = {}): Promise<T> {
  let response: Response;
  try {
    response = await fetch(`${base()}/api/engine/dispatch`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ name, args }) });
  } catch (err) {
    recheckStreams();
    throw new EngineError('unreachable', err instanceof Error ? err.message : String(err));
  }
  if (!response.ok) throw await failure(response);
  const reply = (await response.json()) as Reply<T>;
  if (!reply.ok) throw new EngineError(reply.error.code, reply.error.message);
  return reply.result;
}

let catalogueCache: Promise<Catalogue> | null = null;

export function catalogue(fresh = false): Promise<Catalogue> {
  if (catalogueCache && !fresh) return catalogueCache;
  const pending = (async (): Promise<Catalogue> => {
    let response: Response;
    try { response = await fetch(`${base()}/api/engine/catalogue`, { cache: 'no-store' }); } catch (err) { recheckStreams(); throw new EngineError('unreachable', err instanceof Error ? err.message : String(err)); }
    if (!response.ok) throw await failure(response);
    return (await response.json()) as Catalogue;
  })();
  catalogueCache = pending;
  pending.catch(() => { if (catalogueCache === pending) catalogueCache = null; });
  return pending;
}

export async function fetchRecent(): Promise<string[] | null> {
  try {
    const response = await fetch(`${base()}/api/recent`, { cache: 'no-store' });
    if (!response.ok) return null;
    const body = (await response.json()) as { files?: Array<{ path?: unknown }> };
    return Array.isArray(body.files) ? body.files.map((f) => f.path).filter((p): p is string => typeof p === 'string') : null;
  } catch { return null; }
}

export async function clearRecent(path?: string): Promise<boolean> {
  try {
    const response = await fetch(`${base()}/api/recent${path ? `?path=${encodeURIComponent(path)}` : ''}`, { method: 'DELETE', cache: 'no-store' });
    return response.ok;
  } catch { return false; }
}

export async function fetchVersion(): Promise<Version | null> {
  try {
    const response = await fetch(`${base()}/api/version`, { cache: 'no-store' });
    return response.ok ? ((await response.json()) as Version) : null;
  } catch { return null; }
}

export function serviceUrl(path: string): string {
  return `${base()}${path}`;
}

function installWindowToken(): void {
  const token = (window as unknown as { __FFB_WINDOW__?: unknown }).__FFB_WINDOW__;
  if (typeof token !== 'string' || !token) return;
  const native = window.fetch.bind(window);
  const origin = new URL(base() || location.origin, location.href).origin;
  window.fetch = (input, init) => {
    const url = new URL(input instanceof Request ? input.url : String(input), location.href);
    if (url.origin !== origin || !url.pathname.startsWith('/api/')) return native(input, init);
    const headers = new Headers(init?.headers ?? (input instanceof Request ? input.headers : undefined));
    headers.set('x-ffb-window', token);
    return native(input, { ...init, headers });
  };
}

installWindowToken();
