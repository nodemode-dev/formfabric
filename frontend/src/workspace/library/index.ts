import './library.css';
import type { ShellDeps } from '../../shell/deps';
import type { DocStore } from '../doc';
import type { PatternView } from '../pattern-view/view';
import type { Selection } from '../state';
import { LibraryDrawer } from './drawer';

export function mountLibrary(root: HTMLElement, deps: ShellDeps, doc: DocStore, sel: Selection, view: PatternView): void {
  new LibraryDrawer(root, deps, doc, sel, view);
}
