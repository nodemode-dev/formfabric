import type { Garment } from '../types';
import { itemExists, type FinishKind, type PrintSide, type StyleKind, type TrimItem } from './types';

export interface TrimOptions {
  rotation: number;
  stitchOffset: number | null;
  finishKind: FinishKind;
  finishWidth: number | null;
  finishRatio: number | null;
  finishStrength: number | null;
  printImage: string;
  printWidth: number;
  printHeight: number;
  printSide: PrintSide;
}

class TrimState {
  item: TrimItem | null = null;
  readonly style: Partial<Record<StyleKind, number>> = {};
  readonly opt: TrimOptions = {
    rotation: 0,
    stitchOffset: null,
    finishKind: 'piping',
    finishWidth: null,
    finishRatio: null,
    finishStrength: null,
    printImage: '',
    printWidth: 100,
    printHeight: 100,
    printSide: 'auto',
  };
  private readonly listeners = new Set<() => void>();

  onChange(cb: () => void): () => void {
    this.listeners.add(cb);
    return () => { this.listeners.delete(cb); };
  }

  emit(): void {
    for (const cb of this.listeners) cb();
  }

  select(item: TrimItem | null): void {
    const same = (!item && !this.item) || (item && this.item && item.kind === this.item.kind && item.id === this.item.id);
    this.item = item;
    if (!same) this.emit();
  }

  setStyle(kind: StyleKind, id: number | null): void {
    if (id === null) delete this.style[kind];
    else this.style[kind] = id;
    this.emit();
  }

  prune(g: Garment): void {
    if (this.item && !itemExists(g, this.item)) this.select(null);
  }
}

export const trimState = new TrimState();

export const trimHooks = { addPrint: (_panel: number): void => undefined };
