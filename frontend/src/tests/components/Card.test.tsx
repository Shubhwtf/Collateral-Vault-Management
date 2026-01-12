import { describe, it, expect } from 'vitest'
import { render, screen } from '@testing-library/react'
import Card from '../../components/Card'

describe('Card', () => {
  it('renders card with children', () => {
    render(<Card><div>Card content</div></Card>)
    expect(screen.getByText('Card content')).toBeInTheDocument()
  })

  it('applies as attribute', () => {
    const { container } = render(<Card as="section"><div>Content</div></Card>)
    expect(container.querySelector('section')).toBeInTheDocument()
  })
})
