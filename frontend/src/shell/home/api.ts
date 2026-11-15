import { serviceUrl } from '../../engine/service';

export interface GarmentSummary { id: string; path: string; name: string; title: string; bytes: number; modified: string; panels: number | null; thumb: boolean; library: boolean }
export interface TrashedGarment { entry: string; id: string; title: string; name: string; panels: number | null; deletedAt: number; thumb: boolean }
export interface TemplateSummary { id: string; name: string; category: string; tags: string[]; description: string; source: string; panels: number | null; thumb: boolean }
export interface Library { garments: GarmentSummary[]; library: string; untitled: string; pinned: string[]; recentHidden: string[] }
export interface Templates { categories: string[]; templates: TemplateSummary[]; default: string }
export interface HomeSettings { pinned: string[]; recentHidden: string[] }
export type Imported = { kind: 'garment'; path: string; garment: GarmentSummary } | { kind: 'dxf'; path: string };

async function request<T>(method: string, path: string, body?: BodyInit, json = false): Promise<T> {
  const response = await fetch(serviceUrl(path), { method, cache: 'no-store', headers: json ? { 'Content-Type': 'application/json' } : undefined, body });
  if (!response.ok) {
    let message = response.statusText || `HTTP ${response.status}`;
    try {
      const detail = ((await response.json()) as { detail?: unknown }).detail;
      if (typeof detail === 'string') message = detail;
    } catch { void 0; }
    throw new Error(message);
  }
  return (await response.json()) as T;
}

const id = (s: string): string => encodeURIComponent(s);

export const api = {
  library: (): Promise<Library> => request<Library>('GET', '/api/garments'),
  templates: (): Promise<Templates> => request<Templates>('GET', '/api/garment-templates'),
  trash: (): Promise<{ garments: TrashedGarment[] }> => request('GET', '/api/garments/trash'),
  create: (template: string, title?: string): Promise<GarmentSummary> => request('POST', '/api/garments', JSON.stringify({ template, title }), true),
  duplicate: (gid: string): Promise<GarmentSummary> => request('POST', `/api/garments/${id(gid)}/duplicate`),
  forget: (gid: string): Promise<{ ok: boolean }> => request('POST', `/api/garments/${id(gid)}/forget`),
  remove: (gid: string): Promise<{ ok: boolean; entry: string }> => request('DELETE', `/api/garments/${id(gid)}`),
  restoreTrashed: (entry: string): Promise<GarmentSummary> => request('POST', `/api/garments/trash/${id(entry)}/restore`),
  purgeTrashed: (entry: string): Promise<{ ok: boolean }> => request('DELETE', `/api/garments/trash/${id(entry)}`),
  emptyTrash: (): Promise<{ ok: boolean; removed: number }> => request('DELETE', '/api/garments/trash'),
  setSettings: (patch: Partial<HomeSettings>): Promise<HomeSettings> => request('POST', '/api/garments/settings', JSON.stringify(patch), true),
  importFile: (file: File): Promise<Imported> => request('POST', `/api/garments/import?name=${id(file.name)}`, file),
  putThumb: (path: string, picture: Blob): Promise<{ ok: boolean; id: string }> => request('PUT', `/api/garments/thumb?path=${id(path)}`, picture),
};

const pictures = new Map<string, Promise<string>>();

export function picture(path: string): Promise<string> {
  let held = pictures.get(path);
  if (!held) {
    held = fetch(serviceUrl(path), { cache: 'no-store' }).then(async (response) => {
      if (!response.ok) throw new Error(`HTTP ${response.status}`);
      return URL.createObjectURL(await response.blob());
    });
    pictures.set(path, held);
    held.catch(() => pictures.delete(path));
    if (pictures.size > 400) {
      const first = pictures.keys().next().value;
      if (first !== undefined && first !== path) { void pictures.get(first)?.then((url) => URL.revokeObjectURL(url)).catch(() => undefined); pictures.delete(first); }
    }
  }
  return held;
}

const revisions = new Map<string, number>();

export function thumbChanged(gid: string): void { revisions.set(gid, (revisions.get(gid) ?? 0) + 1); }

export const garmentThumb = (g: { id: string; modified: string }): string => `/api/garments/${id(g.id)}/thumb?v=${id(g.modified)}&r=${revisions.get(g.id) ?? 0}`;
export const templateThumb = (t: TemplateSummary): string => `/api/garment-templates/${id(t.id)}/thumb`;
