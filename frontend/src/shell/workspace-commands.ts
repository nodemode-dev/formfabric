import type { Commands } from '../ui/commands';
import { confirmAction, promptName } from '../ui/dialogs';
import { toast } from '../ui/toast';
import { tr } from '../workspace/library/text';
import type { KeyHints } from './keyhints';
import { WorkspaceLayouts } from './layouts';
import type { LayoutState } from './layout';
import { exportSettings, importSettings } from './settings-io';

export const LAYOUT_SLOTS = 24;

export interface WorkspaceCtx {
  commands: Commands;
  layout: LayoutState;
  hints: KeyHints;
}

export function registerWorkspaceCommands(ctx: WorkspaceCtx): WorkspaceLayouts {
  const { commands } = ctx;
  const layouts = new WorkspaceLayouts(ctx.layout);
  let over3d = false;
  document.addEventListener('pointerover', (e) => {
    if (!(e.target instanceof Element)) return;
    if (e.target.closest('[data-k="pane3d"]')) over3d = true;
    else if (e.target.closest('[data-k="pane2d"]')) over3d = false;
  }, true);
  document.addEventListener('ff:command', (e) => { const id = (e as CustomEvent<{ id: string }>).detail.id; void commands.call(id); });
  commands.scopeNow =() => { const l = ctx.layout.get(); return l === '3d' ? '3d' : l === '2d' ? '2d' : over3d ? '3d' : '2d'; };
  commands.add(
    { id: 'app.layout.save', label: 'Save Layout...', group: 'Window', icon: 'libLayout', run: async () => {
      const name = await promptName(tr('Layout name'), layouts.active() ?? '', 'Save');
      if (!name) return;
      if (layouts.names().includes(name) && !(await confirmAction('Replace the layout?', name, 'Replace', true))) return;
      layouts.save(name);
      toast(`${tr('Saved layout')} ${name}`, 'info');
    } },
    { id: 'app.layout.delete', label: 'Delete Layout...', group: 'Window', enabled: () => layouts.names().some((n) => !layouts.isDefault(n)), run: async () => {
      const mine = layouts.names().filter((n) => !layouts.isDefault(n));
      const name = await promptName(`${tr('Delete which layout')}: ${mine.join(', ')}`, mine[mine.length - 1] ?? '', 'Delete');
      if (name && layouts.remove(name)) toast(`${tr('Deleted layout')} ${name}`, 'info');
    } },
    { id: 'app.key_hints', label: 'Key Hints', group: 'Window', icon: 'libKeys', checked: () => ctx.hints.sticky, run: () => ctx.hints.setSticky(!ctx.hints.sticky) },
    { id: 'app.settings_export', label: 'Export Settings...', group: 'Window', run: async () => {
      const where = await exportSettings(commands);
      if (where) toast(`${tr('Settings exported')}: ${where}`, 'info');
    } },
    { id: 'app.settings_import', label: 'Import Settings...', group: 'Window', run: async () => {
      const r = await importSettings(commands);
      if (!r) return;
      if (!r.ok) { toast(tr(r.message), 'error'); return; }
      toast(`${tr('Settings imported')}: ${r.prefs} ${tr('preferences')}, ${r.shortcuts} ${tr('shortcuts')}, ${r.layouts} ${tr('layouts')}`, 'info');
    } },
  );
  for (let i = 0; i < LAYOUT_SLOTS; i++) {
    commands.add({ id: `app.layout.slot.${i}`, label: () => layouts.names()[i] ?? '', group: 'Window', enabled: () => !!layouts.names()[i], checked: () => layouts.names()[i] === layouts.active(), run: () => { const n = layouts.names()[i]; if (n) layouts.apply(n); } });
  }
  return layouts;
}

export function layoutSlotIds(layouts: WorkspaceLayouts): string[] {
  return layouts.names().slice(0, LAYOUT_SLOTS).map((_, i) => `app.layout.slot.${i}`);
}
