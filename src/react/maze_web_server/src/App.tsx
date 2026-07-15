import { createBrowserRouter, Navigate, RouterProvider } from 'react-router-dom'
import { useAppFeatures } from './context/AppFeaturesContext'
import { AppFeaturesProvider } from './context/AppFeaturesProvider'
import { AuthProvider } from './context/AuthProvider'
import { ThemeProvider } from './context/ThemeProvider'
import { ProtectedRoute } from './components/ProtectedRoute'
import { LoginPage } from './pages/LoginPage'
import { SignUpPage } from './pages/SignUpPage'
import { ForgotPasswordPage } from './pages/ForgotPasswordPage'
import { ResetPasswordPage } from './pages/ResetPasswordPage'
import { VerifyEmailPage } from './pages/VerifyEmailPage'
import { OAuthCallbackPage } from './pages/OAuthCallbackPage'
import { HomePage } from './pages/HomePage'
import { MazesPage } from './pages/MazesPage'
import { MazePage } from './pages/MazePage'
import { MazeGamePage } from './pages/MazeGamePage'
import { WorkshopGamesPage } from './pages/WorkshopGamesPage'
import { WorkshopHubPage } from './pages/WorkshopHubPage'
import { WorkshopCollectionsPage } from './pages/WorkshopCollectionsPage'
import { WorkshopFeaturesPage } from './pages/WorkshopFeaturesPage'
import { Play3dHubPage } from './pages/Play3dHubPage'
import { Play3dFeaturedPage } from './pages/Play3dFeaturedPage'
import { Play3dPlaceholderPage } from './pages/Play3dPlaceholderPage'
import { AccountPage } from './pages/AccountPage'
import { LeaderboardsPage } from './pages/LeaderboardsPage'

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
  // Built out in D4.6 (My Games / Shared) and D4.8 (Community); placeholders for now.
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
          <RouterProvider router={router} />
        </AuthProvider>
      </AppFeaturesProvider>
    </ThemeProvider>
  )
}
