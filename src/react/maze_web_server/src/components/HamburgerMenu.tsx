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
      <div ref={menuRef} style={{ position: 'relative' }}>
        <button
          aria-label="Open menu"
          aria-expanded={open}
          aria-haspopup="menu"
          onClick={() => setOpen(v => !v)}
          style={{ fontSize: '1.5rem', background: 'none', border: 'none', cursor: 'pointer' }}
        >
          ☰
        </button>

        {open && (
          <ul
            role="menu"
            style={{
              position: 'absolute', right: 0, top: '2.5rem',
              background: '#fff', border: '1px solid #ddd',
              borderRadius: '0.375rem', listStyle: 'none',
              margin: 0, padding: '0.25rem 0', minWidth: '160px',
              boxShadow: '0 4px 12px rgba(0,0,0,0.15)', zIndex: 500,
            }}
          >
            <li role="none">
              <button role="menuitem" style={itemStyle} onClick={() => { setOpen(false); setShowAccount(true) }}>
                My Account...
              </button>
            </li>
            <li role="none">
              <button role="menuitem" style={itemStyle} onClick={handleSignOut}>
                Sign Out
              </button>
            </li>
            <li role="none">
              <button role="menuitem" style={itemStyle} onClick={() => { setOpen(false); setShowAbout(true) }}>
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

const itemStyle: React.CSSProperties = {
  display: 'block', width: '100%', textAlign: 'left',
  background: 'none', border: 'none', cursor: 'pointer',
  padding: '0.5rem 1rem', fontSize: '0.95rem',
}
