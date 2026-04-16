import { SPEED_LEVELS } from '../hooks/useWalkSpeed'

interface WalkSpeedControlProps {
  speedIndex: number
  onSpeedChange: (index: number) => void
}

export function WalkSpeedControl({ speedIndex, onSpeedChange }: WalkSpeedControlProps) {
  return (
    <select
      className="walk-speed-select"
      aria-label="Walk speed"
      value={speedIndex}
      onChange={e => onSpeedChange(Number(e.target.value))}
    >
      {SPEED_LEVELS.map((level, i) => (
        <option key={level.label} value={i}>
          {level.label}
        </option>
      ))}
    </select>
  )
}
