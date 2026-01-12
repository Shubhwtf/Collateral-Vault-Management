import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import Input from '../../components/Input'

describe('Input', () => {
  it('renders input element', () => {
    render(<Input placeholder="Enter text" />)
    expect(screen.getByPlaceholderText('Enter text')).toBeInTheDocument()
  })

  it('handles value changes', () => {
    const handleChange = vi.fn()
    render(<Input placeholder="Test" onChange={handleChange} />)
    
    const input = screen.getByPlaceholderText('Test')
    fireEvent.change(input, { target: { value: 'test value' } })
    
    expect(handleChange).toHaveBeenCalled()
  })

  it('is disabled when disabled prop is true', () => {
    render(<Input placeholder="Test" disabled />)
    expect(screen.getByPlaceholderText('Test')).toBeDisabled()
  })

  it('applies different input types correctly', () => {
    const { rerender } = render(<Input placeholder="Email" type="email" />)
    expect(screen.getByPlaceholderText('Email')).toHaveAttribute('type', 'email')
    
    rerender(<Input placeholder="Password" type="password" />)
    expect(screen.getByPlaceholderText('Password')).toHaveAttribute('type', 'password')
  })
})
