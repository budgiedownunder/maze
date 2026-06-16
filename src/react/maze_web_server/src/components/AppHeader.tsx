import type { ReactNode } from 'react'
import { useNavigate } from 'react-router-dom'
import { HamburgerMenu } from './HamburgerMenu'
import { Avatar } from './Avatar'
import { useMenuVariant } from '../hooks/useMenuVariant'
import { useTheme } from '../context/ThemeContext'
import { useAuth } from '../context/AuthContext'

interface AppHeaderProps {
  /** The header title — plain text or a node (e.g. with an inline icon). */
  title: ReactNode
  /** Optional image shown before the title (e.g. the Leaderboards trophy). */
  titleIcon?: string
  /** Element to render the title as. Defaults to `span`; pages that want the
   *  title to be the document heading pass `h1`. */
  titleAs?: 'span' | 'h1'
  /** Page-specific action buttons, rendered on the right before the username
   *  and theme toggle (e.g. a Save / Refresh / New button). */
  children?: ReactNode
}

/**
 * The shared app chrome shown at the top of every signed-in page: the hamburger
 * menu, the page title, any page-specific actions, the signed-in username
 * (linking to the account page), and the light/dark theme toggle.
 */
export function AppHeader({ title, titleIcon, titleAs = 'span', children }: AppHeaderProps) {
  const menuVariant = useMenuVariant()
  const { theme, toggleTheme } = useTheme()
  const { isAuthenticated, profile } = useAuth()
  const navigate = useNavigate()

  const TitleTag = titleAs
  const themeLabel = theme === 'dark' ? 'Switch to light mode' : 'Switch to dark mode'

  return (
    <header className="app-header">
      <div className="header-actions">
        {menuVariant === 'hamburger' && <HamburgerMenu />}
      </div>
      <TitleTag className={`app-header-title${titleIcon ? ' app-header-title--with-icon' : ''}`}>
        {titleIcon && (
          <img src={titleIcon} className="app-header-title-icon" alt="" aria-hidden="true" />
        )}
        {title}
      </TitleTag>
      <div className="header-actions">
        {children}
        <button
          className="theme-toggle"
          onClick={toggleTheme}
          aria-label={themeLabel}
          title={themeLabel}
        >
          {theme === 'dark' ? '☀' : '☾'}
        </button>
        {isAuthenticated && profile && (
          <button
            type="button"
            className="header-avatar-link"
            onClick={() => navigate('/account')}
            title={profile.username}
          >
            <Avatar
              userId={profile.id}
              avatarUpdatedAt={profile.avatar_updated_at}
              size={28}
              alt={profile.username}
            />
          </button>
        )}
      </div>
    </header>
  )
}
