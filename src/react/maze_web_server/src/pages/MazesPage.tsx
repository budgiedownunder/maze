import { HamburgerMenu } from '../components/HamburgerMenu'
import { useMenuVariant } from '../hooks/useMenuVariant'
import { useTheme } from '../context/ThemeContext'

export function MazesPage() {
  const menuVariant = useMenuVariant()
  const { theme, toggleTheme } = useTheme()

  return (
    <div>
      <header className="app-header">
        <span className="app-header-title">Maze</span>
        <div className="header-actions">
          <button
            className="theme-toggle"
            onClick={toggleTheme}
            aria-label={theme === 'dark' ? 'Switch to light mode' : 'Switch to dark mode'}
          >
            {theme === 'dark' ? '☀' : '☾'}
          </button>
          {menuVariant === 'hamburger' && <HamburgerMenu />}
        </div>
      </header>
      <main className="page-content">
        <p>Mazes — coming soon</p>
      </main>
    </div>
  )
}
