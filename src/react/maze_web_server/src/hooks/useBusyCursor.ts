import { useEffect } from 'react'

// Adds the global `is-busy` body class (wait cursor) while `active`, and clears
// it when `active` goes false or the component unmounts — so the cursor is
// always reset, whether the work succeeded or failed.
export function useBusyCursor(active: boolean): void {
  useEffect(() => {
    if (!active) return
    document.body.classList.add('is-busy')
    return () => { document.body.classList.remove('is-busy') }
  }, [active])
}
