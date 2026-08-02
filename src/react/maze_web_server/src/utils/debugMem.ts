/// Whether this build asks the hosted `/game/` page for its diagnostics readout.
/// Set by the `dev:debug` / `build:debug` scripts (`VITE_DEBUG_MEM=true`); any
/// ordinary build leaves it false, and the `withDebugMem` calls below fold away.
///
/// A development-only switch: it is never passed when building for deployment.
/// Note this deliberately does not also test `import.meta.env.DEV`, unlike the
/// MSW guard in `main.tsx` — `vite build` sets `DEV` false, and a debug `dist/`
/// served by a local `maze_web_server` is exactly the case this exists for
/// (testing on a phone against the dev machine).
export const DEBUG_MEM = import.meta.env.VITE_DEBUG_MEM === 'true'

/// Appends the host page's `mem=1` parameter when this is a debug build, picking
/// the right separator for a URL that may already carry a query string. Returns
/// the URL untouched otherwise, so a normal build launches exactly as before.
///
/// The host page also reads `?mem=1` typed by hand, so a normal build can still
/// be switched on per-launch by editing the address bar and reloading.
export function withDebugMem(url: string): string {
  if (!DEBUG_MEM) return url
  return url + (url.includes('?') ? '&' : '?') + 'mem=1'
}
