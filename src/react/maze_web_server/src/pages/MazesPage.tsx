import { HamburgerMenu } from '../components/HamburgerMenu'
import { useMenuVariant } from '../hooks/useMenuVariant'

export function MazesPage() {
  const menuVariant = useMenuVariant()

  return (
    <div>
      <header style={headerStyle}>
        <span style={{ fontWeight: 'bold', fontSize: '1.1rem' }}>Maze</span>
        {menuVariant === 'hamburger' && <HamburgerMenu />}
      </header>
      <main style={{ padding: '2rem', textAlign: 'center' }}>
        <p>Mazes — coming soon</p>
      </main>
    </div>
  )
}

const headerStyle: React.CSSProperties = {
  display: 'flex', alignItems: 'center', justifyContent: 'space-between',
  padding: '0.75rem 1rem', borderBottom: '1px solid #e5e7eb',
  background: '#fff',
}
