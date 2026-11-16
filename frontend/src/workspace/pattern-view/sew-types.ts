import type { Vec } from '../types';

export type SeamFold = 'turned' | 'flat' | 'custom';
export type SeamSide = 'auto' | 'front' | 'back';

export interface SeamEnd { panel: number; inner: number | null; segment: number; t0: number; t1: number }

export interface Seam {
  id: number;
  a: SeamEnd[];
  b: SeamEnd[];
  reversed: boolean;
  fold: SeamFold;
  fold_angle: number;
  fold_strength: number;
  ratio: number;
  tension: number;
  side: SeamSide;
}

export interface SeamInfo {
  id: number;
  length_a: number;
  length_b: number;
  expected_b: number;
  mismatch: number;
  side_resolved: string;
}

export interface TackEnd { panel: number | null; point: Vec; body_point: [number, number, number] | null }

export interface Tack { id: number; a: TackEnd; b: TackEnd; length: number }

export interface SewSnapshot {
  seams: Seam[];
  tacks: Tack[];
  info: Map<number, SeamInfo>;
  bad: Map<number, number>;
  failed?: boolean;
}

export interface SpanArg { panel: number; inner?: number; segment: number; t0: number; t1: number }

export const EMPTY_SEW: SewSnapshot = { seams: [], tacks: [], info: new Map(), bad: new Map() };

export function spanArg(e: SeamEnd): SpanArg {
  const out: SpanArg = { panel: e.panel, segment: e.segment, t0: e.t0, t1: e.t1 };
  if (e.inner !== null && e.inner !== undefined) out.inner = e.inner;
  return out;
}
