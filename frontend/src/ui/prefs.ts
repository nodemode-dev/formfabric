import { LANG_CODES, type Lang } from './i18n/langs';

export type DialogValue = string | boolean;

export const PREFS_KEY = 'formfabric.prefs';
export const PREFS_EVENT = 'ff:prefs-changed';

interface Field<T> { def: T; read(v: unknown): T }

const flag = (def: boolean): Field<boolean> => ({ def, read: (v) => (typeof v === 'boolean' ? v : def) });
const num = (def: number, min: number, max: number): Field<number> => ({ def, read: (v) => (typeof v === 'number' && Number.isFinite(v) ? Math.min(max, Math.max(min, v)) : def) });
const text = (def: string, max = 200): Field<string> => ({ def, read: (v) => (typeof v === 'string' ? v.slice(0, max) : def) });
const pick = <T extends string | number>(def: T, options: readonly T[]): Field<T> => ({ def, read: (v) => (options.includes(v as T) ? (v as T) : def) });
const list = (): Field<string[]> => ({ def: [], read: (v) => (Array.isArray(v) ? v.filter((x): x is string => typeof x === 'string').slice(0, 200) : []) });

export const UNITS = ['mm', 'cm', 'in'] as const;
export const LAYOUTS = ['split', '2d', '3d'] as const;
export const FONT_SIZES = ['small', 'normal', 'large'] as const;

const SPEC = {
  language: pick<Lang>('en', LANG_CODES),
  autoShowHome: flag(true),
  homeOnLastClose: flag(false),
  updateAtStart: flag(true),
  recentFileCount: num(10, 0, 40),
  themeAuto: flag(false),
  themeLight: text('light', 60),
  themeDark: text('studio', 60),
  favouriteThemes: list(),
  uiFontSize: pick<(typeof FONT_SIZES)[number]>('normal', FONT_SIZES),
  animations: flag(true),
  noticeSeconds: num(0, 0, 30),
  defaultLayout: pick<(typeof LAYOUTS)[number]>('split', LAYOUTS),
  rememberLayout: flag(true),
  useGpu: flag(true),
  renderer: pick<'auto' | 'webgpu' | 'webgl'>('auto', ['auto', 'webgpu', 'webgl']),
  framePacingMs: pick<number>(0, [0, 16, 33]),
  workerThreads: num(0, 0, 16),
  historyStates: num(200, 10, 1000),
  historyCapMb: num(64, 0, 512),
  units: pick<(typeof UNITS)[number]>('mm', UNITS),
  gridSize: num(10, 1, 1000),
  snapGrid: flag(true),
  snapPoints: flag(true),
  snapEdges: flag(true),
  seamAllowance: num(10, 0, 50),
  simSubsteps: num(8, 1, 64),
  simGravity: num(9.81, 0, 30),
  simThickness: num(2, 0.1, 20),
  simDevice: pick<'auto' | 'cpu' | 'gpu'>('auto', ['auto', 'cpu', 'gpu']),
  vpShadows: flag(true),
  vpAntialias: pick<number>(4, [0, 2, 4]),
  vpBackground: pick<'theme' | 'dark' | 'light'>('theme', ['theme', 'dark', 'light']),
  vpFloor: flag(true),
  agentsAllowed: flag(true),
  nodemodeUrl: text('http://localhost:3000', 200),
};

type Spec = typeof SPEC;
type Simple = { [K in keyof Spec]: Spec[K] extends Field<infer T> ? T : never };

export interface Prefs extends Simple {
  dialogValues: Record<string, Record<string, DialogValue>>;
  shortcuts: Record<string, string[]>;
  shortcutSets: Record<string, Record<string, string[]>>;
}

const isRecord = (v: unknown): v is Record<string, unknown> => !!v && typeof v === 'object' && !Array.isArray(v);

function dialogValues(v: unknown): Prefs['dialogValues'] {
  const out: Prefs['dialogValues'] = {};
  if (!isRecord(v)) return out;
  for (const [title, vals] of Object.entries(v)) {
    if (!isRecord(vals)) continue;
    const kept: Record<string, DialogValue> = {};
    for (const [k, x] of Object.entries(vals)) if (typeof x === 'string' || typeof x === 'boolean') kept[k] = x;
    out[title] = kept;
  }
  return out;
}

function combos(v: unknown): Record<string, string[]> {
  const out: Record<string, string[]> = {};
  if (!isRecord(v)) return out;
  for (const [id, keys] of Object.entries(v)) if (Array.isArray(keys)) out[id] = keys.filter((k): k is string => typeof k === 'string');
  return out;
}

function sets(v: unknown): Prefs['shortcutSets'] {
  const out: Prefs['shortcutSets'] = {};
  if (!isRecord(v)) return out;
  for (const [name, bindings] of Object.entries(v)) out[name] = combos(bindings);
  return out;
}

function defaults(): Prefs {
  const simple = {} as Record<string, unknown>;
  for (const [k, f] of Object.entries(SPEC)) simple[k] = (f as Field<unknown>).def;
  return { ...(simple as Simple), dialogValues: {}, shortcuts: {}, shortcutSets: {} };
}

export const DEFAULT_PREFS: Prefs = defaults();

let cache: Prefs | null = null;

export function loadPrefs(): Prefs {
  if (cache) return cache;
  let raw: unknown = null;
  try { raw = JSON.parse(localStorage.getItem(PREFS_KEY) ?? 'null'); } catch { raw = null; }
  const src = isRecord(raw) ? raw : {};
  const simple = {} as Record<string, unknown>;
  for (const [k, f] of Object.entries(SPEC)) simple[k] = (f as Field<unknown>).read(src[k]);
  cache = { ...(simple as Simple), dialogValues: dialogValues(src.dialogValues), shortcuts: combos(src.shortcuts), shortcutSets: sets(src.shortcutSets) };
  return cache;
}

export function savePrefs(p: Prefs): void {
  cache = p;
  try { localStorage.setItem(PREFS_KEY, JSON.stringify(p)); } catch { return; }
}

export function updatePrefs(patch: Partial<Prefs>): Prefs {
  const next = { ...loadPrefs(), ...patch };
  savePrefs(next);
  document.dispatchEvent(new CustomEvent(PREFS_EVENT, { detail: { keys: Object.keys(patch) } }));
  return next;
}

export function resetPrefs(): void {
  cache = null;
  try {
    for (const k of Object.keys(localStorage)) if (k.startsWith('formfabric.') || k === 'nodemode_ui_settings') localStorage.removeItem(k);
  } catch { return; }
}
