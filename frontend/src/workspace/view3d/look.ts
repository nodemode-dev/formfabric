import type { ColorwayJson, FabricJson, GarmentFull, PanelFull, RGB, SurfaceJson } from './model';

export interface PanelLook {
  front: RGB;
  back: RGB;
  rough: number;
  sheen: number;
  map: string | null;
  mapScale: number;
  mapRot: number;
  mapOff: [number, number];
}

const NEUTRAL_FRONT: RGB = [0.62, 0.63, 0.66];
const BACK_DARKEN = 0.72;

export function parseColor(value: unknown): RGB | null {
  if (typeof value === 'string') {
    const m = /^#?([0-9a-f]{6})([0-9a-f]{2})?$/i.exec(value.trim());
    if (!m) return null;
    const n = parseInt(m[1], 16);
    return [(n >> 16) / 255, ((n >> 8) & 255) / 255, (n & 255) / 255];
  }
  if (Array.isArray(value) && value.length >= 3 && value.every((v, i) => i > 2 || typeof v === 'number')) {
    const big = Math.max(value[0], value[1], value[2]) > 1;
    const k = big ? 1 / 255 : 1;
    return [value[0] * k, value[1] * k, value[2] * k];
  }
  if (value && typeof value === 'object') {
    const o = value as Record<string, unknown>;
    if (typeof o.r === 'number' && typeof o.g === 'number' && typeof o.b === 'number') return parseColor([o.r, o.g, o.b]);
    if ('color' in o) return parseColor(o.color);
  }
  return null;
}

export function linear(c: RGB): RGB {
  return [Math.pow(c[0], 2.2), Math.pow(c[1], 2.2), Math.pow(c[2], 2.2)];
}

function firstMap(maps: unknown): string | null {
  if (typeof maps === 'string') return maps || null;
  if (Array.isArray(maps)) return maps.find((m): m is string => typeof m === 'string' && m.length > 0) ?? null;
  if (maps && typeof maps === 'object') {
    const o = maps as Record<string, unknown>;
    for (const k of ['base', 'color', 'albedo', 'diffuse']) if (typeof o[k] === 'string' && o[k]) return o[k] as string;
  }
  return null;
}

function overrideFor(colorway: ColorwayJson | undefined, fabric: number): { front?: RGB; back?: RGB } | null {
  if (!colorway) return null;
  const fc = colorway.fabric_colors;
  let entry: unknown;
  if (Array.isArray(fc)) {
    for (const e of fc) {
      if (Array.isArray(e) && e[0] === fabric) entry = e[1];
      else if (e && typeof e === 'object' && ((e as { fabric?: number }).fabric === fabric || (e as { id?: number }).id === fabric)) entry = e;
    }
  } else if (fc && typeof fc === 'object') entry = (fc as Record<string, unknown>)[String(fabric)];
  if (entry === undefined || entry === null) return null;
  if (entry && typeof entry === 'object' && !Array.isArray(entry) && ('front' in entry || 'back' in entry)) {
    const o = entry as { front?: unknown; back?: unknown };
    const out: { front?: RGB; back?: RGB } = {};
    const f = parseColor(o.front);
    const b = parseColor(o.back);
    if (f) out.front = f;
    if (b) out.back = b;
    return out;
  }
  const c = parseColor(entry);
  return c ? { front: c } : null;
}

export function lookOf(g: GarmentFull, p: PanelFull): PanelLook {
  const fabrics = g.fabrics ?? [];
  const wanted = p.fabric ?? g.default_fabric ?? null;
  const fabric: FabricJson | undefined = fabrics.find((f) => f.id === wanted) ?? fabrics[0];
  const front: SurfaceJson | undefined = fabric?.look?.front;
  const back: SurfaceJson | undefined = fabric?.look?.back;
  const colorway = (g.colorways ?? []).find((c) => c.id === g.active_colorway);
  const over = fabric ? overrideFor(colorway, fabric.id) : null;
  const fc = over?.front ?? parseColor(front?.color) ?? NEUTRAL_FRONT;
  const bc = over?.back ?? (over?.front ? null : parseColor(back?.color)) ?? [fc[0] * BACK_DARKEN, fc[1] * BACK_DARKEN, fc[2] * BACK_DARKEN];
  const off = front?.map_offset;
  return {
    front: fc,
    back: bc,
    rough: clamp01(front?.roughness ?? 0.85),
    sheen: clamp01(front?.sheen ?? 0.2),
    map: firstMap(front?.maps),
    mapScale: front?.map_scale && front.map_scale > 0 ? front.map_scale : 200,
    mapRot: ((front?.map_rotation ?? 0) * Math.PI) / 180,
    mapOff: [off?.[0] ?? 0, off?.[1] ?? 0],
  };
}

function clamp01(v: number): number {
  return Math.max(0, Math.min(1, v));
}

export function hueColor(id: number): RGB {
  const h = ((id * 137.508) % 360) / 60;
  const s = 0.55;
  const l = 0.58;
  const c = (1 - Math.abs(2 * l - 1)) * s;
  const x = c * (1 - Math.abs((h % 2) - 1));
  const m = l - c / 2;
  const sector = Math.floor(h) % 6;
  const table: RGB[] = [[c, x, 0], [x, c, 0], [0, c, x], [0, x, c], [x, 0, c], [c, 0, x]];
  const r = table[sector];
  return [r[0] + m, r[1] + m, r[2] + m];
}
