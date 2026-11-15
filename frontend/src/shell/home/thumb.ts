import { api, thumbChanged } from './api';

const WIDTH = 480;
const HEIGHT = 270;

const encode = (canvas: HTMLCanvasElement, type: string, quality: number): Promise<Blob | null> => new Promise((resolve) => { canvas.toBlob(resolve, type, quality); });

export async function saveThumb(path: string): Promise<boolean> {
  const detail: { out: HTMLCanvasElement | null; w: number; h: number } = { out: null, w: WIDTH, h: HEIGHT };
  document.dispatchEvent(new CustomEvent('ff:thumb', { detail }));
  const canvas = detail.out;
  if (!canvas) return false;
  try {
    let blob = await encode(canvas, 'image/webp', 0.82);
    if (!blob || blob.type !== 'image/webp') blob = await encode(canvas, 'image/jpeg', 0.85);
    if (!blob) return false;
    thumbChanged((await api.putThumb(path, blob)).id);
    return true;
  } catch { return false; }
}
