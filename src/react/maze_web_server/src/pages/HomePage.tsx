import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { HamburgerMenu } from '../components/HamburgerMenu'
import { Play3dDifficultyModal } from '../components/Play3dDifficultyModal'
import { useMenuVariant } from '../hooks/useMenuVariant'
import { useTheme } from '../context/ThemeContext'
import appIcon from '../assets/app.png'
import play3dIcon from '../assets/play3d.png'
import scoresIcon from '../assets/scores.svg'

export function HomePage() {
  const menuVariant = useMenuVariant()
  const { theme, toggleTheme } = useTheme()
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

  function handleMyMazes() {
    navigate('/mazes')
  }

  function handleScores() {
    navigate('/scores')
  }

  return (
    <div className="home-page">
      <header className="app-header">
        <div className="header-actions">
          {menuVariant === 'hamburger' && <HamburgerMenu />}
        </div>
        <span className="app-header-title">Home</span>
        <div className="header-actions">
          <button
            className="theme-toggle"
            onClick={toggleTheme}
            aria-label={theme === 'dark' ? 'Switch to light mode' : 'Switch to dark mode'}
            title={theme === 'dark' ? 'Switch to light mode' : 'Switch to dark mode'}
          >
            {theme === 'dark' ? '☀' : '☾'}
          </button>
        </div>
      </header>
      <main className="home-main">
        <section className="home-tiles">
          <button type="button" className="home-tile" onClick={handlePlay3d}>
            <img src={play3dIcon} className="home-tile-img home-tile-img--photo" alt="" aria-hidden="true" />
            <div className="home-tile-text">
              <h2 className="home-tile-title">Play 3D</h2>
              <p className="home-tile-desc">Play in easy, tricky or hard mode</p>
            </div>
          </button>
          <button type="button" className="home-tile" onClick={handleMyMazes}>
            <img src={appIcon} className="home-tile-img home-tile-img--photo" alt="" aria-hidden="true" />
            <div className="home-tile-text">
              <h2 className="home-tile-title">Mazes</h2>
              <p className="home-tile-desc">Design and play your own mazes</p>
            </div>
          </button>
          <button type="button" className="home-tile" onClick={handleScores}>
            <img src={scoresIcon} className="home-tile-img" alt="" aria-hidden="true" />
            <div className="home-tile-text">
              <h2 className="home-tile-title">Scores</h2>
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
