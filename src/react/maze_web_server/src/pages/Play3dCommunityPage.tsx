import { Play3dScopeBrowser } from '../components/Play3dScopeBrowser'

// The "Community" Play-3D page: games + collections other users have published
// for everyone. The one unbounded catalogue — it searches server-side and offers
// a sort, rather than being loaded whole.
export function Play3dCommunityPage() {
  return <Play3dScopeBrowser scope="public" title="Community" />
}
