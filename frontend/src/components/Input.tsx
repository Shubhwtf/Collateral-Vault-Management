import styled from 'styled-components'

const StyledInput = styled.input`
  width: 100%;
  padding: 0.75rem;
  font-size: 0.9375rem;
  background-color: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  color: var(--text-primary);
  transition: border-color 0.2s ease;

  &:focus {
    outline: none;
    border-color: var(--accent);
  }

  &::placeholder {
    color: var(--text-secondary);
  }
`

interface InputProps extends React.InputHTMLAttributes<HTMLInputElement> {}

export default function Input(props: InputProps) {
  return <StyledInput {...props} />
}

