import * as api from '../api/client'
import type { OAuthProviderPublic } from '../types/api'

// Inline SVG provider icons. Inline (rather than separate asset files) so the
// list renders without any extra HTTP requests and so the icons inherit
// currentColor where the canonical brand glyph allows it.

function GoogleIcon() {
  // Official multi-colour Google "G". Brand colours required by Google's
  // identity guidelines, so this one does not honour theme colours.
  return (
    <svg width="18" height="18" viewBox="0 0 48 48" aria-hidden="true">
      <path fill="#FFC107" d="M43.6 20.5H42V20H24v8h11.3c-1.6 4.6-6 8-11.3 8-6.6 0-12-5.4-12-12s5.4-12 12-12c3 0 5.7 1.1 7.8 3l5.7-5.7C33.6 6.5 29 4.5 24 4.5 13.2 4.5 4.5 13.2 4.5 24S13.2 43.5 24 43.5c10.8 0 19.5-8.7 19.5-19.5 0-1.2-.1-2.4-.4-3.5z"/>
      <path fill="#FF3D00" d="M6.3 14.7l6.6 4.8c1.8-4.4 6.1-7.5 11.1-7.5 3 0 5.7 1.1 7.8 3l5.7-5.7C33.6 6.5 29 4.5 24 4.5c-7 0-13.1 4-16.1 9.7z"/>
      <path fill="#4CAF50" d="M24 43.5c4.9 0 9.4-1.9 12.8-5l-5.9-5c-1.9 1.3-4.3 2-6.9 2-5.3 0-9.7-3.4-11.3-8L6.2 32C9.2 38.4 16 43.5 24 43.5z"/>
      <path fill="#1976D2" d="M43.6 20.5H42V20H24v8h11.3c-.8 2.2-2.2 4.1-4 5.5l5.9 5c-.4.4 6.3-4.6 6.3-13.5 0-1.2-.1-2.4-.4-3.5z"/>
    </svg>
  )
}

function GitHubIcon() {
  // Octocat silhouette. Single colour — uses `currentColor` so the icon
  // adapts to dark/light themes automatically.
  return (
    <svg width="18" height="18" viewBox="0 0 24 24" aria-hidden="true" fill="currentColor">
      <path d="M12 .5C5.65.5.5 5.65.5 12c0 5.08 3.29 9.39 7.86 10.91.58.11.79-.25.79-.56v-2c-3.2.7-3.88-1.36-3.88-1.36-.52-1.32-1.27-1.67-1.27-1.67-1.04-.71.08-.7.08-.7 1.15.08 1.76 1.18 1.76 1.18 1.02 1.75 2.68 1.24 3.34.95.1-.74.4-1.24.72-1.53-2.55-.29-5.24-1.28-5.24-5.7 0-1.26.45-2.29 1.18-3.1-.12-.29-.51-1.46.11-3.05 0 0 .97-.31 3.18 1.19a11 11 0 0 1 2.9-.39c.98 0 1.97.13 2.9.39 2.21-1.5 3.18-1.19 3.18-1.19.62 1.59.23 2.76.11 3.05.74.81 1.18 1.84 1.18 3.1 0 4.43-2.7 5.41-5.27 5.69.41.36.78 1.06.78 2.14v3.18c0 .31.21.68.8.56C20.21 21.39 23.5 17.08 23.5 12 23.5 5.65 18.35.5 12 .5z"/>
    </svg>
  )
}

function FacebookIcon() {
  // Stylised "f" mark in Facebook brand blue (#1877F2). Single colour;
  // theme-adaptation handled at the button level (white on dark mode bg
  // through `currentColor`-based CSS) — fixed brand blue in light mode.
  return (
    <svg width="18" height="18" viewBox="0 0 24 24" aria-hidden="true" fill="#1877F2">
      <path d="M22.675 0H1.325C.593 0 0 .593 0 1.325v21.351C0 23.408.593 24 1.325 24H12.82V14.706h-3.13v-3.622h3.13V8.413c0-3.1 1.894-4.788 4.659-4.788 1.325 0 2.464.099 2.795.143v3.24l-1.918.001c-1.504 0-1.795.715-1.795 1.763v2.31h3.587l-.467 3.622h-3.12V24h6.116c.73 0 1.323-.592 1.323-1.324V1.325C24 .593 23.408 0 22.675 0z"/>
    </svg>
  )
}

function ProviderIcon({ name }: { name: string }) {
  switch (name) {
    case 'google':   return <GoogleIcon />
    case 'github':   return <GitHubIcon />
    case 'facebook': return <FacebookIcon />
    default:         return null
  }
}

interface Props {
  providers: OAuthProviderPublic[]
  disabled?: boolean
}

export function OAuthButtons({ providers, disabled }: Props) {
  if (providers.length === 0) return null
  return (
    <div className="oauth-buttons">
      <div className="oauth-divider" role="separator" aria-label="or"><span>or</span></div>
      {providers.map(p => (
        <button
          key={p.name}
          type="button"
          className={`btn-oauth btn-oauth--${p.name}`}
          onClick={() => api.startOAuth(p.name)}
          disabled={disabled}
        >
          <ProviderIcon name={p.name} />
          <span>Continue with {p.display_name}</span>
        </button>
      ))}
    </div>
  )
}
