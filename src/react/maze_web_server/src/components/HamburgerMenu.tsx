import { useEffect, useRef, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { useAuth } from '../context/AuthContext'
import { AboutModal } from './AboutModal'
import { AccountModal } from './AccountModal'

export function HamburgerMenu() {
  const [open, setOpen] = useState(false)
  const [showAbout, setShowAbout] = useState(false)
  const [showAccount, setShowAccount] = useState(false)
  const { logout } = useAuth()
  const navigate = useNavigate()
  const menuRef = useRef<HTMLDivElement>(null)

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
              <button role="menuitem" className="menu-item" onClick={() => { setOpen(false); setShowAccount(true) }}>
                My Account...
              </button>
            </li>
            <li role="none">
              <button role="menuitem" className="menu-item" onClick={handleSignOut}>
                Sign Out
              </button>
            </li>
            <li role="none">
              <button role="menuitem" className="menu-item" onClick={() => { setOpen(false); setShowAbout(true) }}>
                About
              </button>
            </li>
          </ul>
        )}
      </div>

      {showAbout && <AboutModal onClose={() => setShowAbout(false)} />}
      {showAccount && <AccountModal onClose={() => setShowAccount(false)} />}
    </>
  )
}
