import type { ShellDeps } from '../../shell/deps';
import type { DocStore } from '../doc';
import type { PatternView } from '../pattern-view/view';
import type { Selection } from '../state';
import { ObjectsDrawer } from './drawer';

export function mountObjects(root: HTMLElement, deps: ShellDeps, doc: DocStore, sel: Selection, view: PatternView): void {
  new ObjectsDrawer(root, deps, doc, sel, view);
}
