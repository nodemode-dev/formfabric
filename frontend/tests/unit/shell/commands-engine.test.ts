import { describe, it, expect, vi, beforeEach } from 'vitest';
import { Commands, comboOf, formatKeys, normalizeCombo, engineCommands, paramsOf, hasRequired, scopeOf, type Command } from '../../../src/ui/commands';
import { fuzzyScore, CommandPalette } from '../../../src/ui/command-palette';
import { ShortcutsDialog } from '../../../src/ui/shortcuts';
import { isMac } from '../../../src/ui/dom';
import { call, catalogue, fetchRecent, clearRecent, fetchVersion, serviceUrl, EngineError, type Catalogue } from '../../../src/engine/service';
import { resetPrefs, loadPrefs } from '../../../src/ui/prefs';

const cmd = (id: string, extra: Partial<Command> = {}): Command => ({ id, label: id.toUpperCase(), run: () => undefined, ...extra });

const json = (body: unknown, status = 200): Response => new Response(JSON.stringify(body), { status, headers: { 'Content-Type': 'application/json' } });

beforeEach(() => {
  resetPrefs();
  localStorage.clear();
  sessionStorage.clear();
  document.body.innerHTML = '<div id="overlay-root"></div>';
});

describe('command registry', () => {
  it('refuses to register two commands with the same id', () => {
    const c = new Commands();
    c.add(cmd('a'));
    expect(() => c.add(cmd('a'))).toThrow('duplicate command: a');
  });

  it('moves a shortcut away from another command when it is reassigned in the same scope', () => {
    const c = new Commands();
    c.add(cmd('save', { keys: ['Mod+S'] }), cmd('snap', { keys: ['S'] }));
    c.setKeys('snap', ['Mod+S']);
    expect(c.get('save')?.keys).toEqual([]);
    expect(c.forCombo('Mod+S')?.id).toBe('snap');
  });

  it('lets the pattern view and the 3D view share one shortcut without stealing it', () => {
    const c = new Commands();
    c.add(cmd('pat', { group: 'pattern', keys: ['G'] }), cmd('sim', { group: 'sim', keys: [] }));
    c.setKeys('sim', ['G']);
    expect(c.get('pat')?.keys).toEqual(['G']);
    expect(c.forCombo('G', '2d')?.id).toBe('pat');
    expect(c.forCombo('G', '3d')?.id).toBe('sim');
  });

  it('reports only changed shortcuts as overrides and restores defaults on reset', () => {
    const c = new Commands();
    c.add(cmd('a', { keys: ['A'] }), cmd('b', { keys: ['B'] }));
    c.setKeys('a', ['Shift+A']);
    expect(c.overrides()).toEqual({ a: ['Shift+A'] });
    c.resetKeys();
    expect(c.overrides()).toEqual({});
    expect(c.forCombo('A')?.id).toBe('a');
    expect(c.forCombo('Shift+A')).toBeUndefined();
  });

  it('does not run a disabled or blocked command and runs an enabled one with its parameters', () => {
    const c = new Commands();
    const run = vi.fn();
    c.add(cmd('off', { enabled: () => false, run }), cmd('on', { run }), cmd('held', { run }));
    c.blocked = (x) => (x.id === 'held' ? 'Busy' : '');
    expect(c.run('off')).toBe(false);
    expect(c.run('held')).toBe(false);
    expect(c.why(c.get('held')!)).toBe('Busy');
    expect(c.run('missing')).toBe(false);
    expect(c.run('on', { n: 2 })).toBe(true);
    expect(run).toHaveBeenCalledTimes(1);
    expect(run).toHaveBeenCalledWith({ n: 2 });
  });

  it('awaits the result of an asynchronous command and reports the run to the listener', async () => {
    const c = new Commands();
    const seen: string[] = [];
    c.onRun = (_x, label) => seen.push(label);
    c.add(cmd('calc', { label: () => 'Calculate', run: async () => 42 }));
    expect(await c.call('calc')).toEqual({ ran: true, result: 42 });
    expect(seen).toEqual(['Calculate']);
  });

  it('picks the command of the current view when a key is bound in two views', () => {
    const c = new Commands();
    c.add(cmd('pat', { group: 'pattern', keys: ['G'] }), cmd('sim', { group: 'sim', keys: ['G'] }));
    const ev = new KeyboardEvent('keydown', { key: 'g', code: 'KeyG' });
    c.scopeNow = () => '2d';
    expect(c.forEvent(ev)?.id).toBe('pat');
    c.scopeNow = () => '3d';
    expect(c.forEvent(ev)?.id).toBe('sim');
  });

  it('places pattern and sewing commands in the pattern view and simulation commands in the 3D view', () => {
    expect(scopeOf(cmd('x', { group: 'Pattern' }))).toBe('2d');
    expect(scopeOf(cmd('x', { group: 'sew' }))).toBe('2d');
    expect(scopeOf(cmd('x', { group: 'anim' }))).toBe('3d');
    expect(scopeOf(cmd('x', { group: 'file' }))).toBe('global');
  });
});

describe('key combos', () => {
  it('names a key by its physical position and lists modifiers in a fixed order', () => {
    const ev = new KeyboardEvent('keydown', { key: 'Z', code: 'KeyZ', ctrlKey: true, metaKey: true, shiftKey: true, altKey: true });
    expect(comboOf(ev)).toBe('Mod+Alt+Shift+Z');
    expect(comboOf(new KeyboardEvent('keydown', { key: '[', code: 'BracketLeft' }))).toBe('[');
    expect(comboOf(new KeyboardEvent('keydown', { key: '7', code: 'Numpad7' }))).toBe('7');
    expect(comboOf(new KeyboardEvent('keydown', { key: 'ArrowUp', code: 'ArrowUp' }))).toBe('Up');
  });

  it('shows the platform modifier name when formatting a combo', () => {
    expect(formatKeys('Mod+Shift+S')).toBe(isMac ? '\u2318\u21e7S' : 'Ctrl+Shift+S');
    expect(formatKeys('Mod++')).toBe(isMac ? '\u2318+' : 'Ctrl++');
  });

  it('turns Ctrl, Cmd and Meta written in any case into the platform modifier', () => {
    expect(normalizeCombo('ctrl+Shift+K')).toBe('Mod+Shift+K');
    expect(normalizeCombo('Cmd+O')).toBe('Mod+O');
    expect(normalizeCombo('Meta+P')).toBe('Mod+P');
    expect(normalizeCombo('Alt+X')).toBe('Alt+X');
  });
});

describe('engine catalogue to commands', () => {
  const cat: Catalogue = {
    engine: 'fabric', version: 'x',
    commands: [{
      id: 'piece.scale', name: 'piece.scale', group: 'pattern', label: 'Scale Piece', description: 'Scales a piece', keys: ['Ctrl+Shift+K'],
      annotations: { readOnlyHint: false, destructiveHint: true, idempotentHint: false, openWorldHint: false },
      inputSchema: { type: 'object', required: ['factor'], properties: { factor: { type: 'number', minimum: 0.1, maximum: 10 }, mode: { type: ['string', 'null'], enum: ['a', 'b'] } } },
    }, {
      id: 'doc.info', name: 'doc.info', group: 'file', label: 'Info', description: '', keys: [],
      annotations: { readOnlyHint: true, destructiveHint: false, idempotentHint: true, openWorldHint: false },
      inputSchema: { type: 'object' },
    }],
  };

  it('maps each engine command to a local command with its keys, risk and parameters', () => {
    const invoke = vi.fn(() => 'done');
    const [scale, info] = engineCommands(cat, invoke);
    expect(scale.keys).toEqual(['Mod+Shift+K']);
    expect(scale.mutates).toBe(true);
    expect(scale.risky).toBe(true);
    expect(info.mutates).toBe(false);
    expect(scale.params).toEqual({
      factor: { type: 'number', required: true, enum: undefined, min: 0.1, max: 10, description: undefined },
      mode: { type: 'string', required: false, enum: ['a', 'b'], min: undefined, max: undefined, description: undefined },
    });
    expect(scale.run({ factor: 2 })).toBe('done');
    expect(invoke).toHaveBeenCalledWith(cat.commands[0], { factor: 2 });
  });

  it('says whether a schema has required parameters', () => {
    expect(hasRequired(cat.commands[0].inputSchema)).toBe(true);
    expect(hasRequired(cat.commands[1].inputSchema)).toBe(false);
    expect(paramsOf(undefined)).toEqual({});
  });
});

describe('fuzzy search score', () => {
  it('matches letters in order even with gaps and refuses letters out of order', () => {
    expect(fuzzyScore('opn', 'Open File')).not.toBeNull();
    expect(fuzzyScore('nfo', 'Open File')).toBeNull();
    expect(fuzzyScore('toolong', 'Undo')).toBeNull();
    expect(fuzzyScore('', 'Anything')).toBe(0);
  });

  it('ranks a match at word starts above the same letters inside a word', () => {
    const starts = fuzzyScore('sp', 'Save Pattern') as number;
    const inside = fuzzyScore('sp', 'Display') as number;
    expect(starts).toBeGreaterThan(inside);
  });

  it('ignores letter case', () => {
    expect(fuzzyScore('UNDO', 'undo')).toBe(fuzzyScore('undo', 'undo'));
  });
});

describe('engine client', () => {
  it('posts the command name and arguments as JSON and returns the result', async () => {
    const f = vi.fn(async () => json({ ok: true, result: { pieces: 3 } }));
    vi.stubGlobal('fetch', f);
    await expect(call('doc.count', { kind: 'piece' })).resolves.toEqual({ pieces: 3 });
    const [url, init] = f.mock.calls[0] as unknown as [string, RequestInit];
    expect(url).toBe('/api/engine/dispatch');
    expect(init.method).toBe('POST');
    expect(JSON.parse(String(init.body))).toEqual({ name: 'doc.count', args: { kind: 'piece' } });
  });

  it('turns an engine refusal into an error carrying its code and message', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => json({ ok: false, error: { code: 'no_piece', message: 'No piece is selected' } })));
    const err = await call('piece.scale').catch((e: unknown) => e);
    expect(err).toBeInstanceOf(EngineError);
    expect((err as EngineError).code).toBe('no_piece');
    expect((err as EngineError).message).toBe('No piece is selected');
  });

  it('uses the detail text of an HTTP error reply as the message', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => json({ detail: 'Bad arguments' }, 422)));
    const err = (await call('x').catch((e: unknown) => e)) as EngineError;
    expect(err.code).toBe('http_422');
    expect(err.message).toBe('Bad arguments');
  });

  it('keeps a plain text HTTP error body as the message', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => new Response('engine crashed', { status: 500 })));
    const err = (await call('x').catch((e: unknown) => e)) as EngineError;
    expect(err.code).toBe('http_500');
    expect(err.message).toBe('engine crashed');
  });

  it('reports an unreachable engine when the request itself fails', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => { throw new TypeError('Failed to fetch'); }));
    const err = (await call('x').catch((e: unknown) => e)) as EngineError;
    expect(err.code).toBe('unreachable');
    expect(err.message).toBe('Failed to fetch');
  });

  it('asks for the catalogue once and serves later requests from the first answer', async () => {
    const f = vi.fn(async () => json({ engine: 'e', version: 'v', commands: [] }));
    vi.stubGlobal('fetch', f);
    const a = await catalogue(true);
    const b = await catalogue();
    expect(b).toBe(a);
    expect(f).toHaveBeenCalledTimes(1);
    await catalogue(true);
    expect(f).toHaveBeenCalledTimes(2);
  });

  it('forgets a failed catalogue request so the next request tries again', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => new Response('down', { status: 503 })));
    await expect(catalogue(true)).rejects.toMatchObject({ code: 'http_503' });
    const f = vi.fn(async () => json({ engine: 'e', version: 'v', commands: [] }));
    vi.stubGlobal('fetch', f);
    await expect(catalogue()).resolves.toMatchObject({ engine: 'e' });
    expect(f).toHaveBeenCalledTimes(1);
  });

  it('lists recent file paths and drops entries without a text path', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => json({ files: [{ path: '/a.ffb' }, { path: 3 }, {}, { path: '/b.ffb' }] })));
    expect(await fetchRecent()).toEqual(['/a.ffb', '/b.ffb']);
    vi.stubGlobal('fetch', vi.fn(async () => new Response('', { status: 500 })));
    expect(await fetchRecent()).toBeNull();
  });

  it('removes one recent file by its encoded path with a DELETE request', async () => {
    const f = vi.fn(async () => new Response(null, { status: 204 }));
    vi.stubGlobal('fetch', f);
    expect(await clearRecent('/my files/a&b.ffb')).toBe(true);
    const [url, init] = f.mock.calls[0] as unknown as [string, RequestInit];
    expect(url).toBe('/api/recent?path=%2Fmy%20files%2Fa%26b.ffb');
    expect(init.method).toBe('DELETE');
    vi.stubGlobal('fetch', vi.fn(async () => { throw new Error('offline'); }));
    expect(await clearRecent()).toBe(false);
  });

  it('reads the version reply and answers null when the engine refuses', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => json({ app: 'formfabric', version: '1.2.3' })));
    expect(await fetchVersion()).toEqual({ app: 'formfabric', version: '1.2.3' });
    vi.stubGlobal('fetch', vi.fn(async () => new Response('', { status: 404 })));
    expect(await fetchVersion()).toBeNull();
    expect(serviceUrl('/api/x')).toBe('/api/x');
  });
});

describe('window key', () => {
  it('adds the window key header only to requests for the app api', async () => {
    vi.resetModules();
    const seen: Array<{ url: string; key: string | null }> = [];
    const native = vi.fn(async (input: RequestInfo | URL, init?: RequestInit) => {
      seen.push({ url: String(input), key: new Headers(init?.headers).get('x-ffb-window') });
      return new Response('{}');
    });
    vi.stubGlobal('fetch', native);
    (window as unknown as { __FFB_WINDOW__?: string }).__FFB_WINDOW__ = 'win-123';
    await import('../../../src/engine/service');
    expect((window as unknown as { __FFB_WINDOW__?: string }).__FFB_WINDOW__).toBeUndefined();
    await window.fetch('/api/recent');
    await window.fetch('/assets/logo.svg');
    await window.fetch('https://elsewhere.example/api/x');
    expect(seen).toEqual([
      { url: '/api/recent', key: 'win-123' },
      { url: '/assets/logo.svg', key: null },
      { url: 'https://elsewhere.example/api/x', key: null },
    ]);
  });

  it('asks the engine for a new key and keeps it for the session when none is held', async () => {
    vi.resetModules();
    const f = vi.fn(async () => json({ key: 'fresh-key' }));
    vi.stubGlobal('fetch', f);
    const wk = await import('../../../src/engine/window-key');
    expect(await wk.claimWindowKey('')).toBe('fresh-key');
    expect(wk.windowKey()).toBe('fresh-key');
    expect(sessionStorage.getItem('ffb.windowKey')).toBe('fresh-key');
    const [url, init] = f.mock.calls[0] as unknown as [string, RequestInit];
    expect(url).toBe('/api/window/key');
    expect(init.method).toBe('POST');
    await wk.keySettled();
  });

  it('reuses a key kept from earlier in the session when the engine still accepts it', async () => {
    vi.resetModules();
    sessionStorage.setItem('ffb.windowKey', 'kept-key');
    const f = vi.fn(async () => json({ files: [] }));
    vi.stubGlobal('fetch', f);
    const wk = await import('../../../src/engine/window-key');
    expect(await wk.claimWindowKey('')).toBe('kept-key');
    expect(f).toHaveBeenCalledTimes(1);
    const [url, init] = f.mock.calls[0] as unknown as [string, RequestInit];
    expect(url).toBe('/api/recent');
    expect(new Headers(init.headers).get('x-ffb-window')).toBe('kept-key');
  });

  it('claims a new key when the kept key is refused', async () => {
    vi.resetModules();
    sessionStorage.setItem('ffb.windowKey', 'stale');
    const f = vi.fn(async (input: RequestInfo | URL) => (String(input).endsWith('/api/recent') ? new Response('', { status: 403 }) : json({ key: 'new-one' })));
    vi.stubGlobal('fetch', f);
    const wk = await import('../../../src/engine/window-key');
    expect(await wk.claimWindowKey('')).toBe('new-one');
    expect(sessionStorage.getItem('ffb.windowKey')).toBe('new-one');
  });
});

describe('keyboard shortcuts dialog', () => {
  const setup = (): { c: Commands; d: ShortcutsDialog; row: (id: string) => HTMLElement } => {
    const c = new Commands();
    c.add(cmd('file.save', { label: 'Save', keys: ['Mod+S'] }), cmd('view.grid', { label: 'Toggle Grid', keys: ['G'] }), cmd('pat.cut', { label: 'Cut Line', group: 'pattern', keys: [] }));
    const d = new ShortcutsDialog(c);
    d.show();
    const row = (id: string): HTMLElement => d.scrim.querySelector(`.cv-keys-row[data-key="${id}"]`) as HTMLElement;
    return { c, d, row };
  };
  const press = (code: string, key: string, extra: KeyboardEventInit = {}): void => { window.dispatchEvent(new KeyboardEvent('keydown', { code, key, bubbles: true, ...extra })); };

  it('lists every command with its current shortcut and marks view-only commands with their view', () => {
    const { d, row } = setup();
    expect(d.isOpen).toBe(true);
    expect(row('view.grid').querySelector('.cv-list-meta')?.textContent).toBe('G');
    expect(row('pat.cut').querySelector('.cv-list-name')?.textContent).toBe('Cut Line  (Pattern View)');
    expect(row('pat.cut').querySelector('.cv-list-meta')?.textContent).toBe('None');
    expect((row('view.grid').querySelector('[data-reset]') as HTMLButtonElement).disabled).toBe(true);
  });

  it('filters the list by the search text', () => {
    const { d } = setup();
    const input = d.scrim.querySelector('input[type="search"]') as HTMLInputElement;
    input.value = 'grid';
    input.dispatchEvent(new Event('input'));
    expect(Array.from(d.scrim.querySelectorAll('.cv-keys-row')).map((n) => (n as HTMLElement).dataset.key)).toEqual(['view.grid']);
  });

  it('records a newly pressed shortcut, shows it and stores it for the next start', () => {
    const { c, row } = setup();
    const r = row('view.grid');
    r.querySelector<HTMLElement>('.cv-list-name')!.click();
    expect(r.querySelector('.cv-list-meta')?.textContent).toBe('Press keys...');
    press('KeyH', 'h');
    expect(c.get('view.grid')?.keys).toEqual(['H']);
    expect(row('view.grid')).toBe(r);
    expect(r.querySelector('.cv-list-meta')?.textContent).toBe('H');
    expect(JSON.parse(localStorage.getItem('formfabric.keys') ?? '{}')).toEqual({ 'view.grid': ['H'] });
    expect(loadPrefs().shortcuts).toEqual({ 'view.grid': ['H'] });
    expect((r.querySelector('[data-reset]') as HTMLButtonElement).disabled).toBe(false);
  });

  it('warns about a shortcut in use and moves it only when pressed a second time', () => {
    const { c, d, row } = setup();
    row('view.grid').querySelector<HTMLElement>('.cv-list-name')!.click();
    press('KeyS', 's', { ctrlKey: true, metaKey: true });
    const note = d.scrim.querySelector('.cv-note')?.textContent ?? '';
    expect(note).toContain('is used by Save');
    expect(c.get('view.grid')?.keys).toEqual(['G']);
    press('KeyS', 's', { ctrlKey: true, metaKey: true });
    expect(c.get('view.grid')?.keys).toEqual(['Mod+S']);
    expect(c.get('file.save')?.keys).toEqual([]);
  });

  it('cancels recording with Escape and keeps the old shortcut', () => {
    const { c, row } = setup();
    row('view.grid').querySelector<HTMLElement>('.cv-list-name')!.click();
    press('Escape', 'Escape');
    expect(c.get('view.grid')?.keys).toEqual(['G']);
    expect(row('view.grid').querySelector('.cv-list-meta')?.textContent).toBe('G');
  });

  it('clears a shortcut and then resets it to the default', () => {
    const { c, row } = setup();
    row('file.save').querySelector<HTMLElement>('[data-clear]')!.click();
    expect(c.get('file.save')?.keys).toEqual([]);
    expect(row('file.save').querySelector('.cv-list-meta')?.textContent).toBe('None');
    row('file.save').querySelector<HTMLElement>('[data-reset]')!.click();
    expect(c.get('file.save')?.keys).toEqual(['Mod+S']);
  });
});

describe('command search palette', () => {
  const setup = (): { c: Commands; p: CommandPalette; ran: string[]; input: HTMLInputElement } => {
    const ran: string[] = [];
    const c = new Commands();
    c.add(
      cmd('file.open', { label: 'Open...', keys: ['Mod+O'], run: () => { ran.push('open'); } }),
      cmd('file.export', { label: 'Export PDF', run: () => { ran.push('export'); }, enabled: () => false }),
      cmd('view.grid', { label: 'Grid', checked: () => true, run: () => { ran.push('grid'); } }),
    );
    const p = new CommandPalette(c, () => [{ label: 'File', items: ['file.open', '-', { label: 'Export', items: ['file.export'] }] }, { label: 'View', items: ['view.grid'] }] as never);
    p.show();
    const input = p.scrim.querySelector('input') as HTMLInputElement;
    return { c, p, ran, input };
  };
  const type = (input: HTMLInputElement, text: string): void => { input.value = text; input.dispatchEvent(new Event('input')); };
  const names = (p: CommandPalette): string[] => Array.from(p.scrim.querySelectorAll('.cv-palette-row .cv-list-name')).map((n) => n.textContent ?? '');

  it('shows matching commands with their menu path, state and shortcut', () => {
    const { p, input } = setup();
    type(input, 'grid');
    expect(names(p)).toEqual(['Grid']);
    expect(p.scrim.querySelector('.cv-palette-row .cv-list-meta')?.textContent).toBe('On \u00b7 View');
    type(input, 'open');
    expect(names(p)).toEqual(['Open']);
    expect(p.scrim.querySelector('.cv-palette-row .cv-list-meta')?.textContent).toBe(`File \u00b7 ${formatKeys('Mod+O')}`);
  });

  it('finds a command by the name of the submenu holding it', () => {
    const { p, input } = setup();
    type(input, 'file export');
    expect(names(p)).toContain('Export PDF');
  });

  it('shows the empty note when nothing matches', () => {
    const { p, input } = setup();
    type(input, 'zzzz');
    expect(names(p)).toEqual([]);
    expect((p.scrim.querySelector('.cv-palette-empty') as HTMLElement).hidden).toBe(false);
  });

  it('marks a disabled command and does not run it', () => {
    const { p, ran, input } = setup();
    type(input, 'export pdf');
    const row = p.scrim.querySelector('.cv-palette-row') as HTMLElement;
    expect(row.getAttribute('aria-disabled')).toBe('true');
    row.click();
    expect(ran).toEqual([]);
    expect(p.isOpen).toBe(true);
  });

  it('runs the selected command with Enter after moving down with the arrow key and closes', () => {
    const { p, ran, input } = setup();
    type(input, '');
    const order = names(p);
    input.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowDown' }));
    expect(p.scrim.querySelectorAll('.cv-palette-row')[1].getAttribute('aria-selected')).toBe('true');
    input.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter' }));
    const expected: Record<string, string> = { Open: 'open', Grid: 'grid', 'Export PDF': 'export' };
    expect(order).toHaveLength(3);
    expect(ran).toEqual([expected[order[1]]]);
    expect(p.isOpen).toBe(false);
  });

  it('lists the last run command first when the search is empty', () => {
    const { p, input } = setup();
    type(input, 'grid');
    input.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter' }));
    p.show();
    expect(names(p)[0]).toBe('Grid');
  });
});
