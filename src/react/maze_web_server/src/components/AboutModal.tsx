import appIcon from '../assets/app.png'

interface Props {
  onClose: () => void
}

export function AboutModal({ onClose }: Props) {
  return (
    <div role="dialog" aria-modal="true" aria-label="About" className="modal-overlay">
      <div className="modal modal-sm modal-centered">
        <img src={appIcon} alt="Maze app icon" width={64} height={64} className="auth-logo" />
        <p className="about-name">Maze</p>
        <p className="about-copy">© BudgieDownUnder, 2026</p>
        <hr className="about-rule" />
        <p className="about-desc">
          Maze designer and solver
        </p>
        <button onClick={onClose} className="btn-gray about-close">Close</button>
      </div>
    </div>
  )
}
