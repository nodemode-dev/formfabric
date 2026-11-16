export class Selection {
  panels = new Set<number>();
  pts: { panel: number; idx: number[] } | null = null;
  seg: { panel: number; i: number } | null = null;
  inner: { panel: number; id: number } | null = null;
  seam: number | null = null;
  tack: number | null = null;

  clear(): void {
    this.panels.clear();
    this.pts = null;
    this.seg = null;
    this.inner = null;
    this.seam = null;
    this.tack = null;
  }

  only(id: number): void {
    this.panels.clear();
    this.panels.add(id);
    this.pts = null;
    this.seg = null;
    this.seam = null;
    this.tack = null;
    if (this.inner && this.inner.panel !== id) this.inner = null;
  }

  list(): number[] {
    return [...this.panels];
  }

  private lens = new Map<number, number>();

  prune(ids: Set<number>, seams?: Set<number>, tacks?: Set<number>, lens?: Map<number, number>): void {
    if (lens) {
      const moved = (panel: number): boolean => { const was = this.lens.get(panel); return was !== undefined && was !== lens.get(panel); };
      if (this.pts && moved(this.pts.panel)) this.pts = null;
      if (this.seg && moved(this.seg.panel)) this.seg = null;
      this.lens = new Map(lens);
    }
    if (seams && this.seam !== null && !seams.has(this.seam)) this.seam = null;
    if (tacks && this.tack !== null && !tacks.has(this.tack)) this.tack = null;
    for (const id of [...this.panels]) if (!ids.has(id)) this.panels.delete(id);
    if (this.pts && !ids.has(this.pts.panel)) this.pts = null;
    if (this.seg && !ids.has(this.seg.panel)) this.seg = null;
    if (this.inner && !ids.has(this.inner.panel)) this.inner = null;
  }
}

export const UNIT_MM: Record<string, number> = { mm: 1, cm: 10, in: 25.4 };
const DECIMALS: Record<string, number> = { mm: 1, cm: 2, in: 3 };

export function toUnit(mm: number, unit: string): number {
  return mm / (UNIT_MM[unit] ?? 1);
}

export function fromUnit(v: number, unit: string): number {
  return v * (UNIT_MM[unit] ?? 1);
}

export function fmtNum(v: number, digits: number): string {
  const s = v.toFixed(digits);
  return s === '-0' || /^-0\.0*$/.test(s) ? s.slice(1) : s;
}

export function fmtLen(mm: number, unit: string, withUnit = true): string {
  return fmtNum(toUnit(mm, unit), DECIMALS[unit] ?? 1) + (withUnit ? ` ${unit}` : '');
}

export function fmtArea(mm2: number, unit: string): string {
  const f = UNIT_MM[unit] ?? 1;
  return `${fmtNum(mm2 / (f * f), unit === 'mm' ? 0 : unit === 'cm' ? 1 : 2)} ${unit}2`;
}

export function decimals(unit: string): number {
  return DECIMALS[unit] ?? 1;
}
