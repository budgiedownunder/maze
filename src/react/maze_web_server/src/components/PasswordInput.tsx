import { useState } from 'react'
import showIcon from '../assets/icon_show_password.png'
import hideIcon from '../assets/icon_hide_password.png'

interface Props {
  id: string
  value: string
  onChange: (value: string) => void
  placeholder?: string
  disabled?: boolean
}

export function PasswordInput({ id, value, onChange, placeholder, disabled }: Props) {
  const [visible, setVisible] = useState(false)

  return (
    <div className="password-wrapper">
      <input
        id={id}
        type={visible ? 'text' : 'password'}
        value={value}
        onChange={e => onChange(e.target.value)}
        placeholder={placeholder}
        disabled={disabled}
        className="password-input"
      />
      <button
        type="button"
        onClick={() => setVisible(v => !v)}
        disabled={disabled}
        aria-label={visible ? 'Hide password' : 'Show password'}
        className="password-toggle"
      >
        <img src={visible ? hideIcon : showIcon} alt="" width={20} height={20} />
      </button>
    </div>
  )
}
