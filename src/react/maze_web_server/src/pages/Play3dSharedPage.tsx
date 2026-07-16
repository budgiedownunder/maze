import { Play3dScopeBrowser } from '../components/Play3dScopeBrowser'

// The "Shared with me" Play-3D page: games + collections other users have shared
// with the caller (not their own, not public/curated), each a tab on the shared
// scope browser.
export function Play3dSharedPage() {
  return <Play3dScopeBrowser scope="shared" title="Shared with me" />
}
