import { loadPrefs } from './prefs';
import { LANG_NAMES, LANG_PARENT, RTL_LANGS, isLang as isLangCode, type Lang } from './i18n/langs';

export type { Lang } from './i18n/langs';

type Table = Record<string, string>;
const LOADERS: Record<Exclude<Lang, 'en'>, () => Promise<Table>> = {
  es: () => import('./i18n/es').then((m) => m.es),
  'es-MX': () => import('./i18n/es-MX').then((m) => m.esMX),
  'pt-BR': () => import('./i18n/pt-BR').then((m) => m.ptBR),
  fr: () => import('./i18n/fr').then((m) => m.fr),
  'fr-CA': () => import('./i18n/fr-CA').then((m) => m.frCA),
  de: () => import('./i18n/de').then((m) => m.de),
  it: () => import('./i18n/it').then((m) => m.it),
  nl: () => import('./i18n/nl').then((m) => m.nl),
  da: () => import('./i18n/da').then((m) => m.da),
  fi: () => import('./i18n/fi').then((m) => m.fi),
  nb: () => import('./i18n/nb').then((m) => m.nb),
  sv: () => import('./i18n/sv').then((m) => m.sv),
  ru: () => import('./i18n/ru').then((m) => m.ru),
  pl: () => import('./i18n/pl').then((m) => m.pl),
  cs: () => import('./i18n/cs').then((m) => m.cs),
  hu: () => import('./i18n/hu').then((m) => m.hu),
  uk: () => import('./i18n/uk').then((m) => m.uk),
  tr: () => import('./i18n/tr').then((m) => m.tr),
  'zh-CN': () => import('./i18n/zh-CN').then((m) => m.zhCN),
  'zh-TW': () => import('./i18n/zh-TW').then((m) => m.zhTW),
  ja: () => import('./i18n/ja').then((m) => m.ja),
  ko: () => import('./i18n/ko').then((m) => m.ko),
  id: () => import('./i18n/id').then((m) => m.id),
  th: () => import('./i18n/th').then((m) => m.th),
  vi: () => import('./i18n/vi').then((m) => m.vi),
  ar: () => import('./i18n/ar').then((m) => m.ar),
  hi: () => import('./i18n/hi').then((m) => m.hi),
  el: () => import('./i18n/el').then((m) => m.el),
  'sr-Latn': () => import('./i18n/sr-Latn').then((m) => m.srLatn),
};
const tables = new Map<Lang, Table>([['en', {}]]);

const saved = loadPrefs().language;
let current: Lang = 'en';
let wanted: Lang = saved;
const listeners = new Set<() => void>();

export const languages = (): ReadonlyArray<readonly [Lang, string]> => LANG_NAMES;

export const language = (): Lang => current;

export const isLang = isLangCode;

async function load(l: Lang): Promise<void> {
  for (const code of [LANG_PARENT[l], l]) {
    if (!code || tables.has(code) || code === 'en') continue;
    tables.set(code, await LOADERS[code]());
  }
}

export function t(english: string): string {
  if (current === 'en') return english;
  const tail = english.endsWith('...') ? '...' : english.endsWith(':') && !english.endsWith('::') ? ':' : '';
  const base = tail ? english.slice(0, -tail.length) : english;
  const parent = LANG_PARENT[current];
  const look = (k: string): string | undefined => tables.get(current)?.[k] ?? (parent ? tables.get(parent)?.[k] : undefined);
  const found = look(base) ?? (tail ? look(english) : undefined);
  if (found === undefined) return english;
  return tail && look(base) !== undefined ? `${found}${tail}` : found;
}

export function tn(n: number, one: string, other: string): string {
  const lang = current;
  let cat: Intl.LDMLPluralRule = n === 1 ? 'one' : 'other';
  try { if (lang !== 'en') cat = new Intl.PluralRules(lang).select(n); } catch {  }
  const hit = (key: string): string | undefined => { const f = t(key); return f === key ? undefined : f; };
  const form = lang === 'en' ? (n === 1 ? one : other) : hit(`${other}#${cat}`) ?? hit(cat === 'one' ? one : other) ?? hit(other) ?? (n === 1 ? one : other);
  let shown = String(n);
  try { shown = new Intl.NumberFormat(lang === 'en' ? 'en' : lang).format(n); } catch {  }
  return form.replace('{n}', shown);
}

const known = (text: string): boolean => current !== 'en' && t(text) !== text;

export async function setLanguage(l: Lang): Promise<void> {
  wanted = l;
  try { await load(l); } catch { return; }
  if (wanted !== l || l === current) return;
  current = l;
  document.documentElement.lang = l;
  document.documentElement.dir = RTL_LANGS.has(l) ? 'rtl' : 'ltr';
  for (const fn of listeners) fn();
  translatePage();
}

export async function startLanguage(): Promise<void> {
  if (saved === 'en') return;
  try { await load(saved); } catch { return; }
  if (wanted !== saved) return;
  current = saved;
  document.documentElement.lang = saved;
  document.documentElement.dir = RTL_LANGS.has(saved) ? 'rtl' : 'ltr';
}

export function onLanguage(fn: () => void): () => void {
  listeners.add(fn);
  return () => { listeners.delete(fn); };
}

const ATTRS = ['title', 'aria-label', 'placeholder'] as const;
const SKIP = 'textarea, input, script, style, canvas, [contenteditable], [data-no-i18n], .cv-type-editor, .cv-layer-label, .cv-tab-name, .cv-comp-name, .cv-path-name, .cv-style-name, .cv-home-name, .cv-vtl-clip-name, .cv-vtl-lane-name, .cv-vtl-head-name';
const textEn = new WeakMap<Text, { en: string; shown: string }>();
const attrEn = new WeakMap<Element, Map<string, { en: string; shown: string }>>();
let observer: MutationObserver | null = null;

const skipped = (n: Node | null): boolean => {
  const e = n instanceof Element ? n : n?.parentElement ?? null;
  return !e || !!e.closest(SKIP);
};

function translateText(node: Text): void {
  const raw = node.data, rec = textEn.get(node);
  if (rec && raw === rec.shown) {
    const next = t(rec.en);
    if (next !== raw) { rec.shown = next; node.data = next; }
    if (current === 'en' || next === rec.en) textEn.delete(node);
    return;
  }
  const trimmed = raw.trim();
  if (!trimmed || trimmed.length > 200 || !known(trimmed) || skipped(node)) { if (rec) textEn.delete(node); return; }
  const next = raw.replace(trimmed, t(trimmed));
  textEn.set(node, { en: raw, shown: next });
  node.data = next;
}

function translateAttrs(el: Element): void {
  for (const a of ATTRS) {
    const v = el.getAttribute(a);
    if (v === null) continue;
    let recs = attrEn.get(el);
    const rec = recs?.get(a);
    if (rec && v === rec.shown) {
      const next = t(rec.en);
      if (next !== v) { rec.shown = next; el.setAttribute(a, next); }
      continue;
    }
    if (!known(v) || skippedAttr(el)) { recs?.delete(a); continue; }
    if (!recs) { recs = new Map(); attrEn.set(el, recs); }
    const next = t(v);
    recs.set(a, { en: v, shown: next });
    el.setAttribute(a, next);
  }
}

const SKIP_ATTR = SKIP.replace('textarea, input, ', '');
const skippedAttr = (e: Element): boolean => !!e.closest(SKIP_ATTR);

function walk(root: Node): void {
  if (root instanceof Text) { translateText(root); return; }
  if (!(root instanceof Element)) return;
  if (root.closest(SKIP)) { if (root.matches('input, textarea')) translateAttrs(root); return; }
  translateAttrs(root);
  const it = document.createTreeWalker(root, NodeFilter.SHOW_ELEMENT | NodeFilter.SHOW_TEXT, {
    acceptNode: (n) => {
      if (!(n instanceof Element) || !n.matches(SKIP)) return NodeFilter.FILTER_ACCEPT;
      if (n.matches('input, textarea')) translateAttrs(n);
      return NodeFilter.FILTER_REJECT;
    },
  });
  for (let n = it.nextNode(); n; n = it.nextNode()) {
    if (n instanceof Text) translateText(n);
    else if (n instanceof Element) translateAttrs(n);
  }
}

function translatePage(): void {
  if (typeof document === 'undefined') return;
  observer?.disconnect();
  walk(document.body);
  observer?.observe(document.body, { subtree: true, childList: true, characterData: true, attributes: true, attributeFilter: [...ATTRS] });
}

export function startPageTranslator(): void {
  if (observer || typeof MutationObserver === 'undefined') return;
  observer = new MutationObserver((records) => {
    if (current === 'en') return;
    observer?.disconnect();
    for (const r of records) {
      if (r.type === 'childList') r.addedNodes.forEach((n) => walk(n));
      else if (r.type === 'characterData' && r.target instanceof Text) translateText(r.target);
      else if (r.type === 'attributes' && r.target instanceof Element) translateAttrs(r.target);
    }
    observer?.observe(document.body, { subtree: true, childList: true, characterData: true, attributes: true, attributeFilter: [...ATTRS] });
  });
  translatePage();
}
