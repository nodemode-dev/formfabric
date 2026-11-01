import type { DocStore } from '../doc';
import type { Selection } from '../state';
import type { FabricStore } from './store';
import type { FabricInfo } from './types';

export interface Ctx {
  store: FabricStore;
  doc: DocStore;
  sel: Selection;
  selId: number | null;
  selected(): FabricInfo | undefined;
  select(id: number | null): void;
  info(message: string): void;
  fail(message: string): void;
  openLab(): void;
}
