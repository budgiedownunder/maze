import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import './App.css'
import App from './App.tsx'

async function prepare() {
  if (import.meta.env.DEV && import.meta.env.VITE_MSW === 'true') {
    const { worker } = await import('./mocks/browser')
    await worker.start({ onUnhandledRequest: 'bypass' })
    // Expose mock token maps to Playwright e2e (and curious developers)
    // ONLY when MSW is active. Lets specs read minted reset / verification
    // tokens via `page.evaluate(() => window.__mswState)`.
    const handlers = await import('./mocks/handlers')
    ;(window as unknown as { __mswState: unknown }).__mswState = {
      resetTokens: handlers.mockResetTokens,
      verificationTokens: handlers.mockVerificationTokens,
    }
  }
}

prepare().then(() => {
  createRoot(document.getElementById('root')!).render(
    <StrictMode>
      <App />
    </StrictMode>,
  )
})
