import { createBrowserRouter, Navigate, RouterProvider } from 'react-router-dom'
import { lazy, Suspense } from 'react'
import { useAppFeatures } from './context/AppFeaturesContext'
import { AppFeaturesProvider } from './context/AppFeaturesProvider'
import { AuthProvider } from './context/AuthProvider'
import { ThemeProvider } from './context/ThemeProvider'
import { ProtectedRoute } from './components/ProtectedRoute'

// Route-level code-splitting: each page loads on demand as its own chunk, so the
// initial bundle is just the app shell + providers + router rather than every
// page at once. Pages are named exports, so each is mapped to a `default` for
// `React.lazy`.
const LoginPage = lazy(() => import('./pages/LoginPage').then(m => ({ default: m.LoginPage })))
const SignUpPage = lazy(() => import('./pages/SignUpPage').then(m => ({ default: m.SignUpPage })))
const ForgotPasswordPage = lazy(() => import('./pages/ForgotPasswordPage').then(m => ({ default: m.ForgotPasswordPage })))
const ResetPasswordPage = lazy(() => import('./pages/ResetPasswordPage').then(m => ({ default: m.ResetPasswordPage })))
const VerifyEmailPage = lazy(() => import('./pages/VerifyEmailPage').then(m => ({ default: m.VerifyEmailPage })))
const OAuthCallbackPage = lazy(() => import('./pages/OAuthCallbackPage').then(m => ({ default: m.OAuthCallbackPage })))
const HomePage = lazy(() => import('./pages/HomePage').then(m => ({ default: m.HomePage })))
const MazesPage = lazy(() => import('./pages/MazesPage').then(m => ({ default: m.MazesPage })))
const MazePage = lazy(() => import('./pages/MazePage').then(m => ({ default: m.MazePage })))
const MazeGamePage = lazy(() => import('./pages/MazeGamePage').then(m => ({ default: m.MazeGamePage })))
const WorkshopGamesPage = lazy(() => import('./pages/WorkshopGamesPage').then(m => ({ default: m.WorkshopGamesPage })))
const WorkshopHubPage = lazy(() => import('./pages/WorkshopHubPage').then(m => ({ default: m.WorkshopHubPage })))
const WorkshopCollectionsPage = lazy(() => import('./pages/WorkshopCollectionsPage').then(m => ({ default: m.WorkshopCollectionsPage })))
const WorkshopFeaturesPage = lazy(() => import('./pages/WorkshopFeaturesPage').then(m => ({ default: m.WorkshopFeaturesPage })))
const Play3dHubPage = lazy(() => import('./pages/Play3dHubPage').then(m => ({ default: m.Play3dHubPage })))
const Play3dFeaturedPage = lazy(() => import('./pages/Play3dFeaturedPage').then(m => ({ default: m.Play3dFeaturedPage })))
const Play3dPlaceholderPage = lazy(() => import('./pages/Play3dPlaceholderPage').then(m => ({ default: m.Play3dPlaceholderPage })))
const AccountPage = lazy(() => import('./pages/AccountPage').then(m => ({ default: m.AccountPage })))
const LeaderboardsPage = lazy(() => import('./pages/LeaderboardsPage').then(m => ({ default: m.LeaderboardsPage })))

export function SignupRoute() {
  const { allow_signup } = useAppFeatures()
  if (!allow_signup) return <Navigate to="/login" replace />
  return <SignUpPage />
}

const router = createBrowserRouter([
  { path: '/login', element: <LoginPage /> },
  { path: '/signup', element: <SignupRoute /> },
  { path: '/forgot-password', element: <ForgotPasswordPage /> },
  { path: '/reset-password', element: <ResetPasswordPage /> },
  { path: '/verify-email', element: <VerifyEmailPage /> },
  { path: '/oauth/callback', element: <OAuthCallbackPage /> },
  { path: '/', element: <ProtectedRoute><HomePage /></ProtectedRoute> },
  { path: '/mazes', element: <ProtectedRoute><MazesPage /></ProtectedRoute> },
  { path: '/mazes/new', element: <ProtectedRoute><MazePage /></ProtectedRoute> },
  { path: '/mazes/:id', element: <ProtectedRoute><MazePage /></ProtectedRoute> },
  { path: '/play/:id', element: <ProtectedRoute><MazeGamePage /></ProtectedRoute> },
  { path: '/workshop', element: <ProtectedRoute><WorkshopHubPage /></ProtectedRoute> },
  { path: '/workshop/games', element: <ProtectedRoute><WorkshopGamesPage /></ProtectedRoute> },
  { path: '/workshop/game-collections', element: <ProtectedRoute><WorkshopCollectionsPage /></ProtectedRoute> },
  { path: '/workshop/features', element: <ProtectedRoute><WorkshopFeaturesPage /></ProtectedRoute> },
  { path: '/play-3d', element: <ProtectedRoute><Play3dHubPage /></ProtectedRoute> },
  { path: '/play-3d/featured', element: <ProtectedRoute><Play3dFeaturedPage /></ProtectedRoute> },
  // Placeholders until the My Games / Shared with me / Community scope pages are built.
  { path: '/play-3d/my-games', element: <ProtectedRoute><Play3dPlaceholderPage title="My Games" /></ProtectedRoute> },
  { path: '/play-3d/shared', element: <ProtectedRoute><Play3dPlaceholderPage title="Shared with me" /></ProtectedRoute> },
  { path: '/play-3d/community', element: <ProtectedRoute><Play3dPlaceholderPage title="Community" /></ProtectedRoute> },
  // The bare stub route is retired; its surface now lives under the workshop hub.
  { path: '/games', element: <Navigate to="/workshop" replace /> },
  { path: '/leaderboards', element: <ProtectedRoute><LeaderboardsPage /></ProtectedRoute> },
  { path: '/account', element: <ProtectedRoute><AccountPage /></ProtectedRoute> },
  { path: '*', element: <Navigate to="/login" replace /> },
])

export default function App() {
  return (
    <ThemeProvider>
      <AppFeaturesProvider>
        <AuthProvider>
          <Suspense fallback={<div className="loading-center">Loading...</div>}>
            <RouterProvider router={router} />
          </Suspense>
        </AuthProvider>
      </AppFeaturesProvider>
    </ThemeProvider>
  )
}
