import { useNavigate } from 'react-router-dom'
import { AppHeader } from '../components/AppHeader'
import appIcon from '../assets/app.png'
import play3dIcon from '../assets/play3d.png'
import workshopIcon from '../assets/workshop.svg'
import leaderboardsIcon from '../assets/leaderboards.svg'

export function HomePage() {
  const navigate = useNavigate()

  function handle3dGames() {
    navigate('/play-3d')
  }

  function handleWorkshop() {
    navigate('/workshop')
  }

  function handleMyMazes() {
    navigate('/mazes')
  }

  function handleLeaderboards() {
    navigate('/leaderboards')
  }

  return (
    <div className="home-page">
      <AppHeader title="Home" />
      <main className="home-main">
        <section className="home-tiles">
          <button type="button" className="home-tile" onClick={handle3dGames}>
            <img src={play3dIcon} className="home-tile-img home-tile-img--photo" alt="" aria-hidden="true" />
            <div className="home-tile-text">
              <h2 className="home-tile-title">3D Games</h2>
              <p className="home-tile-desc">Browse and play 3D games</p>
            </div>
          </button>
          <button type="button" className="home-tile" onClick={handleWorkshop}>
            <img src={workshopIcon} className="home-tile-img" alt="" aria-hidden="true" />
            <div className="home-tile-text">
              <h2 className="home-tile-title">3D Game Workshop</h2>
              <p className="home-tile-desc">Create, publish and share your own 3D games and collections</p>
            </div>
          </button>
          <button type="button" className="home-tile" onClick={handleMyMazes}>
            <img src={appIcon} className="home-tile-img home-tile-img--photo" alt="" aria-hidden="true" />
            <div className="home-tile-text">
              <h2 className="home-tile-title">Mazes</h2>
              <p className="home-tile-desc">Design and play your own single-level mazes</p>
            </div>
          </button>
          <button type="button" className="home-tile" onClick={handleLeaderboards}>
            <img src={leaderboardsIcon} className="home-tile-img" alt="" aria-hidden="true" />
            <div className="home-tile-text">
              <h2 className="home-tile-title">Leaderboards</h2>
              <p className="home-tile-desc">See your times and how you rank</p>
            </div>
          </button>
        </section>
      </main>
    </div>
  )
}
