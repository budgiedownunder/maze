import { Play3dScopeBrowser } from '../components/Play3dScopeBrowser'

// The "My Games" Play-3D page: the caller's own games + collections (any
// visibility), each a tab on the shared scope browser.
export function Play3dMyGamesPage() {
  return <Play3dScopeBrowser scope="mine" title="My Games" />
}
