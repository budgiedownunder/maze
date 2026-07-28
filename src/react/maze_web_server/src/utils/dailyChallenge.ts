import { getFeaturedGameItems, getGameCollection } from '../api/client'
import { launchDefinition } from './play3dLaunch'

// The curated collection the daily games live in (seeded at server startup); the
// Today's Challenge entry points find it in the featured catalogue by this name.
const DAILY_CHALLENGES_COLLECTION = 'Daily Challenges'

// Resolve and launch today's daily challenge — client-side, no dedicated
// endpoint: find the curated "Daily Challenges" collection in the featured
// catalogue, resolve its detail, and launch its daily member (the host page
// date-mixes the seed for the current UTC day). Returns true when a game was
// launched, false when there's nothing to play (no collection or no member);
// throws on a network failure so the caller can distinguish "unavailable" from
// "load failed". Shared by the Home tile and the hamburger menu.
export async function launchTodaysChallenge(token: string): Promise<boolean> {
  const featured = await getFeaturedGameItems(token, { limit: 100 })
  const collection = featured.items.find(
    i => i.kind === 'collection' && i.collection?.name === DAILY_CHALLENGES_COLLECTION,
  )?.collection
  const detail = collection ? await getGameCollection(token, collection.id) : null
  const daily = detail?.definitions.find(d => d.rotation === 'daily') ?? detail?.definitions[0]
  if (!daily) return false
  launchDefinition(daily.id)
  return true
}
