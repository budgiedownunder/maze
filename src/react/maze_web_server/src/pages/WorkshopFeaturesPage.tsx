import { Navigate } from 'react-router-dom'
import { AppHeader } from '../components/AppHeader'
import { useAuth } from '../context/AuthContext'

// Placeholder for the admin-only curated catalogue (featured games +
// collections); the catalogue itself lands in a later step. Guarded here as
// well as hidden from the hub so a non-admin who navigates here directly is
// bounced back rather than shown the section.
export function WorkshopFeaturesPage() {
  const { profile } = useAuth()
  if (!profile?.is_admin) return <Navigate to="/workshop" replace />

  return (
    <div className="games-page">
      <AppHeader title="Manage Features" />
      <main>
        <p>Coming soon.</p>
      </main>
    </div>
  )
}
