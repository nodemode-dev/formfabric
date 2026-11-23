import { describe, it, expect, vi, beforeEach } from 'vitest';
import { THEMES, DEFAULT_THEME, DEFAULT_ACCENT, STORAGE_KEY, APPEARANCE, readChoice, saveChoice, applyTheme, setTheme, setAccent, setIntensity, themeSwatchesHtml } from '../../../src/ui/themes';
import { toast, noticeHistory, clearNoticeHistory, onNoticeHistory } from '../../../src/ui/toast';
import { patchList, escapeHtml, el, setHtml, mountOnce } from '../../../src/ui/dom';
import { PREFS_KEY, DEFAULT_PREFS, loadPrefs, savePrefs, updatePrefs, resetPrefs, sanitizePrefs } from '../../../src/ui/prefs';

const rootVar = (name: string): string => document.documentElement.style.getPropertyValue(name);

beforeEach(() => {
  resetPrefs();
  localStorage.clear();
  document.body.innerHTML = '';
  document.body.className = '';
  document.documentElement.removeAttribute('style');
  document.documentElement.removeAttribute('data-oled');
});

describe('theme choice', () => {
  it('falls back to the default theme and accent when nothing is saved', () => {
    expect(readChoice()).toEqual({ theme: DEFAULT_THEME, accentColor: DEFAULT_ACCENT, intensity: 1 });
  });

  it('ignores a saved choice from an older appearance version', () => {
    localStorage.setItem(STORAGE_KEY, JSON.stringify({ theme: Object.keys(THEMES).find((k) => k !== DEFAULT_THEME), accentColor: '#123456', appearance: 1 }));
    expect(readChoice().theme).toBe(DEFAULT_THEME);
  });

  it('replaces an unknown saved theme name with the default theme', () => {
    localStorage.setItem(STORAGE_KEY, JSON.stringify({ theme: 'no-such-theme', accentColor: '#123456', appearance: APPEARANCE }));
    expect(readChoice()).toMatchObject({ theme: DEFAULT_THEME, accentColor: '#123456' });
  });

  it('clamps a saved accent intensity into the range zero to one and a half', () => {
    localStorage.setItem(STORAGE_KEY, JSON.stringify({ theme: DEFAULT_THEME, accentColor: '#123456', intensity: 9, appearance: APPEARANCE }));
    expect(readChoice().intensity).toBe(1.5);
    localStorage.setItem(STORAGE_KEY, JSON.stringify({ theme: DEFAULT_THEME, accentColor: '#123456', intensity: -2, appearance: APPEARANCE }));
    expect(readChoice().intensity).toBe(0);
  });

  it('survives a corrupt saved value by returning the defaults', () => {
    vi.spyOn(console, 'error').mockImplementation(() => undefined);
    localStorage.setItem(STORAGE_KEY, '{not json');
    expect(readChoice().theme).toBe(DEFAULT_THEME);
  });

  it('round-trips a saved choice and keeps unrelated stored keys', () => {
    localStorage.setItem(STORAGE_KEY, JSON.stringify({ other: 'kept' }));
    const theme = Object.keys(THEMES)[1];
    saveChoice({ theme, accentColor: '#00ff00', intensity: 0.5 });
    expect(readChoice()).toEqual({ theme, accentColor: '#00ff00', intensity: 0.5 });
    expect(JSON.parse(localStorage.getItem(STORAGE_KEY) as string).other).toBe('kept');
  });
});

describe('applying a theme to the page', () => {
  it('writes every variable of the chosen theme onto the root element', () => {
    const id = Object.keys(THEMES).find((k) => k !== DEFAULT_THEME) as string;
    applyTheme({ theme: id, accentColor: '#3366ff' });
    for (const [k, v] of Object.entries(THEMES[id].vars)) expect(rootVar(k), k).toBe(v);
  });

  it('derives the accent tints from the accent colour as rgba channels', () => {
    applyTheme({ theme: DEFAULT_THEME, accentColor: '#3366ff' });
    expect(rootVar('--accent-color')).toBe('#3366ff');
    expect(rootVar('--accent-color-dim')).toBe('#3366ff33');
    expect(rootVar('--accent-tint').replace(/\s/g, '')).toBe('rgba(51,102,255,0.06)');
    expect(rootVar('--accent-tint-strong').replace(/\s/g, '')).toBe('rgba(51,102,255,0.1)');
  });

  it('expands a three digit accent colour before deriving the tint', () => {
    applyTheme({ theme: DEFAULT_THEME, accentColor: '#f00' });
    expect(rootVar('--accent-tint').replace(/\s/g, '')).toBe('rgba(255,0,0,0.06)');
  });

  it('marks the body as light only for light themes', () => {
    const light = Object.keys(THEMES).find((k) => THEMES[k].light);
    const dark = Object.keys(THEMES).find((k) => !THEMES[k].light) as string;
    expect(light).toBeDefined();
    applyTheme({ theme: light as string, accentColor: '#f97316' });
    expect(document.body.classList.contains('theme-light')).toBe(true);
    applyTheme({ theme: dark, accentColor: '#f97316' });
    expect(document.body.classList.contains('theme-light')).toBe(false);
  });

  it('sets the oled marker only for oled themes and removes it again', () => {
    const oled = Object.keys(THEMES).find((k) => THEMES[k].oled);
    const plain = Object.keys(THEMES).find((k) => !THEMES[k].oled) as string;
    expect(oled).toBeDefined();
    applyTheme({ theme: oled as string, accentColor: '#f97316' });
    expect(document.documentElement.hasAttribute('data-oled')).toBe(true);
    applyTheme({ theme: plain, accentColor: '#f97316' });
    expect(document.documentElement.hasAttribute('data-oled')).toBe(false);
  });

  it('applies the saved theme with no argument so the page is themed before first paint', () => {
    const id = Object.keys(THEMES).find((k) => k !== DEFAULT_THEME) as string;
    saveChoice({ theme: id, accentColor: '#abcdef' });
    applyTheme();
    expect(rootVar('--accent-color')).toBe('#abcdef');
    const someVar = Object.keys(THEMES[id].vars)[0];
    expect(rootVar(someVar)).toBe(THEMES[id].vars[someVar]);
  });

  it('saves, applies and announces a theme change', () => {
    const id = Object.keys(THEMES).find((k) => k !== DEFAULT_THEME) as string;
    const seen: string[] = [];
    const on = (e: Event): void => { seen.push((e as CustomEvent<{ key: string }>).detail.key); };
    document.addEventListener('nm:ui-settings-changed', on);
    setTheme(id);
    setAccent('#010203');
    setIntensity(7);
    document.removeEventListener('nm:ui-settings-changed', on);
    expect(seen).toEqual(['theme', 'accentColor', 'intensity']);
    expect(readChoice()).toEqual({ theme: id, accentColor: '#010203', intensity: 1.5 });
    expect(rootVar('--accent-intensity')).toBe('1.5');
  });

  it('builds one swatch per theme and marks only the active one', () => {
    const host = document.createElement('div');
    const id = Object.keys(THEMES)[2];
    host.innerHTML = themeSwatchesHtml(id, '#112233');
    const swatches = host.querySelectorAll<HTMLButtonElement>('.nm-theme-swatch');
    expect(swatches.length).toBe(Object.keys(THEMES).length);
    const active = host.querySelectorAll('.nm-theme-swatch.is-active');
    expect(active.length).toBe(1);
    expect((active[0] as HTMLElement).dataset.themeId).toBe(id);
    expect((active[0] as HTMLElement).title).toBe(THEMES[id].label);
  });
});

describe('toast notices', () => {
  const toastRoot = document.createElement('div');
  toastRoot.id = 'toast-root';
  beforeEach(() => {
    clearNoticeHistory();
    toastRoot.replaceChildren();
    document.body.append(toastRoot);
  });

  it('shows the notice text in the toast area with a dismiss button', () => {
    toast('Saved the pattern');
    const node = document.querySelector('#toast-root .cv-toast') as HTMLElement;
    expect(node.querySelector('.cv-toast-text')?.textContent).toBe('Saved the pattern');
    expect(node.querySelector('.cv-toast-close')?.getAttribute('aria-label')).toBe('Dismiss notice');
    expect(node.classList.contains('is-error')).toBe(false);
  });

  it('marks error notices and records them in the history with their kind', () => {
    toast('Could not open file', 'error');
    expect(document.querySelector('.cv-toast.is-error')).not.toBeNull();
    expect(noticeHistory().at(-1)).toMatchObject({ text: 'Could not open file', kind: 'error', count: 1 });
  });

  it('shows markup in the text literally instead of building elements', () => {
    toast('<img src=x onerror=alert(1)>');
    expect(document.querySelector('#toast-root img')).toBeNull();
    expect(document.querySelector('.cv-toast-text')?.textContent).toBe('<img src=x onerror=alert(1)>');
  });

  it('folds a repeated notice into one toast and counts it in the history', () => {
    toast('Same text');
    toast('Same text');
    expect(document.querySelectorAll('.cv-toast').length).toBe(1);
    expect(noticeHistory().at(-1)?.count).toBe(2);
  });

  it('keeps at most four toasts on screen, dropping the oldest', () => {
    for (let i = 0; i < 6; i++) toast(`Notice ${i}`);
    const texts = Array.from(document.querySelectorAll('.cv-toast-text')).map((n) => n.textContent);
    expect(texts).toEqual(['Notice 2', 'Notice 3', 'Notice 4', 'Notice 5']);
  });

  it('runs an action when its button is clicked and removes the toast', () => {
    const run = vi.fn();
    toast('Deleted 3 pieces', 'info', { actions: [{ label: 'Undo', run }] });
    const btn = document.querySelector('.cv-toast-action') as HTMLButtonElement;
    expect(btn.textContent).toBe('Undo');
    btn.click();
    expect(run).toHaveBeenCalledTimes(1);
    expect(document.querySelector('.cv-toast')).toBeNull();
  });

  it('removes the toast when the dismiss button is clicked', () => {
    toast('Bye');
    (document.querySelector('.cv-toast-close') as HTMLButtonElement).click();
    expect(document.querySelector('.cv-toast')).toBeNull();
  });

  it('removes the toast by itself after its display time', () => {
    vi.useFakeTimers();
    try {
      toast('Short lived', 'info', { ms: 500 });
      expect(document.querySelector('.cv-toast')).not.toBeNull();
      vi.advanceTimersByTime(499);
      expect(document.querySelector('.cv-toast')).not.toBeNull();
      vi.advanceTimersByTime(2);
      expect(document.querySelector('.cv-toast')).toBeNull();
    } finally {
      vi.useRealTimers();
    }
  });

  it('records a notice in the history even when no toast area exists and tells watchers', () => {
    document.body.innerHTML = '';
    const fn = vi.fn();
    const off = onNoticeHistory(fn);
    toast('Headless notice');
    off();
    expect(fn).toHaveBeenCalled();
    expect(noticeHistory().at(-1)?.text).toBe('Headless notice');
  });

  it('keeps the history to the last one hundred notices', () => {
    for (let i = 0; i < 130; i++) toast(`n${i}`);
    expect(noticeHistory().length).toBe(100);
    expect(noticeHistory()[0].text).toBe('n30');
  });
});

describe('patching lists in place', () => {
  interface Row { id: string; label: string }
  const opts = {
    key: (r: Row) => r.id,
    create: () => document.createElement('li'),
    update: (n: HTMLElement, r: Row) => { n.textContent = r.label; },
  };

  it('reuses the same element for an item when the data arrives again', () => {
    const ul = document.createElement('ul');
    patchList(ul, [{ id: 'a', label: 'A' }, { id: 'b', label: 'B' }], opts);
    const first = ul.children[0];
    patchList(ul, [{ id: 'a', label: 'A2' }, { id: 'b', label: 'B' }], opts);
    expect(ul.children[0]).toBe(first);
    expect(first.textContent).toBe('A2');
  });

  it('reorders, adds and removes elements to match the new item order', () => {
    const ul = document.createElement('ul');
    patchList(ul, [{ id: 'a', label: 'A' }, { id: 'b', label: 'B' }, { id: 'c', label: 'C' }], opts);
    const b = ul.children[1];
    patchList(ul, [{ id: 'c', label: 'C' }, { id: 'b', label: 'B' }, { id: 'd', label: 'D' }], opts);
    expect(Array.from(ul.children).map((n) => n.textContent)).toEqual(['C', 'B', 'D']);
    expect(ul.children[1]).toBe(b);
  });

  it('escapes the five HTML special characters', () => {
    const s = escapeHtml(`<a href="x">'&'</a>`);
    const host = document.createElement('div');
    host.innerHTML = s;
    expect(host.children.length).toBe(0);
    expect(host.textContent).toBe(`<a href="x">'&'</a>`);
  });

  it('builds a single element from markup', () => {
    const b = el<HTMLButtonElement>('<button class="x">Go</button>');
    expect(b.tagName).toBe('BUTTON');
    expect(b.textContent).toBe('Go');
  });

  it('does not rebuild markup when the same key is set again', () => {
    const host = document.createElement('div');
    setHtml(host, '<span>one</span>', 'k1');
    const span = host.firstElementChild;
    setHtml(host, '<span>two</span>', 'k1');
    expect(host.firstElementChild).toBe(span);
    expect(host.textContent).toBe('one');
    setHtml(host, '<span>two</span>', 'k2');
    expect(host.textContent).toBe('two');
  });

  it('mounts markup only once into the same host', () => {
    const host = document.createElement('div');
    mountOnce(host, '<p>first</p>');
    const p = host.firstElementChild;
    expect(() => mountOnce(host, '<p>second</p>')).toThrow(/already rendered/);
    expect(host.firstElementChild).toBe(p);
    expect(host.textContent).toBe('first');
  });
});

describe('preferences', () => {
  it('returns the defaults when nothing is stored', () => {
    expect(loadPrefs()).toEqual(DEFAULT_PREFS);
  });

  it('round-trips saved preferences', () => {
    const p = { ...DEFAULT_PREFS, recentFileCount: 7, language: 'de' as const };
    savePrefs(p);
    expect(loadPrefs()).toEqual(p);
  });

  it('clamps numbers and rejects values of the wrong type or outside the allowed list', () => {
    const clean = sanitizePrefs({ recentFileCount: 99, historyStates: 1, uiFontSize: 'huge', language: 'xx', animations: 'yes', noticeSeconds: Number.NaN });
    expect(clean.recentFileCount).toBe(40);
    expect(clean.historyStates).toBe(10);
    expect(clean.uiFontSize).toBe('normal');
    expect(clean.language).toBe('en');
    expect(clean.animations).toBe(true);
    expect(clean.noticeSeconds).toBe(0);
  });

  it('ignores corrupt stored preferences', () => {
    localStorage.setItem(PREFS_KEY, '[[[');
    expect(loadPrefs()).toEqual(DEFAULT_PREFS);
  });

  it('merges a partial update into the stored preferences and returns the result', () => {
    savePrefs({ ...DEFAULT_PREFS, recentFileCount: 5 });
    const next = updatePrefs({ animations: false });
    expect(next.recentFileCount).toBe(5);
    expect(next.animations).toBe(false);
    expect(loadPrefs().animations).toBe(false);
  });

  it('goes back to the defaults after a reset', () => {
    updatePrefs({ recentFileCount: 3 });
    resetPrefs();
    expect(loadPrefs()).toEqual(DEFAULT_PREFS);
  });
});
