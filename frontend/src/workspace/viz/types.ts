export type LightKind = 'directional' | 'point' | 'spot' | 'area';

export interface LightJson {
  id: number;
  kind: LightKind;
  position: [number, number, number];
  direction: [number, number, number];
  color: [number, number, number];
  intensity: number;
  shadows: boolean;
}

export interface StageJson {
  lights: LightJson[];
  hdri: string | null;
  hdri_rotation: number;
  exposure: number;
  ground: boolean;
  background: unknown;
}

export interface SavedViewJson {
  id: number;
  name: string;
  camera: unknown;
}

export interface FurJson {
  id: number;
  panel: number;
  length: number;
  density: number;
  thickness: number;
  curl: number;
  color: [number, number, number];
}

export interface VizJson {
  stage: StageJson;
  views: SavedViewJson[];
  fur: FurJson[];
  map: string | null;
  compare: unknown;
}

export const EMPTY_STAGE: StageJson = { lights: [], hdri: null, hdri_rotation: 0, exposure: 0, ground: false, background: null };

export const EMPTY_VIZ: VizJson = { stage: EMPTY_STAGE, views: [], fur: [], map: null, compare: null };

export function vizOf(garment: unknown): VizJson {
  const v = (garment as { viz?: Partial<VizJson> } | null)?.viz;
  if (!v) return EMPTY_VIZ;
  return {
    stage: { ...EMPTY_STAGE, ...(v.stage ?? {}), lights: v.stage?.lights ?? [] },
    views: v.views ?? [],
    fur: v.fur ?? [],
    map: v.map ?? null,
    compare: v.compare ?? null,
  };
}

export interface MeasureJson {
  id: number;
  kind: 'length' | 'circumference' | 'distance';
  points: Array<[number, number, number]>;
  value?: number;
  lines?: Array<Array<[number, number, number]>>;
  height?: number;
}
