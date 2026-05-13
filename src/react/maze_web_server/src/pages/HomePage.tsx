import { useNavigate } from 'react-router-dom'
import { HamburgerMenu } from '../components/HamburgerMenu'
import { useMenuVariant } from '../hooks/useMenuVariant'
import { useTheme } from '../context/ThemeContext'
import appIcon from '../assets/app.png'
import play3dIcon from '../assets/play3d.png'

export function HomePage() {
  const menuVariant = useMenuVariant()
  const { theme, toggleTheme } = useTheme()
  const navigate = useNavigate()

  function handlePlay3d() {
    // No id => Bevy starts a random maze, mirroring the MAUI flyout entry.
    window.location.href = '/game/'
  }

  function handleMyMazes() {
    navigate('/mazes')
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
            <img src={play3dIcon} className="home-tile-img" alt="" aria-hidden="true" />
            <div className="home-tile-text">
              <h2 className="home-tile-title">Play 3D</h2>
              <p className="home-tile-desc">Play a random 3D game</p>
            </div>
          </button>
          <button type="button" className="home-tile" onClick={handleMyMazes}>
            <img src={appIcon} className="home-tile-img" alt="" aria-hidden="true" />
            <div className="home-tile-text">
              <h2 className="home-tile-title">Design &amp; Play</h2>
              <p className="home-tile-desc">Design and play your own mazes</p>
            </div>
          </button>
        </section>
      </main>
    </div>
  )
}
