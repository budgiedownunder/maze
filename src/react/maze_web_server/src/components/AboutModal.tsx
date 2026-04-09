import appIcon from '../assets/app.png'

interface Props {
  onClose: () => void
}

export function AboutModal({ onClose }: Props) {
  return (
    <div role="dialog" aria-modal="true" aria-label="About" style={overlayStyle}>
      <div style={modalStyle}>
        <img src={appIcon} alt="Maze app icon" width={64} height={64} style={{ borderRadius: '50%' }} />
        <p style={{ fontWeight: 'bold', margin: '0.75rem 0 0.25rem' }}>Maze Application v1.0</p>
        <p style={{ margin: '0 0 0.75rem' }}>© BudgieDownUnder, 2026</p>
        <hr style={{ width: '100%', margin: '0.5rem 0' }} />
        <p style={{ margin: '0.75rem 0 1.5rem', textAlign: 'center' }}>
          An app for designing and solving mazes
        </p>
        <button onClick={onClose}>Close</button>
      </div>
    </div>
  )
}

const overlayStyle: React.CSSProperties = {
  position: 'fixed', inset: 0,
  background: 'rgba(0,0,0,0.5)',
  display: 'flex', alignItems: 'center', justifyContent: 'center',
  zIndex: 1000,
}

const modalStyle: React.CSSProperties = {
  background: '#fff',
  borderRadius: '0.5rem',
  padding: '2rem',
  minWidth: '280px',
  maxWidth: '360px',
  display: 'flex',
  flexDirection: 'column',
  alignItems: 'center',
}
