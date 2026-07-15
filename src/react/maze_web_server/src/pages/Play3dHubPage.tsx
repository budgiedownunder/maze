import { useNavigate } from 'react-router-dom'
import { AppHeader } from '../components/AppHeader'

// The four browse scopes, in order. Featured is live; the other three link to a
// "coming soon" placeholder until D4.6 (My Games / Shared) and D4.8 (Community).
const TILES = [
  { to: '/play-3d/featured', img: '/images/workshop/workshop-features.svg', title: 'Featured', desc: 'Games and collections featured for everyone' },
  { to: '/play-3d/my-games', img: '/images/workshop/workshop-my-games.svg', title: 'My Games', desc: 'Your own 3D games and collections' },
  { to: '/play-3d/shared', img: '/images/workshop/workshop-shared.svg', title: 'Shared with me', desc: 'Games and collections others have shared with you' },
  { to: '/play-3d/community', img: '/images/workshop/workshop-community.svg', title: 'Community', desc: 'Public games and collections from everyone' },
] as const

// The 3D Games hub: a signpost to the four browse scopes, mirroring the Workshop
// hub. Open to any signed-in user (no admin gate — these are consumer views).
export function Play3dHubPage() {
  const navigate = useNavigate()
  return (
    <div className="home-page">
      <AppHeader title="3D Games" />
      <main className="home-main">
        <section className="home-tiles">
          {TILES.map(tile => (
            <button key={tile.to} type="button" className="home-tile" onClick={() => navigate(tile.to)}>
              <img src={tile.img} className="home-tile-img" alt="" aria-hidden="true" />
              <div className="home-tile-text">
                <h2 className="home-tile-title">{tile.title}</h2>
                <p className="home-tile-desc">{tile.desc}</p>
              </div>
            </button>
          ))}
        </section>
      </main>
    </div>
  )
}
