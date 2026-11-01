import type { Catalogue, EngineChanged } from '../engine/service';
import type { Commands } from '../ui/commands';
import type { Prefs } from '../ui/prefs';
import type { StatusBar } from './statusbar';
import type { LayoutState } from './layout';

export interface EngineApi {
  call<T>(name: string, args?: object): Promise<T>;
  catalogue(): Promise<Catalogue>;
  onEngineChanged(cb: (e: EngineChanged) => void): () => void;
}

export interface ShellDeps {
  commands: Commands;
  engine: EngineApi;
  status: StatusBar;
  layout: LayoutState;
  toast: (text: string, kind?: 'info' | 'error') => void;
  prefs: () => Prefs;
  onPrefs: (cb: (p: Prefs, keys: string[]) => void) => void;
}
