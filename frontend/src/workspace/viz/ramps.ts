export type RampName = 'viridis' | 'inferno' | 'coolwarm';

const STOPS: Record<RampName, string[]> = {
  viridis: ['#440154', '#482878', '#3e4989', '#31688e', '#26828e', '#1f9e89', '#35b779', '#6ece58', '#b5de2b', '#fde725'],
  inferno: ['#000004', '#1b0c41', '#4a0c6b', '#781c6d', '#a52c60', '#cf4446', '#ed6925', '#fb9b06', '#f7d13d', '#fcffa4'],
  coolwarm: ['#3b4cc0', '#6788ee', '#9abbff', '#c9d7f0', '#edd1c2', '#f7a889', '#e26952', '#b40426'],
};

const parsed = new Map<RampName, number[][]>();

function stops(name: RampName): number[][] {
  let p = parsed.get(name);
  if (!p) {
    p = STOPS[name].map((h) => {
      const n = parseInt(h.slice(1), 16);
      return [(n >> 16) & 255, (n >> 8) & 255, n & 255];
    });
    parsed.set(name, p);
  }
  return p;
}

export function rampBytes(name: RampName, size: number): Uint8Array {
  const s = stops(name);
  const out = new Uint8Array(size * 4);
  for (let i = 0; i < size; i++) {
    const x = (i / (size - 1)) * (s.length - 1);
    const k = Math.min(s.length - 2, Math.floor(x));
    const f = x - k;
    for (let c = 0; c < 3; c++) out[i * 4 + c] = Math.round(s[k][c] * (1 - f) + s[k + 1][c] * f);
    out[i * 4 + 3] = 255;
  }
  return out;
}

export function cssGradient(name: RampName): string {
  const s = STOPS[name];
  return `linear-gradient(to right, ${s.map((c, i) => `${c} ${Math.round((i / (s.length - 1)) * 100)}%`).join(', ')})`;
}

export const MAP_RAMP: Record<string, RampName> = {
  strain: 'viridis',
  stress: 'inferno',
  pressure: 'inferno',
  fit: 'coolwarm',
  contact: 'inferno',
  thickness: 'viridis',
  compare: 'coolwarm',
};

export const MAP_UNIT: Record<string, string> = {
  strain: '%',
  stress: 'kPa',
  pressure: 'kPa',
  fit: 'mm',
  contact: 'flag',
  thickness: 'mm',
  compare: 'mm',
};
