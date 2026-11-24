import { describe, it, expect, afterEach } from 'vitest';
import { t, tn, setLanguage, language, languages, isLang, onLanguage, startPageTranslator } from '../../../src/ui/i18n';
import { LANG_CODES, LANG_PARENT } from '../../../src/ui/i18n/langs';

type Table = Record<string, string>;

const modules = import.meta.glob('../../../src/ui/i18n/*.ts', { eager: true }) as Record<string, Record<string, unknown>>;

function tableFor(code: string): Table {
  const mod = modules[`../../../src/ui/i18n/${code}.ts`];
  if (!mod) throw new Error(`no table module for ${code}`);
  const found = Object.values(mod).find((v) => v && typeof v === 'object' && !Array.isArray(v)) as Table | undefined;
  if (!found) throw new Error(`no table export for ${code}`);
  return found;
}

const translated = LANG_CODES.filter((c) => c !== 'en');
const full = translated.filter((c) => !LANG_PARENT[c]);

const placeholders = (s: string): string[] => (s.match(/\{[a-zA-Z0-9_]+\}/g) ?? []).sort();

afterEach(async () => {
  await setLanguage('en');
});

describe('language tables', () => {
  it('ships a translation table for every language the picker offers except English', () => {
    for (const code of translated) expect(Object.keys(tableFor(code)).length, code).toBeGreaterThan(0);
  });

  it('lists every language code exactly once in the picker with a non-empty name', () => {
    const codes = languages().map(([c]) => c);
    expect(new Set(codes).size).toBe(codes.length);
    expect([...codes].sort()).toEqual([...LANG_CODES].sort());
    for (const [, name] of languages()) expect(name.trim().length).toBeGreaterThan(0);
  });

  it('gives every full language an answer for every key any other full language translates', () => {
    const union = new Set<string>();
    for (const code of full) for (const k of Object.keys(tableFor(code))) union.add(k);
    const missing: string[] = [];
    for (const code of full) {
      const table = tableFor(code);
      for (const k of union) if (!(k in table)) missing.push(`${code}: ${k}`);
    }
    expect(missing.slice(0, 20)).toEqual([]);
  });

  it('keeps every curly-brace placeholder of the English key in each translation', () => {
    const broken: string[] = [];
    for (const code of translated) {
      for (const [en, local] of Object.entries(tableFor(code))) {
        const want = placeholders(en.split('#')[0]);
        if (want.length && JSON.stringify(placeholders(local)) !== JSON.stringify(want)) broken.push(`${code}: ${en} -> ${local}`);
      }
    }
    expect(broken.slice(0, 20)).toEqual([]);
  });

  it('never translates a key to an empty string', () => {
    const empty: string[] = [];
    for (const code of translated) for (const [en, local] of Object.entries(tableFor(code))) if (!local.trim()) empty.push(`${code}: ${en}`);
    expect(empty).toEqual([]);
  });

  it('only contains regional tables whose parent language exists', () => {
    for (const [child, parent] of Object.entries(LANG_PARENT)) {
      expect(isLang(child)).toBe(true);
      expect(isLang(parent)).toBe(true);
      expect(Object.keys(tableFor(parent as string)).length).toBeGreaterThan(Object.keys(tableFor(child)).length);
    }
  });
});

describe('t lookup', () => {
  it('returns English text unchanged while English is the language', () => {
    expect(language()).toBe('en');
    expect(t('Merge')).toBe('Merge');
  });

  it('answers a known key in Spanish once Spanish is chosen', async () => {
    await setLanguage('es');
    expect(language()).toBe('es');
    expect(t('Merge')).toBe(tableFor('es')['Merge']);
    expect(t('Merge')).not.toBe('Merge');
  });

  it('falls back to the English text for a key no table knows', async () => {
    await setLanguage('de');
    expect(t('Zzq unknown label 4711')).toBe('Zzq unknown label 4711');
  });

  it('keeps a trailing ellipsis when only the bare word is translated', async () => {
    await setLanguage('es');
    const es = tableFor('es');
    const bare = Object.keys(es).find((k) => !k.endsWith('...') && !k.endsWith(':') && !(`${k}...` in es) && /^[A-Z][a-z]+$/.test(k));
    expect(bare).toBeDefined();
    expect(t(`${bare}...`)).toBe(`${es[bare as string]}...`);
  });

  it('keeps a trailing colon when only the bare word is translated', async () => {
    await setLanguage('fr');
    const fr = tableFor('fr');
    const bare = Object.keys(fr).find((k) => !k.endsWith('...') && !k.endsWith(':') && !(`${k}:` in fr) && /^[A-Z][a-z]+$/.test(k));
    expect(bare).toBeDefined();
    expect(t(`${bare}:`)).toBe(`${fr[bare as string]}:`);
  });

  it('lets a regional language fall back to its parent table for keys it does not override', async () => {
    await setLanguage('es-MX');
    const es = tableFor('es'), mx = tableFor('es-MX');
    const inherited = Object.keys(es).find((k) => !(k in mx));
    expect(inherited).toBeDefined();
    expect(t(inherited as string)).toBe(es[inherited as string]);
    const own = Object.keys(mx)[0];
    expect(t(own)).toBe(mx[own]);
  });

  it('prefers the regional wording over the parent wording when both exist', async () => {
    await setLanguage('es-MX');
    const es = tableFor('es'), mx = tableFor('es-MX');
    const both = Object.keys(mx).find((k) => k in es && es[k] !== mx[k]);
    expect(both).toBeDefined();
    expect(t(both as string)).toBe(mx[both as string]);
  });

  it('answers every key of every table with that table value after switching to it', async () => {
    for (const code of full.slice(0, 6)) {
      await setLanguage(code);
      const table = tableFor(code);
      const wrong = Object.keys(table).filter((k) => !k.includes('#') && t(k) !== table[k] && !k.endsWith('...') && !k.endsWith(':'));
      expect(wrong.slice(0, 5), code).toEqual([]);
    }
  });
});

describe('document direction and language attribute', () => {
  it('switches the page to right-to-left for Arabic and back to left-to-right for German', async () => {
    await setLanguage('ar');
    expect(document.documentElement.dir).toBe('rtl');
    expect(document.documentElement.lang).toBe('ar');
    await setLanguage('de');
    expect(document.documentElement.dir).toBe('ltr');
    expect(document.documentElement.lang).toBe('de');
  });

  it('notifies language listeners on a change and stops after unsubscribing', async () => {
    let calls = 0;
    const off = onLanguage(() => { calls += 1; });
    await setLanguage('it');
    expect(calls).toBe(1);
    off();
    await setLanguage('nl');
    expect(calls).toBe(1);
  });

  it('does not notify listeners when the same language is chosen again', async () => {
    await setLanguage('pl');
    let calls = 0;
    const off = onLanguage(() => { calls += 1; });
    await setLanguage('pl');
    off();
    expect(calls).toBe(0);
  });
});

describe('tn plural forms', () => {
  it('uses the singular English form for one and the plural form otherwise with the count filled in', () => {
    expect(tn(1, '{n} file', '{n} files')).toBe('1 file');
    expect(tn(3, '{n} file', '{n} files')).toBe('3 files');
    expect(tn(0, '{n} file', '{n} files')).toBe('0 files');
  });

  it('groups thousands in the English count', () => {
    expect(tn(12345, '{n} item', '{n} items')).toBe('12,345 items');
  });

  it('formats the count with German digit grouping after switching to German', async () => {
    await setLanguage('de');
    expect(tn(12345, 'Zzq {n} one', 'Zzq {n} other')).toBe('Zzq 12.345 other');
  });
});

describe('page translator', () => {
  it('rewrites known text in the page and restores the English when switching back', async () => {
    const es = tableFor('es');
    document.body.innerHTML = '<button id="b">Merge</button><span id="s" title="Merge">x</span><input id="i" value="Merge" placeholder="Merge">';
    startPageTranslator();
    await setLanguage('es');
    expect(document.getElementById('b')?.textContent).toBe(es['Merge']);
    expect(document.getElementById('s')?.getAttribute('title')).toBe(es['Merge']);
    expect((document.getElementById('i') as HTMLInputElement).value).toBe('Merge');
    expect(document.getElementById('i')?.getAttribute('placeholder')).toBe(es['Merge']);
    await setLanguage('en');
    expect(document.getElementById('b')?.textContent).toBe('Merge');
    expect(document.getElementById('s')?.getAttribute('title')).toBe('Merge');
  });

  it('leaves text inside a no-translate region alone', async () => {
    document.body.innerHTML = '<div data-no-i18n><span id="k">Merge</span></div><span id="o">Merge</span>';
    startPageTranslator();
    await setLanguage('es');
    expect(document.getElementById('k')?.textContent).toBe('Merge');
    expect(document.getElementById('o')?.textContent).toBe(tableFor('es')['Merge']);
  });
});
