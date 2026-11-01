import type { ShellDeps } from '../shell/deps';

export function mountWorkspace(host: HTMLElement, deps: ShellDeps): void {
  void deps;
  host.dataset.layout = deps.layout.get();
  host.innerHTML = '<div class="cv-workspace-reserve"></div>';
}
