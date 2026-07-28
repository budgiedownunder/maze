import { useEffect, useRef, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { useAuth, useToken } from '../context/AuthContext'
import { useBusyCursor } from '../hooks/useBusyCursor'
import { launchTodaysChallenge } from '../utils/dailyChallenge'
import { AboutModal } from './AboutModal'
import { AlertModal } from './AlertModal'

export function HamburgerMenu() {
  const [open, setOpen] = useState(false)
  const [showAbout, setShowAbout] = useState(false)
  const [resolvingDaily, setResolvingDaily] = useState(false)
  const [dailyError, setDailyError] = useState<string | null>(null)
  const { logout, profile } = useAuth()
  const token = useToken()
  const navigate = useNavigate()
  const menuRef = useRef<HTMLDivElement>(null)
  useBusyCursor(resolvingDaily)

  useEffect(() => {
    function handleClickOutside(e: MouseEvent) {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        setOpen(false)
      }
    }
    if (open) document.addEventListener('mousedown', handleClickOutside)
    return () => document.removeEventListener('mousedown', handleClickOutside)
  }, [open])

  async function handleSignOut() {
    setOpen(false)
    await logout()
    navigate('/login', { replace: true })
  }

  // Launch today's daily challenge (see launchTodaysChallenge) — mirrors the
  // Home tile. The menu closes immediately; a missing challenge or load failure
  // surfaces a friendly alert.
  async function handleTodaysChallenge() {
    setOpen(false)
    if (!token || resolvingDaily) return
    setResolvingDaily(true)
    setDailyError(null)
    try {
      if (!(await launchTodaysChallenge(token))) {
        setDailyError('There is no daily challenge available right now.')
      }
    } catch {
      setDailyError('Could not load today’s challenge. Please try again.')
    } finally {
      setResolvingDaily(false)
    }
  }

  return (
    <>
      <div ref={menuRef} className="menu-container">
        <button
          aria-label="Open menu"
          aria-expanded={open}
          aria-haspopup="menu"
          onClick={() => setOpen(v => !v)}
          className="menu-toggle"
        >
          ☰
        </button>

        {open && (
          <ul role="menu" className="menu-dropdown">
            <li role="none">
              <button role="menuitem" className="menu-item" onClick={() => { setOpen(false); navigate('/') }}>
                Home
              </button>
            </li>
            <li role="separator" className="menu-separator" />
            <li role="none">
              <button role="menuitem" className="menu-item" onClick={handleTodaysChallenge}>
                Today's Challenge
              </button>
            </li>
            <li role="none">
              <button role="menuitem" className="menu-item" onClick={() => { setOpen(false); navigate('/play-3d') }}>
                3D Games
              </button>
            </li>
            <li role="none">
              <button role="menuitem" className="menu-item menu-subitem" onClick={() => { setOpen(false); navigate('/play-3d/featured') }}>
                Featured
              </button>
            </li>
            <li role="none">
              <button role="menuitem" className="menu-item menu-subitem" onClick={() => { setOpen(false); navigate('/play-3d/my-games') }}>
                My Games
              </button>
            </li>
            <li role="none">
              <button role="menuitem" className="menu-item menu-subitem" onClick={() => { setOpen(false); navigate('/play-3d/shared') }}>
                Shared with me
              </button>
            </li>
            <li role="none">
              <button role="menuitem" className="menu-item menu-subitem" onClick={() => { setOpen(false); navigate('/play-3d/community') }}>
                Community
              </button>
            </li>
            <li role="none">
              <button role="menuitem" className="menu-item" onClick={() => { setOpen(false); navigate('/workshop') }}>
                3D Game Workshop
              </button>
            </li>
            <li role="none">
              <button role="menuitem" className="menu-item menu-subitem" onClick={() => { setOpen(false); navigate('/workshop/games') }}>
                Manage Games
              </button>
            </li>
            <li role="none">
              <button role="menuitem" className="menu-item menu-subitem" onClick={() => { setOpen(false); navigate('/workshop/game-collections') }}>
                Manage Game Collections
              </button>
            </li>
            {profile?.is_admin && (
              <li role="none">
                <button role="menuitem" className="menu-item menu-subitem" onClick={() => { setOpen(false); navigate('/workshop/features') }}>
                  Manage Features
                </button>
              </li>
            )}
            <li role="none">
              <button role="menuitem" className="menu-item" onClick={() => { setOpen(false); navigate('/mazes') }}>
                Mazes
              </button>
            </li>
            <li role="none">
              <button role="menuitem" className="menu-item" onClick={() => { setOpen(false); navigate('/leaderboards') }}>
                Leaderboards
              </button>
            </li>
            <li role="separator" className="menu-separator" />
            <li role="none">
              <button role="menuitem" className="menu-item" onClick={() => { setOpen(false); navigate('/account') }}>
                My Account
              </button>
            </li>
            <li role="none">
              <button role="menuitem" className="menu-item" onClick={handleSignOut}>
                Sign Out
              </button>
            </li>
            <li role="separator" className="menu-separator" />
            <li role="none">
              <button role="menuitem" className="menu-item" onClick={() => { setOpen(false); setShowAbout(true) }}>
                About
              </button>
            </li>
          </ul>
        )}
      </div>

      {showAbout && <AboutModal onClose={() => setShowAbout(false)} />}
      {dailyError && (
        <AlertModal title="Daily Challenge" message={dailyError} onClose={() => setDailyError(null)} />
      )}
    </>
  )
}
