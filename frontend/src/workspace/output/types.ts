export type Vec = [number, number];

export type OptKind = 'boolean' | 'number' | 'integer' | 'string' | 'enum' | 'pair';

export interface OptSpec {
  key: string;
  kind: OptKind;
  label: string;
  hint: string;
  choices: Array<[string, string]>;
  min: number | null;
  max: number | null;
  def: unknown;
}

export interface FormatInfo {
  id: string;
  label: string;
  ext: string;
  opts: OptSpec[];
}

export interface UvIslandInfo {
  panel: number;
  name: string;
  offset: Vec;
  rotation: number;
  scale: number;
  tile: number;
  loops: Vec[][];
}

export interface UvState {
  islands: UvIslandInfo[];
  tiles: number;
  cols: number;
  padding: number;
}

export interface Metric {
  key: string;
  label: string;
  value: string;
}

export interface Guide {
  panel: number;
  points: Vec[];
}

export interface RetopoState {
  mesh: string;
  density: number;
  guides: Guide[];
  metrics: Metric[];
}

export interface JobInfo {
  running: boolean;
  progress: number | null;
  label: string;
}

export interface AnimRange {
  start: number;
  end: number;
  fromCache: boolean;
}

export const EMPTY_UV: UvState = { islands: [], tiles: 1, cols: 1, padding: 0.01 };
export const EMPTY_RETOPO: RetopoState = { mesh: 'triangle', density: 0.5, guides: [], metrics: [] };
