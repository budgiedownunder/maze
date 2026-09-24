/** The flash notices sibling pages can ask the login page to show, by code.
 *  Codes rather than text: `/login?message=…` is reachable by anyone, and a
 *  crafted link that put its own words on the real login page would read as if
 *  the site had said them. An unrecognised code shows nothing. */
const LOGIN_FLASH_MESSAGES = {
  password_reset: 'Password reset successful. Sign in with your new password.',
  signup_check_inbox: 'Account created. Check your inbox for a verification email before signing in.',
  signup_ready: 'Account created. You can sign in now.',
} as const

export type LoginFlashCode = keyof typeof LOGIN_FLASH_MESSAGES

/** The notice for a `?message=` code, or null when the code is unknown. */
export function getLoginFlashMessage(code: string | null): string | null {
  if (!code) return null
  return LOGIN_FLASH_MESSAGES[code as LoginFlashCode] ?? null
}
