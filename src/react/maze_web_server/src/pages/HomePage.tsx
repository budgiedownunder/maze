import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { AppHeader } from '../components/AppHeader'
import { Play3dDifficultyModal } from '../components/Play3dDifficultyModal'
import appIcon from '../assets/app.png'
import play3dIcon from '../assets/play3d.png'
import workshopIcon from '../assets/workshop.svg'
import leaderboardsIcon from '../assets/leaderboards.svg'

export function HomePage() {
  const navigate = useNavigate()
  const [showDifficultyModal, setShowDifficultyModal] = useState(false)

  function handlePlay3d() {
    setShowDifficultyModal(true)
  }

  function startPlay3d(difficulty: string) {
    // `/game/` is the standalone Bevy/WASM page, not a React route — a full
    // page navigation is required. The server maps `?difficulty=` to a preset.
    window.location.href = `/game/?difficulty=${encodeURIComponent(difficulty)}`
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
          <button type="button" className="home-tile" onClick={handlePlay3d}>
            <img src={play3dIcon} className="home-tile-img home-tile-img--photo" alt="" aria-hidden="true" />
            <div className="home-tile-text">
              <h2 className="home-tile-title">Play 3D</h2>
              <p className="home-tile-desc">Play single-level and multi-level 3D games in easy, tricky or hard mode</p>
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
      {showDifficultyModal && (
        <Play3dDifficultyModal
          onPlay={startPlay3d}
          onCancel={() => setShowDifficultyModal(false)}
        />
      )}
    </div>
  )
}
