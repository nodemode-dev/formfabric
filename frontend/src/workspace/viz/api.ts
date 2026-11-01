import { call, catalogue, onEngineLink } from '../../engine/service';
import { toast } from '../../ui/toast';
import { t } from '../../ui/i18n';

const known = new Set<string>();
let loaded = false;
const listeners = new Set<() => void>();

export async function refreshCommands(): Promise<void> {
  try {
    const c = await catalogue(true);
    known.clear();
    for (const x of c.commands) known.add(x.id);
    loaded = true;
    for (const f of listeners) f();
  } catch { return; }
}

export function onCommands(fn: () => void): void {
  listeners.add(fn);
  if (loaded) fn();
}

export function hasCommand(id: string): boolean {
  return known.has(id);
}

onEngineLink((state) => { if (state === 'up') void refreshCommands(); });

export function message(err: unknown): string {
  return err instanceof Error ? err.message : String(err);
}

export async function run<T>(name: string, args: Record<string, unknown> = {}): Promise<T | undefined> {
  try {
    return await call<T>(name, args);
  } catch (err) {
    toast(message(err), 'error');
    return undefined;
  }
}

export async function quiet<T>(name: string, args: Record<string, unknown> = {}): Promise<T | undefined> {
  try {
    return await call<T>(name, args);
  } catch { return undefined; }
}

export function missing(id: string): string {
  return `${t('The engine does not provide')} ${id}`;
}

export function obj(v: unknown): Record<string, unknown> {
  return v && typeof v === 'object' ? (v as Record<string, unknown>) : {};
}

export function num(v: unknown): number | null {
  return typeof v === 'number' && Number.isFinite(v) ? v : null;
}
