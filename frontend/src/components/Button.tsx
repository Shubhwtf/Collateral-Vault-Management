import styled from 'styled-components'

interface ButtonProps {
  variant?: 'primary' | 'secondary'
  disabled?: boolean
  onClick?: () => void
  children: React.ReactNode
}

const StyledButton = styled.button<{ variant: 'primary' | 'secondary' }>`
  padding: 0.75rem 1.5rem;
  font-size: 0.9375rem;
  font-weight: 500;
  border: 1px solid ${props => props.variant === 'primary' ? 'var(--accent)' : 'var(--border-color)'};
  background-color: ${props => props.variant === 'primary' ? 'var(--accent)' : 'transparent'};
  color: ${props => props.variant === 'primary' ? 'var(--bg-primary)' : 'var(--text-primary)'};
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.2s ease;

  &:hover:not(:disabled) {
    background-color: ${props => props.variant === 'primary' ? 'var(--accent-hover)' : 'var(--bg-tertiary)'};
    border-color: ${props => props.variant === 'primary' ? 'var(--accent-hover)' : 'var(--text-secondary)'};
  }

  &:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
`

export default function Button({ variant = 'primary', disabled, onClick, children }: ButtonProps) {
  return (
    <StyledButton variant={variant} disabled={disabled} onClick={onClick}>
      {children}
    </StyledButton>
  )
}

