import { describe, it, expect } from 'vitest';
import { renderMarkdown, MarkdownStream } from '../../../src/agent/markdown';
import { readState, readSeen, moveLegacy, jsonLegacy, type DockStore } from '../../../src/dock/state';
import { fill, strings, STRINGS } from '../../../src/dock/strings';

const html = (nodes: HTMLElement[]): string => nodes.map((n) => n.outerHTML).join('');

const memStore = (init: Record<string, string> = {}): DockStore & { data: Record<string, string> } => {
  const data = { ...init };
  return {
    data,
    getItem: (k) => (k in data ? data[k] : null),
    setItem: (k, v) => { data[k] = v; },
    removeItem: (k) => { delete data[k]; },
  };
};

describe('agent reply markdown', () => {
  it('renders headings, emphasis and inline code as elements', () => {
    const [h, p] = renderMarkdown('## Result\nUse **bold** and *soft* and `code`');
    expect(h.className).toBe('ff-md-h ff-md-h2');
    expect(h.textContent).toBe('Result');
    expect(p.querySelector('strong')?.textContent).toBe('bold');
    expect(p.querySelector('em')?.textContent).toBe('soft');
    expect(p.querySelector('code')?.textContent).toBe('code');
    expect(p.textContent).toBe('Use bold and soft and code');
  });

  it('never builds elements from raw markup in the reply', () => {
    const nodes = renderMarkdown('<img src=x onerror=alert(1)> and <script>bad()</script>');
    const host = document.createElement('div');
    host.append(...nodes);
    expect(host.querySelector('img, script')).toBeNull();
    expect(host.textContent).toBe('<img src=x onerror=alert(1)> and <script>bad()</script>');
  });

  it('shows a link as plain text with its address in the tooltip instead of a clickable anchor', () => {
    const [p] = renderMarkdown('See [the docs](https://example.com/x)');
    expect(p.querySelector('a')).toBeNull();
    const link = p.querySelector('.ff-md-link') as HTMLElement;
    expect(link.textContent).toBe('the docs');
    expect(link.title).toBe('https://example.com/x');
  });

  it('keeps code blocks verbatim, including markdown characters inside them', () => {
    const [pre] = renderMarkdown('```ts\nconst a = **b**;\n# not a heading\n```');
    expect(pre.tagName).toBe('PRE');
    expect(pre.querySelector('code')?.textContent).toBe('const a = **b**;\n# not a heading');
    expect(pre.querySelector('strong')).toBeNull();
  });

  it('builds numbered and bulleted lists and joins indented continuation lines', () => {
    const [ol, ul] = renderMarkdown('1. first\n2. second\n   more\n\n- a\n- b');
    expect(ol.tagName).toBe('OL');
    expect(Array.from(ol.children).map((li) => li.textContent)).toEqual(['first', 'second more']);
    expect(ul.tagName).toBe('UL');
    expect(ul.children.length).toBe(2);
  });

  it('builds a table with a header row and skips the divider row', () => {
    const [wrap] = renderMarkdown('| Piece | Area |\n|---|---:|\n| Front | 1200 |\n| Back | 1300 |');
    const rows = Array.from(wrap.querySelectorAll('tr'));
    expect(rows.length).toBe(3);
    expect(Array.from(rows[0].querySelectorAll('th')).map((c) => c.textContent)).toEqual(['Piece', 'Area']);
    expect(Array.from(rows[2].querySelectorAll('td')).map((c) => c.textContent)).toEqual(['Back', '1300']);
  });

  it('joins wrapped lines into one paragraph and splits paragraphs at blank lines', () => {
    const nodes = renderMarkdown('one\ntwo\n\nthree\r\n\n---');
    expect(html(nodes)).toBe('<p>one two</p><p>three</p><hr>');
  });
});

describe('streamed agent reply', () => {
  it('shows unfinished text live and commits finished paragraphs as rendered blocks', () => {
    const s = new MarkdownStream();
    s.push('Hello **wor');
    expect(s.root.querySelector('.ff-md-live')?.textContent).toBe('Hello **wor');
    expect(s.root.querySelector('strong')).toBeNull();
    s.push('ld**\n\nNext');
    expect(s.root.children[0].tagName).toBe('P');
    expect(s.root.children[0].querySelector('strong')?.textContent).toBe('world');
    expect(s.root.querySelector('.ff-md-live')?.textContent).toBe('Next');
  });

  it('does not split a code block at a blank line inside it', () => {
    const s = new MarkdownStream();
    s.push('```\na\n\nb');
    expect(s.root.querySelector('pre')).toBeNull();
    s.push('\n```\n\n');
    expect(s.root.querySelector('pre code')?.textContent).toBe('a\n\nb');
  });

  it('renders the same blocks streamed in small pieces as all at once', () => {
    const text = '# Title\n\nSome *text* here.\n\n- one\n- two\n\n```\ncode\n```\n\nEnd.';
    const s = new MarkdownStream();
    for (let i = 0; i < text.length; i += 3) s.push(text.slice(i, i + 3));
    s.end();
    expect(s.root.innerHTML).toBe(html(renderMarkdown(text)));
    expect(s.root.querySelector('.ff-md-live')).toBeNull();
  });
});

describe('terminal dock state', () => {
  const defaults = { h: 280, w: 420 };

  it('starts closed at the bottom with the default size when nothing is stored', () => {
    expect(readState(memStore(), 'dock', undefined, defaults)).toEqual({ open: false, side: 'bottom', h: 280, w: 420, tab: '' });
  });

  it('reads a stored state and replaces values of the wrong type with defaults', () => {
    const store = memStore({ dock: JSON.stringify({ open: true, side: 'left', h: '300', w: 500, tab: 'chat' }) });
    expect(readState(store, 'dock', undefined, defaults)).toEqual({ open: true, side: 'bottom', h: 280, w: 500, tab: 'chat' });
  });

  it('survives a corrupt stored value', () => {
    expect(readState(memStore({ dock: '{oops' }), 'dock', undefined, defaults).open).toBe(false);
    expect(readSeen(memStore({ dock: '{oops' }), 'dock')).toBe(false);
  });

  it('falls back to the first readable older key', () => {
    const store = memStore({ old1: 'bad json', old2: JSON.stringify({ open: true, side: 'right' }) });
    expect(readState(store, 'dock', ['old1', 'old2'], defaults)).toMatchObject({ open: true, side: 'right' });
  });

  it('moves an older stored state to the new key once and removes the older keys', () => {
    const store = memStore({ old: JSON.stringify({ open: true, h: 333 }) });
    expect(moveLegacy(store, 'dock', jsonLegacy(['old'], true), defaults)).toBe(true);
    expect(store.data.old).toBeUndefined();
    expect(JSON.parse(store.data.dock)).toEqual({ open: true, side: 'bottom', h: 333, w: 420, tab: '', seen: true });
    expect(readSeen(store, 'dock')).toBe(true);
    expect(moveLegacy(store, 'dock', ['old'], defaults)).toBe(false);
  });

  it('does not overwrite a state already stored under the new key', () => {
    const store = memStore({ dock: JSON.stringify({ open: false }), old: JSON.stringify({ open: true }) });
    expect(moveLegacy(store, 'dock', ['old'], defaults)).toBe(false);
    expect(store.data.old).toBeDefined();
  });
});

describe('dock wording', () => {
  it('fills named placeholders and leaves unknown ones in place', () => {
    expect(fill('{program} needs {missing}', { program: 'Shell' })).toBe('Shell needs {missing}');
    expect(fill('{n} of {n}', { n: 3 })).toBe('3 of 3');
  });

  it('fills the app noun into dock sentences and prefers the host override and translation', () => {
    const words = { app: 'FormFabric', noun: 'garment', example: 'x', lead: '' };
    const say = strings({ words, strings: { send: 'Send it' }, t: (_k: string, en: string) => en } as never);
    expect(say('chatAbout')).toBe('Chat with Claude about your garment');
    expect(say('send')).toBe('Send it');
    expect(say('programReady', { program: 'Codex' })).toBe('Codex is installed. Start it in a new tab.');
    const de = strings({ words, t: (k: string, en: string) => (k === 'askPlaceholder' ? 'Frag nach deinem {noun}' : en) } as never);
    expect(de('askPlaceholder')).toBe('Frag nach deinem garment');
  });

  it('has no dock sentence left with an unfilled placeholder once the app words are given', () => {
    const say = strings({ words: { app: 'A', noun: 'N', example: 'E', lead: '' }, t: (_k: string, en: string) => en } as never);
    const vars = { program: 'P', chord: 'C', count: 1, n: 1, name: 'X', seconds: 1, code: 1, path: 'p', url: 'u', cmd: 'c', label: 'l', tool: 't', agent: 'a', key: 'k', error: 'e', total: 2, shown: 1, mins: 1, what: 'w' };
    const left = (Object.keys(STRINGS) as Array<keyof typeof STRINGS>).map((k) => say(k, vars)).filter((s) => /\{\w+\}/.test(s));
    expect(left).toEqual([]);
  });
});
