import { onEngineLink } from '../engine/service';
import { loadWasm } from '../engine/wasm';
import type { ShellDeps } from '../shell/deps';
import type { Layout } from '../shell/layout';
import { anyDialogOpen } from '../ui/dialogs';
import { q, setClass, setStyleVar, setText } from '../ui/dom';
import { onLanguage, t } from '../ui/i18n';
import { icon } from '../ui/icons';
import { textFieldFocused } from '../ui/native-menu';
import { PREFS_EVENT, updatePrefs } from '../ui/prefs';
import { DocStore } from './doc';
import { Inspector } from './inspector';
import { SeamCheck } from './pattern-view/seam-check';
import { PatternView } from './pattern-view/view';
import { Selection, decimals, toUnit } from './state';
import { trimHooks, trimState } from './trims/state';
import { addPrint } from './trims/tools';
import type { Vec } from './types';
import { View3D } from './view3d';

const RATIO_KEY = 'formfabric.layout.split';
const LAYOUTS: Array<[Layout, string, string, string]> = [
  ['2d', 'app.layout_2d', 'layout2d', 'Pattern Only'],
  ['split', 'app.layout_split', 'layoutSplit', 'Pattern and 3D'],
  ['3d', 'app.layout_3d', 'layout3d', '3D Only'],
];

const SHELL =
  '<div class="ff-ws">' +
  '<div class="ff-bar"><div class="ff-seg" data-k="layouts" role="group"></div><span class="ff-bar-gap"></span><span class="ff-zoom" data-k="zoom"></span></div>' +
  '<div class="ff-body">' +
  '<nav class="ff-tools" data-k="tools"></nav>' +
  '<section class="ff-pane ff-pane-2d" data-k="pane2d"><canvas class="ff-canvas" data-k="canvas2d"></canvas></section>' +
  '<div class="ff-splitter" data-k="splitter" role="separator" aria-orientation="vertical" tabindex="0"></div>' +
  '<section class="ff-pane ff-pane-3d" data-k="pane3d"><canvas class="ff-canvas" data-k="canvas3d"></canvas><p class="ff-nogl" data-k="nogl" hidden></p></section>' +
  '<aside class="ff-inspector" data-k="inspector"></aside>' +
  '</div></div>';

function loadRatio(): number {
  const v = Number(localStorage.getItem(RATIO_KEY));
  return Number.isFinite(v) && v >= 0.15 && v <= 0.85 && v !== 0 ? v : 0.5;
}

export function assemble(host: HTMLElement, deps: ShellDeps): void {
  host.dataset.layout = deps.layout.get();
  const stage = document.createElement('div');
  stage.innerHTML = SHELL;
  const root = stage.firstElementChild as HTMLElement;
  root.dataset.layout = deps.layout.get();
  host.replaceChildren(root);

  const unit = (): string => deps.prefs().units;
  const sel = new Selection();
  const doc = new DocStore(deps.engine, (m) => deps.toast(m, 'error'));

  const inspector = new Inspector(q(root, '[data-k="inspector"]'), doc, sel, unit, () => view.redraw());
  let inspectorQueued = false;
  const scheduleInspector = (): void => {
    if (inspectorQueued) return;
    inspectorQueued = true;
    requestAnimationFrame(() => { inspectorQueued = false; inspector.update(); });
  };
  void import('./fabric').then((m) => m.mountFabric(root, deps, doc, sel));
  void import('./body').then((m) => m.mountBody(root, deps, doc, sel));
  void import('./trims').then((m) => m.mountTrims(root, deps, doc, sel));
  const zoomEl = q(root, '[data-k="zoom"]');

  const toolsEl = q(root, '[data-k="tools"]');
  const toolButtons = new Map<string, HTMLButtonElement>();
  const view = new PatternView(q(root, '[data-k="pane2d"]'), q(root, '[data-k="canvas2d"]'), deps, doc, sel, {
    selected: () => { inspector.update(); view.redraw(); view3d.redraw(); },
    cursor: (w: Vec | null) => {
      const u = unit();
      const d = decimals(u);
      deps.status.setCursor(w ? `X ${toUnit(w[0], u).toFixed(d)}  Y ${toUnit(w[1], u).toFixed(d)} ${u}` : '');
    },
    zoom: (s) => setText(zoomEl, `${Math.round(s * 100)}%`),
    tool: (id) => { for (const [k, b] of toolButtons) setClass(b, 'is-on', k === id); },
  });
  setText(zoomEl, `${Math.round(view.cam.s * 100)}%`);
  trimHooks.addPrint = (id) => { void addPrint(view, id, null); };
  const check = new SeamCheck(q(root, '[data-k="pane2d"]'), doc, sel, () => { inspector.update(); view.redraw(); check.update(); }, (on) => setClass(checkBtn, 'is-on', on));

  const addButton = (id: string, iconName: string, key: string, label: string, onClick: () => void): HTMLButtonElement => {
    const b = document.createElement('button');
    b.type = 'button';
    b.className = 'ff-tool';
    b.dataset.id = id;
    b.innerHTML = `${icon(iconName, 18)}<kbd>${key.replace('Shift+', '⇧')}</kbd>`;
    b.addEventListener('click', () => { onClick(); b.blur(); });
    b.dataset.label = label;
    toolsEl.appendChild(b);
    return b;
  };
  const addRule = (): void => {
    const r = document.createElement('i');
    r.className = 'ff-tool-rule';
    toolsEl.appendChild(r);
  };
  for (const tool of view.tools) {
    if (tool.id === 'seam' || tool.id === 'sew' || tool.id === 'button') addRule();
    const b = addButton(tool.id, tool.icon, tool.key, tool.label, () => view.setTool(tool.id));
    toolButtons.set(tool.id, b);
  }
  addRule();
  const snapBtn = addButton('snap', 'snap', 'N', 'Snap', () => {
    const p = deps.prefs();
    const on = !(p.snapGrid || p.snapPoints || p.snapEdges);
    updatePrefs({ snapGrid: on, snapPoints: on, snapEdges: on });
  });
  const rulerBtn = addButton('rulers', 'rulers', 'U', 'Rulers', () => { view.setRulers(!view.rulers); setClass(rulerBtn, 'is-on', view.rulers); });
  addButton('frame', 'frameAll', 'F', 'Frame', () => view.frame());
  const checkBtn = addButton('seamcheck', 'seamCheck', '', 'Seam Check', () => check.toggle());
  setClass(checkBtn, 'is-on', check.open);
  const titles = (): void => {
    for (const b of toolsEl.querySelectorAll<HTMLButtonElement>('.ff-tool')) {
      const key = b.querySelector('kbd')?.textContent ?? '';
      const text = key ? `${t(b.dataset.label ?? '')} (${key})` : t(b.dataset.label ?? '');
      if (b.title !== text) b.title = text;
    }
  };
  titles();
  onLanguage(titles);
  toolButtons.get(view.tool.id)?.classList.add('is-on');
  setClass(rulerBtn, 'is-on', view.rulers);

  const layoutBox = q(root, '[data-k="layouts"]');
  const layoutButtons = new Map<Layout, HTMLButtonElement>();
  for (const [id, cmd, ico, label] of LAYOUTS) {
    const b = document.createElement('button');
    b.type = 'button';
    b.className = 'ff-seg-btn';
    b.innerHTML = icon(ico, 16);
    b.addEventListener('click', () => { deps.commands.run(cmd); b.blur(); });
    layoutBox.appendChild(b);
    layoutButtons.set(id, b);
    b.dataset.label = label;
  }
  const paintLayout = (l: Layout): void => {
    root.dataset.layout = l;
    host.dataset.layout = l;
    for (const [id, b] of layoutButtons) setClass(b, 'is-on', id === l);
    setText(zoomEl, `${Math.round(view.cam.s * 100)}%`);
  };
  const layoutTitles = (): void => { for (const b of layoutButtons.values()) { const text = t(b.dataset.label ?? ''); if (b.title !== text) b.title = text; } };
  layoutTitles();
  onLanguage(layoutTitles);
  paintLayout(deps.layout.get());
  deps.layout.onChange(paintLayout);

  let ratio = loadRatio();
  const applyRatio = (): void => {
    setStyleVar(root, '--ff-ra', `${ratio}fr`);
    setStyleVar(root, '--ff-rb', `${1 - ratio}fr`);
  };
  applyRatio();
  const splitter = q(root, '[data-k="splitter"]');
  const pane2d = q(root, '[data-k="pane2d"]');
  const pane3d = q(root, '[data-k="pane3d"]');
  const from = (x: number): void => {
    const a = pane2d.getBoundingClientRect().left;
    const b = pane3d.getBoundingClientRect().right;
    if (b - a < 100) return;
    ratio = Math.max(0.15, Math.min(0.85, (x - a) / (b - a)));
    applyRatio();
  };
  let dragging = false;
  splitter.addEventListener('pointerdown', (e) => { dragging = true; splitter.setPointerCapture(e.pointerId); setClass(splitter, 'is-drag', true); });
  splitter.addEventListener('pointermove', (e) => { if (dragging) from(e.clientX); });
  const stop = (): void => { if (!dragging) return; dragging = false; setClass(splitter, 'is-drag', false); localStorage.setItem(RATIO_KEY, String(ratio)); };
  splitter.addEventListener('pointerup', stop);
  splitter.addEventListener('pointercancel', stop);
  splitter.addEventListener('dblclick', () => { ratio = 0.5; applyRatio(); localStorage.setItem(RATIO_KEY, '0.5'); });
  splitter.addEventListener('keydown', (e) => {
    if (e.key !== 'ArrowLeft' && e.key !== 'ArrowRight') return;
    e.preventDefault();
    ratio = Math.max(0.15, Math.min(0.85, ratio + (e.key === 'ArrowLeft' ? -0.02 : 0.02)));
    applyRatio();
    localStorage.setItem(RATIO_KEY, String(ratio));
  });

  const view3d = new View3D(pane3d, q(root, '[data-k="canvas3d"]'), { doc, sel, selected: () => { inspector.update(); view.redraw(); } });
  const noGl = q(root, '[data-k="nogl"]');
  if (!view3d.available) {
    noGl.hidden = false;
    const paintGl = (): void => setText(noGl, t('WebGL2 is not available'));
    paintGl();
    onLanguage(paintGl);
  }

  doc.onChange(() => {
    sel.prune(new Set(doc.garment.panels.map((p) => p.id)), new Set(doc.sew.seams.map((s) => s.id)), new Set(doc.sew.tacks.map((k) => k.id)));
    trimState.prune(doc.garment);
    check.update();
    scheduleInspector();
    view.redraw();
  });
  const refreshed = async (name: string): Promise<void> => {
    const fresh = name === 'file.new' || name === 'file.open' || name === 'exchange.import_dxf';
    if (fresh) sel.clear();
    await doc.refresh();
    if (fresh) view.frame();
  };
  deps.engine.onEngineChanged((e) => { if (e.name !== 'view.units') void refreshed(e.name); });
  onEngineLink((state) => { if (state === 'up') void doc.refresh(); });
  void loadWasm().then((w) => { doc.setWasm(w); });
  void doc.refresh();

  const syncPrefs = (): void => {
    const p = deps.prefs();
    setClass(snapBtn, 'is-on', p.snapGrid || p.snapPoints || p.snapEdges);
    inspector.update();
    view.redraw();
  };
  syncPrefs();
  document.addEventListener(PREFS_EVENT, syncPrefs);

  const SEW_TOOLS = ['sew', 'freeseam', 'editseam', 'tack', 'pleat', 'gather', 'button', 'buttonhole', 'fasten', 'zipper', 'topstitch', 'finish', 'print', 'trimselect'];
  const tabs = document.querySelector('.cv-ws');
  const followTab = (): void => {
    const id = tabs?.querySelector<HTMLElement>('.cv-ws-tab.is-active')?.dataset.ws;
    if (id === 'sew' && !SEW_TOOLS.includes(view.tool.id)) view.setTool('sew');
    else if (id === 'pattern' && SEW_TOOLS.includes(view.tool.id)) view.setTool('select');
  };
  if (tabs) new MutationObserver(followTab).observe(tabs, { attributes: true, attributeFilter: ['class'], subtree: true });
  followTab();

  const locked = (e: KeyboardEvent): boolean => {
    if (anyDialogOpen() || textFieldFocused()) return true;
    if (document.querySelector('.cv-home:not([hidden])')) return true;
    const target = e.target;
    return target instanceof HTMLButtonElement && (e.key === 'Enter' || e.key === ' ');
  };
  document.addEventListener('keydown', (e) => {
    if (e.defaultPrevented || locked(e)) return;
    if (e.code === 'Space' && !e.repeat && deps.layout.get() !== '3d') { view.setSpace(true); e.preventDefault(); return; }
    const mod = e.ctrlKey || e.metaKey;
    if (deps.layout.get() === '3d') return;
    if (mod && !e.shiftKey && !e.altKey && e.key.toLowerCase() === 'd') {
      if (sel.panels.size) void doc.run('pattern.duplicate', { panels: sel.list() });
      e.preventDefault();
      return;
    }
    if (e.key === 'F2') { if (sel.panels.size === 1) { inspector.focusName(); e.preventDefault(); } return; }
    if (!mod && !e.altKey && e.key.toUpperCase() === 'N') { snapBtn.click(); e.preventDefault(); return; }
    if (!mod && !e.altKey && e.key.toUpperCase() === 'U') { rulerBtn.click(); e.preventDefault(); return; }
    if (view.keydown(e)) e.preventDefault();
  }, true);
  document.addEventListener('keyup', (e) => { if (e.code === 'Space') view.setSpace(false); });
  window.addEventListener('blur', () => view.setSpace(false));
}
