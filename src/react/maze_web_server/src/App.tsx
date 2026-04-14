import { createBrowserRouter, Navigate, RouterProvider } from 'react-router-dom'
import { AuthProvider } from './context/AuthContext'
import { ThemeProvider } from './context/ThemeContext'
import { ProtectedRoute } from './components/ProtectedRoute'
import { LoginPage } from './pages/LoginPage'
import { SignUpPage } from './pages/SignUpPage'
import { MazesPage } from './pages/MazesPage'
import { MazePage } from './pages/MazePage'

const router = createBrowserRouter([
  { path: '/login', element: <LoginPage /> },
  { path: '/signup', element: <SignUpPage /> },
  { path: '/mazes', element: <ProtectedRoute><MazesPage /></ProtectedRoute> },
  { path: '/mazes/new', element: <ProtectedRoute><MazePage /></ProtectedRoute> },
  { path: '/mazes/:id', element: <ProtectedRoute><MazePage /></ProtectedRoute> },
  { path: '*', element: <Navigate to="/login" replace /> },
])

export default function App() {
  return (
    <ThemeProvider>
      <AuthProvider>
        <RouterProvider router={router} />
      </AuthProvider>
    </ThemeProvider>
  )
}
