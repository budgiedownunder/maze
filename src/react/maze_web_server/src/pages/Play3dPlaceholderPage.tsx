import { AppHeader } from '../components/AppHeader'

// A shared "coming soon" stand-in for the Play-3D scope pages not yet built —
// My Games / Shared with me (D4.6) and Community (D4.8). The hub links to all
// four scopes from the outset; this fills the three that arrive later.
export function Play3dPlaceholderPage({ title }: { title: string }) {
  return (
    <div className="games-page">
      <AppHeader title={title} />
      <main className="maze-list-page">
        <p>Coming soon.</p>
      </main>
    </div>
  )
}
