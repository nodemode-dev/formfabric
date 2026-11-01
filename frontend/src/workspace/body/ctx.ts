import type { DocStore } from '../doc';
import type { Selection } from '../state';
import type { BodyStore } from './store';
import type { BodyInfo } from './types';

export interface Ctx {
  store: BodyStore;
  doc: DocStore;
  sel: Selection;
  selected(): BodyInfo | undefined;
  info(message: string): void;
  fail(message: string): void;
}
