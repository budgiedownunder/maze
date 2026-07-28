import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { AppHeader } from '../components/AppHeader'
import { AlertModal } from '../components/AlertModal'
import { useToken } from '../context/AuthContext'
import { useBusyCursor } from '../hooks/useBusyCursor'
import { launchTodaysChallenge } from '../utils/dailyChallenge'
import appIcon from '../assets/app.png'
import play3dIcon from '../assets/play3d.png'
import workshopIcon from '../assets/workshop.svg'
import leaderboardsIcon from '../assets/leaderboards.svg'
import dailyChallengeIcon from '../assets/daily-challenge.svg'

export function HomePage() {
  const navigate = useNavigate()
  const token = useToken()
  const [resolvingDaily, setResolvingDaily] = useState(false)
  const [dailyError, setDailyError] = useState<string | null>(null)
  useBusyCursor(resolvingDaily)

  // Launch today's daily challenge (see launchTodaysChallenge). Guarded end to
  // end: nothing to play, or a load failure, surfaces a friendly alert rather
  // than a dead tile.
  async function handleTodaysChallenge() {
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

  function handle3dGames() {
    navigate('/play-3d')
  }

  function handleWorkshop() {
    navigate('/workshop')
  }

  function handleMyMazes() {
    navigate('/mazes')
  }

  function handleLeaderboards() {
    navigate('/leaderboards')
  }

  return (
    <div className="home-page">
      <AppHeader title="Home" />
      <main className="home-main">
        <section className="home-tiles">
          <button type="button" className="home-tile" onClick={handleTodaysChallenge} disabled={resolvingDaily}>
            <img src={dailyChallengeIcon} className="home-tile-img" alt="" aria-hidden="true" />
            <div className="home-tile-text">
              <h2 className="home-tile-title">Today's Challenge</h2>
              <p className="home-tile-desc">Play today's daily 3D game and climb the board</p>
            </div>
          </button>
          <button type="button" className="home-tile" onClick={handle3dGames}>
            <img src={play3dIcon} className="home-tile-img home-tile-img--photo" alt="" aria-hidden="true" />
            <div className="home-tile-text">
              <h2 className="home-tile-title">3D Games</h2>
              <p className="home-tile-desc">Browse and play 3D games</p>
            </div>
          </button>
          <button type="button" className="home-tile" onClick={handleWorkshop}>
            <img src={workshopIcon} className="home-tile-img" alt="" aria-hidden="true" />
            <div className="home-tile-text">
              <h2 className="home-tile-title">3D Game Workshop</h2>
              <p className="home-tile-desc">Create, publish and share your own 3D games and collections</p>
            </div>
          </button>
          <button type="button" className="home-tile" onClick={handleMyMazes}>
            <img src={appIcon} className="home-tile-img home-tile-img--photo" alt="" aria-hidden="true" />
            <div className="home-tile-text">
              <h2 className="home-tile-title">Mazes</h2>
              <p className="home-tile-desc">Design and play your own single-level mazes</p>
            </div>
          </button>
          <button type="button" className="home-tile" onClick={handleLeaderboards}>
            <img src={leaderboardsIcon} className="home-tile-img" alt="" aria-hidden="true" />
            <div className="home-tile-text">
              <h2 className="home-tile-title">Leaderboards</h2>
              <p className="home-tile-desc">See your times and how you rank</p>
            </div>
          </button>
        </section>
      </main>
      {dailyError && (
        <AlertModal title="Daily Challenge" message={dailyError} onClose={() => setDailyError(null)} />
      )}
    </div>
  )
}
