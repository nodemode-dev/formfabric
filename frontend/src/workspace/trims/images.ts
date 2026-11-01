import { previewUrl } from '../fabric/files';

type Entry = { img: HTMLImageElement | null; failed: boolean };

const cache = new Map<string, Entry>();
let ready: () => void = () => undefined;

export function onImageReady(cb: () => void): void {
  ready = cb;
}

async function urlFor(path: string): Promise<string | null> {
  const native = await previewUrl(path);
  if (native) return native;
  return import.meta.env.DEV && path.startsWith('/') ? `/@fs${path}` : null;
}

export function imageFor(path: string): HTMLImageElement | null {
  if (!path) return null;
  const hit = cache.get(path);
  if (hit) return hit.img;
  const entry: Entry = { img: null, failed: false };
  cache.set(path, entry);
  void urlFor(path).then((url) => {
    if (!url) { entry.failed = true; return; }
    const img = new Image();
    img.decoding = 'async';
    img.onload = () => { entry.img = img; ready(); };
    img.onerror = () => { entry.failed = true; };
    img.src = url;
  });
  return null;
}

export function imageFailed(path: string): boolean {
  return cache.get(path)?.failed === true;
}
