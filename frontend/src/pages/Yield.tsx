import { useEffect, useState } from 'react'
import { useWallet } from '@solana/wallet-adapter-react'
import styled from 'styled-components'
import { api, BalanceResponse, YieldInfoResponse } from '../api/client'
import Button from '../components/Button'
import Card from '../components/Card'
import { formatUSDT } from '../utils/format'
import { Connection, Transaction } from '@solana/web3.js'

const YieldContainer = styled.div`
  display: flex;
  flex-direction: column;
  gap: 2rem;
  max-width: 800px;
`

const Section = styled(Card)`
  padding: 2rem;
`

const SectionTitle = styled.h2`
  font-size: 1.25rem;
  font-weight: 600;
  margin-bottom: 1.5rem;
  color: var(--text-primary);
`

const InfoGrid = styled.div`
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 1rem;
  margin-bottom: 2rem;
  padding: 1.5rem;
  background-color: var(--bg-tertiary);
  border-radius: 4px;
`

const InfoItem = styled.div`
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
`

const InfoLabel = styled.div`
  color: var(--text-secondary);
  font-size: 0.75rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
`

const InfoValue = styled.div`
  font-size: 1.25rem;
  font-weight: 600;
  color: var(--text-primary);
`

const StatusMessage = styled.div<{ type: 'success' | 'error' | 'info' }>`
  padding: 1rem;
  margin-top: 1rem;
  border-radius: 4px;
  background-color: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  color: ${props => 
    props.type === 'success' ? 'var(--success)' :
    props.type === 'error' ? 'var(--error)' :
    'var(--text-secondary)'
  };
`
const Toggle = styled.label`
  display: flex;
  align-items: center;
  gap: 0.75rem;
  cursor: pointer;
`

const ToggleSwitch = styled.div<{ checked: boolean }>`
  width: 48px;
  height: 24px;
  background-color: ${props => props.checked ? 'var(--primary, #3b82f6)' : 'var(--bg-tertiary)'};
  border-radius: 12px;
  position: relative;
  transition: background-color 0.2s;

  &::after {
    content: '';
    position: absolute;
    width: 20px;
    height: 20px;
    background-color: white;
    border-radius: 50%;
    top: 2px;
    left: ${props => props.checked ? '26px' : '2px'};
    transition: left 0.2s;
  }
`

export default function Yield() {
  const { publicKey, connected, signTransaction } = useWallet()
  const [balance, setBalance] = useState<BalanceResponse | null>(null)
  const [loading, setLoading] = useState(false)
  const [status, setStatus] = useState<{ type: 'success' | 'error' | 'info', message: string } | null>(null)
  const [yieldInfo, setYieldInfo] = useState<YieldInfoResponse | null>(null)

  useEffect(() => {
    if (connected && publicKey) {
      fetchBalance()
      fetchYieldInfo()
    }
  }, [connected, publicKey])

  const fetchBalance = async () => {
    if (!publicKey) return
    try {
      const res = await api.getBalance(publicKey.toBase58())
      setBalance(res.data)
    } catch (err: any) {
      if (err.response?.status === 404) {
        setBalance(null)
      }
    }
  }

  const fetchYieldInfo = async () => {
    if (!publicKey) return
    try {
      const res = await api.getYieldInfo(publicKey.toBase58())
      setYieldInfo(res.data)
    } catch (err: any) {
      console.error('Failed to fetch yield info', err)
      if (err.response?.status === 404) {
        setYieldInfo(null)
      }
    }
  }

  const handleCompoundYield = async () => {
    if (!publicKey || !signTransaction) {
      setStatus({ type: 'error', message: 'Wallet not connected' })
      return
    }

    // Check if yield is enabled
    if (!yieldInfo?.yield_enabled) {
      setStatus({ type: 'error', message: 'Please enable auto-compound first before compounding yield' })
      return
    }

    // Check cooldown
    if (yieldInfo?.time_until_next_compound && yieldInfo.time_until_next_compound > 0) {
      setStatus({ type: 'error', message: `Please wait ${Math.ceil(yieldInfo.time_until_next_compound / 60)} more minutes before compounding` })
      return
    }

    setLoading(true)
    setStatus(null)

    try {
      const txRes = await api.buildCompoundYieldTx(publicKey.toBase58())
      const txData = txRes.data

      const txBuffer = Buffer.from(txData.transaction_base64, 'base64')
      const transaction = Transaction.from(txBuffer)

      const signedTx = await signTransaction(transaction)
      
      const rpcUrl = import.meta.env.VITE_SOLANA_RPC_URL || 'https://api.devnet.solana.com'
      const connection = new Connection(rpcUrl, 'confirmed')
      const signature = await connection.sendRawTransaction(signedTx.serialize())
      
      console.log('Transaction sent:', signature)
      
      await connection.confirmTransaction(signature, 'confirmed')

      console.log('Transaction confirmed')

      await api.syncYieldTx(publicKey.toBase58(), signature)

      setStatus({ type: 'success', message: `Yield compounded successfully! Signature: ${signature.slice(0, 8)}...` })
      
      // Refresh data after a delay
      setTimeout(async () => {
        await fetchBalance()
        await fetchYieldInfo()
      }, 2000)
    } catch (err: any) {
      console.error('Compound error:', err)
      const errorMsg = err.response?.data?.error || err.message || 'Failed to compound yield'
      setStatus({ type: 'error', message: errorMsg })
    } finally {
      setLoading(false)
    }
  }

  const handleToggleAutoCompound = async () => {
    if (!publicKey || !signTransaction) {
      setStatus({ type: 'error', message: 'Wallet not connected' })
      return
    }

    if (loading) return // Prevent multiple clicks

    setLoading(true)
    setStatus(null)

    try {
      const newState = !yieldInfo?.yield_enabled
      
      const txRes = await api.buildConfigureYieldTx(publicKey.toBase58(), newState)
      const txData = txRes.data

      const txBuffer = Buffer.from(txData.transaction_base64, 'base64')
      const transaction = Transaction.from(txBuffer)

      const signedTx = await signTransaction(transaction)
      
      const rpcUrl = import.meta.env.VITE_SOLANA_RPC_URL || 'https://api.devnet.solana.com'
      const connection = new Connection(rpcUrl, 'confirmed')
      const signature = await connection.sendRawTransaction(signedTx.serialize())
      
      await connection.confirmTransaction(signature, 'confirmed')

      await api.syncYieldTx(publicKey.toBase58(), signature)

      // Update local state immediately for better UX
      setYieldInfo(prev => prev ? { ...prev, yield_enabled: newState } : null)

      setStatus({ 
        type: 'success', 
        message: `Auto-compound ${newState ? 'enabled' : 'disabled'} successfully!` 
      })
      
      // Wait longer for blockchain state to propagate before fetching
      setTimeout(async () => {
        await fetchYieldInfo()
      }, 3000)
    } catch (err: any) {
      setStatus({ type: 'error', message: err.response?.data?.error || err.message || 'Failed to update auto-compound' })
      // Revert optimistic update on error
      await fetchYieldInfo()
    } finally {
      setLoading(false)
    }
  }

  if (!connected) {
    return (
      <YieldContainer>
        <Card style={{ padding: '2rem', textAlign: 'center' }}>
          <div style={{ color: 'var(--text-secondary)' }}>
            Please connect your Phantom wallet to manage yield
          </div>
        </Card>
      </YieldContainer>
    )
  }

  if (!balance) {
    return (
      <YieldContainer>
        <Card style={{ padding: '2rem', textAlign: 'center' }}>
          <div style={{ color: 'var(--text-secondary)' }}>
            Initialize your vault first to start earning yield
          </div>
        </Card>
      </YieldContainer>
    )
  }

  // const apy = yieldInfo ? ((yieldInfo.total_yield_earned / balance.vault.total_balance) * 100) : 0
  const lastCompoundDate = yieldInfo?.last_yield_compound 
    ? new Date(yieldInfo.last_yield_compound * 1000).toLocaleDateString() 
    : 'Never'

  return (
    <YieldContainer>
      <Section>
        <SectionTitle>Yield Overview</SectionTitle>
        <InfoGrid>
          <InfoItem>
            <InfoLabel>Accumulated Yield</InfoLabel>
            <InfoValue>{formatUSDT(yieldInfo?.total_yield_earned || 0)}</InfoValue>
          </InfoItem>
          <InfoItem>
            <InfoLabel>Estimated Next Yield</InfoLabel>
            <InfoValue>{formatUSDT(yieldInfo?.estimated_next_yield || 0)}</InfoValue>
          </InfoItem>
          <InfoItem>
            <InfoLabel>Total Balance</InfoLabel>
            <InfoValue>{formatUSDT(balance.vault.total_balance)}</InfoValue>
          </InfoItem>
          <InfoItem>
            <InfoLabel>Last Compounded</InfoLabel>
            <InfoValue style={{ fontSize: '0.875rem' }}>
              {lastCompoundDate}
            </InfoValue>
          </InfoItem>
        </InfoGrid>
        {yieldInfo && yieldInfo.time_until_next_compound > 0 && (
          <p style={{ color: 'var(--text-secondary)', fontSize: '0.875rem', marginTop: '1rem' }}>
            Next compound available in {Math.ceil(yieldInfo.time_until_next_compound / 60)} minutes
          </p>
        )}
      </Section>

      <Section>
        <SectionTitle>Compound Yield</SectionTitle>
        <p style={{ color: 'var(--text-secondary)', marginBottom: '1.5rem' }}>
          Manually compound your earned yield to increase your balance
        </p>
        <Button 
          onClick={handleCompoundYield} 
          disabled={loading || (yieldInfo?.time_until_next_compound || 0) > 0}
        >
          {loading ? 'Processing...' : 'Compound Now'}
        </Button>
      </Section>

      <Section>
        <SectionTitle>Auto-Compound Settings</SectionTitle>
        <p style={{ color: 'var(--text-secondary)', marginBottom: '1.5rem' }}>
          Enable automatic yield compounding to maximize your returns
        </p>
        <Toggle>
          <ToggleSwitch 
            checked={yieldInfo?.yield_enabled || false}
            onClick={loading ? undefined : handleToggleAutoCompound}
            style={{ cursor: loading ? 'not-allowed' : 'pointer', opacity: loading ? 0.6 : 1 }}
          />
          <span style={{ color: 'var(--text-primary)' }}>
            {loading ? 'Updating...' : (yieldInfo?.yield_enabled ? 'Enabled' : 'Disabled')}
          </span>
        </Toggle>
      </Section>

      {status && (
        <StatusMessage type={status.type}>{status.message}</StatusMessage>
      )}
    </YieldContainer>
  )
}
