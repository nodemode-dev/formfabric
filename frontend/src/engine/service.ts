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
    try { response = await fetch(`${base()}/api/engine/catalogue`, { cache: 'no-store' }); } catch (err) { throw new EngineError('unreachable', err instanceof Error ? err.message : String(err)); }
    if (!response.ok) throw await failure(response);
    return (await response.json()) as Catalogue;
  })();
  catalogueCache = pending;
  pending.catch(() => { if (catalogueCache === pending) catalogueCache = null; });
  return pending;
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
