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
import { AccountPage } from './pages/AccountPage'
import { MyScoresPage } from './pages/MyScoresPage'

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
  { path: '/scores', element: <ProtectedRoute><MyScoresPage /></ProtectedRoute> },
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
