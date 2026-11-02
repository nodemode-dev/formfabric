import type { Commands } from '../ui/commands';
import type { MenuDef, MenuItem } from '../ui/menu';
import { PREF_SECTIONS } from '../ui/settings';
import { SAMPLES, SAMPLE_BODIES } from './window-commands';

const NONE: MenuItem[] = ['app.none'];

export function buildMenus(commands: Commands, recent: () => string[]): MenuDef[] {
  const has = (id: string): boolean => !!commands.get(id);
  const present = (ids: MenuItem[]): MenuItem[] => ids.filter((i) => typeof i !== 'string' || i === '-' || has(i));
  const used = new Set<string>();
  const fixed = (ids: MenuItem[]): MenuItem[] => {
    const out = present(ids);
    for (const i of out) if (typeof i === 'string') used.add(i);
    return out;
  };
  const group = (name: string): string[] => commands.all().filter((c) => c.group === name && !used.has(c.id)).map((c) => c.id);
  const tidy = (items: MenuItem[]): MenuItem[] => {
    const out: MenuItem[] = [];
    for (const i of items) {
      if (i === '-' && (!out.length || out[out.length - 1] === '-')) continue;
      out.push(i);
    }
    while (out.length && out[out.length - 1] === '-') out.pop();
    return out;
  };
  const withGroup = (head: MenuItem[], name: string): MenuItem[] => {
    const rest = group(name);
    const items = tidy([...head, ...(head.length && rest.length ? ['-' as const] : []), ...rest]);
    return items.length ? items : NONE;
  };

  return [
    { label: 'File', items: () => {
      used.clear();
      const files = recent().slice(0, 10).map((_, i) => `app.recent.${i}`);
      const exports = fixed(commands.all().filter((c) => c.id.startsWith('app.export.')).map((c) => c.id));
      fixed(['export.formats', 'export.run']);
      const others = group('Export');
      const sub: MenuItem[] = [...exports, ...(exports.length && others.length ? ['-' as const] : []), ...others];
      const samples: MenuItem[] = SAMPLES.map(([n, label]) => ({ label, items: fixed(SAMPLE_BODIES.map(([slot]) => `app.sample.${n}.${slot}`)) }));
      return tidy([...fixed(['file.new', 'file.open']), { label: 'Open Recent', items: files.length ? [...files, '-', 'app.clear_recent'] : NONE }, { label: 'Open Sample', items: samples }, '-', ...fixed(['file.save', 'app.save_as', '-', 'app.home']), ...(sub.length ? ['-' as const, { label: 'Export', items: sub }] : []), '-', ...fixed(['app.exit'])]);
    } },
    { label: 'Edit', items: () => {
      used.clear();
      return tidy([...fixed(['history.undo', 'history.redo', 'history.list', '-', 'app.cut', 'app.copy', 'app.paste', 'app.select_all', '-', 'pattern.duplicate', 'pattern.delete', 'pattern.rename', '-', 'app.palette', 'app.shortcuts', 'app.settings']), { label: 'Preferences', items: PREF_SECTIONS.map((s) => `app.prefs.${s}`) }]);
    } },
    { label: 'Pattern', items: () => withGroup(fixed(['pattern.draw_panel', 'pattern.rectangle', 'pattern.ellipse', '-', 'pattern.inner_shape', 'pattern.inner_line', '-', 'pattern.transform', 'pattern.curvature', 'pattern.split_line', 'pattern.add_point', 'pattern.delete_point', 'pattern.set_point_kind', 'pattern.smooth', '-', 'pattern.measure']), 'Pattern') },
    { label: 'Sew', items: () => withGroup([], 'Sew') },
    { label: 'Drape', items: () => withGroup([], 'Sim') },
    { label: 'Body', items: () => withGroup([], 'Body') },
    { label: 'Fabric', items: () => withGroup([], 'Fabric') },
    { label: 'Output', items: () => { used.clear(); return tidy(fixed(['app.output.panel', 'app.output.uv', '-', 'app.output.guides', 'app.output.bake', 'app.output.gameready'])); } },
    { label: 'View', items: () => {
      used.clear();
      return withGroup(fixed(['app.layout_2d', 'app.layout_split', 'app.layout_3d', '-', 'app.snap_grid', 'app.snap_points', 'app.snap_edges', '-', 'app.units_mm', 'app.units_cm', 'app.units_in', '-', 'app.theme_toggle', 'app.custom_title_bar', 'app.menus_in_window']), 'View');
    } },
    { label: 'Window', items: () => tidy(fixed(['app.ws_pattern', 'app.ws_sew', 'app.ws_drape', 'app.ws_animate', 'app.ws_render', '-', 'app.terminal', 'app.claude', '-', 'app.home'])) },
    { label: 'Help', items: () => { used.clear(); return tidy([...fixed(['app.about', 'app.shortcuts', '-', 'app.source', 'app.report_issue']), ...group('System').length ? ['-' as const, ...group('System')] : []]); } },
  ];
}
