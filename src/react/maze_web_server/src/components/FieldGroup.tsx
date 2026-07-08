import type { ReactNode } from 'react'

// A titled card grouping related fields within a modal tab panel — the editor's
// Objects tab (Doors / Keys / …) and Layout tab (Grid / Levels) use it. Renders
// a bordered card (`.field-group`) with an accent caption wired as the group's
// accessible name, so screen readers announce e.g. "Grid, Rows".
interface FieldGroupProps {
  /** Caption shown as the heading and used as the group's accessible name. */
  title: string
  /** Unique-within-the-dialog slug for the heading id (aria wiring). */
  id: string
  children: ReactNode
}

export function FieldGroup({ title, id, children }: FieldGroupProps) {
  const headingId = `gamedef-grp-${id}`
  return (
    <div className="field-group" role="group" aria-labelledby={headingId}>
      <h4 id={headingId} className="field-group-title">{title}</h4>
      {children}
    </div>
  )
}
