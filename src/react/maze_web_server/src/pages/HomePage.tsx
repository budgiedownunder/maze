import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { AppHeader } from '../components/AppHeader'
import { AlertModal } from '../components/AlertModal'
import { useToken } from '../context/AuthContext'
import { useBusyCursor } from '../hooks/useBusyCursor'
import { getFeaturedGameItems, getGameCollection } from '../api/client'
import { launchDefinition } from '../utils/play3dLaunch'
import appIcon from '../assets/app.png'
import play3dIcon from '../assets/play3d.png'
import workshopIcon from '../assets/workshop.svg'
import leaderboardsIcon from '../assets/leaderboards.svg'

// The curated collection the daily games live in (seeded at server startup); the
// Today's Challenge tile finds it in the featured catalogue by this name.
const DAILY_CHALLENGES_COLLECTION = 'Daily Challenges'

export function HomePage() {
  const navigate = useNavigate()
  const token = useToken()
  const [resolvingDaily, setResolvingDaily] = useState(false)
  const [dailyError, setDailyError] = useState<string | null>(null)
  useBusyCursor(resolvingDaily)

  // Launch today's daily challenge by client-resolving the curated "Daily
  // Challenges" collection — find it in the featured catalogue, then launch its
  // daily member (the host page date-mixes the seed for the current UTC day). No
  // dedicated endpoint. Guarded end to end: a missing collection or member (or a
  // load failure) surfaces a friendly alert rather than a dead tile.
  async function handleTodaysChallenge() {
    if (!token || resolvingDaily) return
    setResolvingDaily(true)
    setDailyError(null)
    try {
      const featured = await getFeaturedGameItems(token, { limit: 100 })
      const collection = featured.items.find(
        i => i.kind === 'collection' && i.collection?.name === DAILY_CHALLENGES_COLLECTION,
      )?.collection
      const detail = collection ? await getGameCollection(token, collection.id) : null
      const daily = detail?.definitions.find(d => d.rotation === 'daily') ?? detail?.definitions[0]
      if (!daily) {
        setDailyError('There is no daily challenge available right now.')
        return
      }
      launchDefinition(daily.id)
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
            <img src={leaderboardsIcon} className="home-tile-img" alt="" aria-hidden="true" />
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
