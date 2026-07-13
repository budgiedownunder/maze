import { useNavigate } from 'react-router-dom'
import { AppHeader } from '../components/AppHeader'
import { useAuth } from '../context/AuthContext'

// The 3D Game Workshop hub: a signpost to the management sub-areas. Games and
// Collections are open to any signed-in user; the Features (curated catalogue)
// area is shown only to admins.
export function WorkshopHubPage() {
  const navigate = useNavigate()
  const { profile } = useAuth()

  return (
    <div className="home-page">
      <AppHeader title="3D Game Workshop" />
      <main className="home-main">
        <section className="home-tiles">
          <button type="button" className="home-tile" onClick={() => navigate('/workshop/games')}>
            <img src="/images/workshop/workshop-games.svg" className="home-tile-img" alt="" aria-hidden="true" />
            <div className="home-tile-text">
              <h2 className="home-tile-title">Manage Games</h2>
              <p className="home-tile-desc">Create, edit, publish and share your 3D games</p>
            </div>
          </button>
          <button type="button" className="home-tile" onClick={() => navigate('/workshop/game-collections')}>
            <img src="/images/workshop/workshop-game-collections.svg" className="home-tile-img" alt="" aria-hidden="true" />
            <div className="home-tile-text">
              <h2 className="home-tile-title">Manage Game Collections</h2>
              <p className="home-tile-desc">Group your games into ordered collections</p>
            </div>
          </button>
          {profile?.is_admin && (
            <button type="button" className="home-tile" onClick={() => navigate('/workshop/features')}>
              <img src="/images/workshop/workshop-features.svg" className="home-tile-img" alt="" aria-hidden="true" />
              <div className="home-tile-text">
                <h2 className="home-tile-title">Manage Features [Admin]</h2>
                <p className="home-tile-desc">Manage the featured games and collections everyone sees</p>
              </div>
            </button>
          )}
        </section>
      </main>
    </div>
  )
}
