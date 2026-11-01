import './anim.css';
import type { ShellDeps } from '../../shell/deps';
import { q } from '../../ui/dom';
import type { DocStore } from '../doc';
import type { Selection } from '../state';
import { AnimDock } from './dock';

export function mountAnim(root: HTMLElement, deps: ShellDeps, doc: DocStore, sel: Selection): void {
  void doc;
  void sel;
  const dock = new AnimDock(q(root, '[data-k="pane3d"]'), q(root, '.ff-bar'), deps);
  if (!deps.commands.get('app.timeline')) {
    deps.commands.add({ id: 'app.timeline', label: 'Timeline', group: 'View', icon: 'tlTimeline', run: () => { dock.setOpen(!dock.open); }, checked: () => dock.open });
  }
}
