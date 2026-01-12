import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen } from '@testing-library/react'
import { BrowserRouter } from 'react-router-dom'
import Dashboard from '../../pages/Dashboard'

// Mock the Solana wallet adapter
vi.mock('@solana/wallet-adapter-react', () => ({
  useWallet: () => ({
    connected: true,
    publicKey: {
      toString: () => '7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU'
    },
    connect: vi.fn(),
    disconnect: vi.fn(),
  }),
}))

// Mock the API client
vi.mock('../../api/client', () => ({
  default: {
    get: vi.fn().mockResolvedValue({ 
      data: { 
        total_value_locked: 0,
        total_users: 0,
        active_vaults: 0
      } 
    }),
  },
}))

describe('Dashboard', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('renders dashboard statistics', () => {
    render(
      <BrowserRouter>
        <Dashboard />
      </BrowserRouter>
    )

    expect(screen.getByText('Total Value Locked')).toBeInTheDocument()
    expect(screen.getByText('Total Users')).toBeInTheDocument()
    expect(screen.getByText('Active Vaults')).toBeInTheDocument()
  })

  it('renders vault section', () => {
    render(
      <BrowserRouter>
        <Dashboard />
      </BrowserRouter>
    )

    expect(screen.getByText('Your Vault')).toBeInTheDocument()
  })
})
