import { useEffect, useState } from 'react'
import { useWallet } from '@solana/wallet-adapter-react'
import { Connection, Transaction } from '@solana/web3.js'
import styled from 'styled-components'
import { api, BalanceResponse } from '../api/client'
import Button from '../components/Button'
import Card from '../components/Card'
import Input from '../components/Input'
import MfaSetup from '../components/MfaSetup'
import { formatUSDT } from '../utils/format'

const VaultContainer = styled.div`
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

const FormGroup = styled.div`
  margin-bottom: 1.5rem;
`

const Label = styled.label`
  display: block;
  margin-bottom: 0.5rem;
  color: var(--text-secondary);
  font-size: 0.875rem;
  font-weight: 500;
`

const ButtonGroup = styled.div`
  display: flex;
  gap: 1rem;
  margin-top: 1.5rem;
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

const BalanceDisplay = styled.div`
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 1rem;
  margin-bottom: 2rem;
  padding: 1.5rem;
  background-color: var(--bg-tertiary);
  border-radius: 4px;
`

const BalanceItem = styled.div`
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
`

const BalanceLabel = styled.div`
  color: var(--text-secondary);
  font-size: 0.75rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
`

const BalanceValue = styled.div`
  font-size: 1.25rem;
  font-weight: 600;
  color: var(--text-primary);
`

const MfaSection = styled(Card)`
  padding: 2rem;
  border: 2px solid var(--primary, #3b82f6);
`

const MfaStatusBadge = styled.div<{ enabled: boolean }>`
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 1rem;
  border-radius: 9999px;
  background-color: ${props => props.enabled ? 'var(--success, #10b981)' : 'var(--bg-tertiary)'};
  color: ${props => props.enabled ? '#000000' : 'var(--text-secondary)'};
  font-size: 0.875rem;
  font-weight: 600;
  margin-bottom: 1rem;
`

const MfaModal = styled.div`
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.75);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
`

const MfaModalContent = styled(Card)`
  padding: 2rem;
  max-width: 400px;
  width: 90%;
`

const MfaModalTitle = styled.h3`
  font-size: 1.25rem;
  font-weight: 600;
  margin-bottom: 1rem;
  color: var(--text-primary);
`

export default function Vault() {
  const { publicKey, connected, signTransaction } = useWallet()
  const [balance, setBalance] = useState<BalanceResponse | null>(null)
  const [loading, setLoading] = useState(false)
  const [status, setStatus] = useState<{ type: 'success' | 'error' | 'info', message: string } | null>(null)
  const [depositAmount, setDepositAmount] = useState('')
  const [withdrawAmount, setWithdrawAmount] = useState('')
  const [mfaEnabled, setMfaEnabled] = useState(false)
  const [showMfaSetup, setShowMfaSetup] = useState(false)
  const [showMfaVerify, setShowMfaVerify] = useState(false)
  const [mfaCode, setMfaCode] = useState('')
  const [pendingWithdrawal, setPendingWithdrawal] = useState<number | null>(null)

  useEffect(() => {
    if (connected && publicKey) {
      fetchBalance()
    }
  }, [connected, publicKey])

  useEffect(() => {
    if (balance?.vault.vault_address) {
      checkMfaStatus()
    }
  }, [balance?.vault.vault_address])

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

  const checkMfaStatus = async () => {
    if (!balance?.vault.vault_address) return
    try {
      const res = await api.getMfaStatus(balance.vault.vault_address)
      setMfaEnabled(res.data.mfa_enabled)
    } catch (err) {
      // MFA status check failed silently
    }
  }

  const handleMfaSetupComplete = () => {
    setShowMfaSetup(false)
    setMfaEnabled(true)
    setStatus({ type: 'success', message: 'MFA has been enabled successfully!' })
  }

  const handleDisableMfa = async () => {
    if (!balance?.vault.vault_address) return
    
    const code = prompt('Enter your 6-digit MFA code to disable 2FA:')
    if (!code) return

    setLoading(true)
    try {
      await api.disableMfa(balance.vault.vault_address, code)
      setMfaEnabled(false)
      setStatus({ type: 'success', message: 'MFA has been disabled' })
    } catch (err: any) {
      setStatus({ 
        type: 'error', 
        message: err.response?.data?.error || 'Failed to disable MFA' 
      })
    } finally {
      setLoading(false)
    }
  }

  const handleInitialize = async () => {
    if (!publicKey || !signTransaction) {
      setStatus({ type: 'error', message: 'Wallet not connected' })
      return
    }

    setLoading(true)
    setStatus(null)

    try {
      const res = await api.buildInitializeTx(publicKey.toBase58())
      const { transaction_base64 } = res.data

      const txBytes = Uint8Array.from(atob(transaction_base64), c => c.charCodeAt(0))
      const tx = Transaction.from(txBytes)

      const connection = new Connection('https://api.devnet.solana.com')
      const { blockhash } = await connection.getLatestBlockhash('finalized')
      tx.recentBlockhash = blockhash
      tx.feePayer = publicKey

      const signedTx = await signTransaction(tx)

      const signature = await connection.sendRawTransaction(signedTx.serialize(), {
        skipPreflight: false,
        preflightCommitment: 'confirmed',
      })

      await connection.confirmTransaction(signature, 'confirmed')

      await api.syncTx(publicKey.toBase58(), signature, 'Deposit')
      
      await fetchBalance()

      setStatus({ type: 'success', message: 'Vault initialized successfully!' })
    } catch (err: any) {
      const errorMessage = err.message || ''
      const errorLogs = err.logs || []
      
      if (errorMessage.includes('already in use') || 
          errorLogs.some((log: string) => log.includes('already in use'))) {
        setStatus({ type: 'info', message: 'Vault already exists on-chain. Syncing to database...' })
        
        try {
          await api.forceSyncVault(publicKey.toBase58())
          await fetchBalance()
          setStatus({ type: 'success', message: 'Vault synced successfully! You can now use your vault.' })
        } catch (syncErr: any) {
          setStatus({ 
            type: 'error', 
            message: 'Vault exists but failed to sync: ' + (syncErr.response?.data?.error || syncErr.message)
          })
        }
      } else {
        setStatus({ type: 'error', message: err.response?.data?.error || err.message || 'Failed to initialize vault' })
      }
    } finally {
      setLoading(false)
    }
  }

  const handleDeposit = async () => {
    if (!publicKey || !signTransaction) {
      setStatus({ type: 'error', message: 'Wallet not connected' })
      return
    }

    const amount = parseFloat(depositAmount)
    if (isNaN(amount) || amount <= 0) {
      setStatus({ type: 'error', message: 'Please enter a valid amount' })
      return
    }

    setLoading(true)
    setStatus(null)

    try {
      const amountLamports = Math.floor(amount * 1_000_000)
      
      const res = await api.buildDepositTx(publicKey.toBase58(), amountLamports)
      const { transaction_base64 } = res.data

      const txBytes = Uint8Array.from(atob(transaction_base64), c => c.charCodeAt(0))
      const tx = Transaction.from(txBytes)

      const connection = new Connection('https://api.devnet.solana.com')
      const { blockhash } = await connection.getLatestBlockhash('finalized')
      tx.recentBlockhash = blockhash
      tx.feePayer = publicKey

      const signedTx = await signTransaction(tx)

      const signature = await connection.sendRawTransaction(signedTx.serialize(), {
        skipPreflight: false,
        preflightCommitment: 'confirmed',
      })

      await connection.confirmTransaction(signature, 'confirmed')

      await api.syncTx(publicKey.toBase58(), signature, 'Deposit', amountLamports)
      
      await fetchBalance()
      setDepositAmount('')

      setStatus({ type: 'success', message: 'Deposit successful!' })
    } catch (err: any) {
      setStatus({ type: 'error', message: err.response?.data?.error || err.message || 'Failed to deposit' })
    } finally {
      setLoading(false)
    }
  }

  const handleWithdraw = async () => {
    if (!publicKey || !signTransaction) {
      setStatus({ type: 'error', message: 'Wallet not connected' })
      return
    }

    const amount = parseFloat(withdrawAmount)
    if (isNaN(amount) || amount <= 0) {
      setStatus({ type: 'error', message: 'Please enter a valid amount' })
      return
    }

    if (balance && amount * 1_000_000 > balance.vault.available_balance) {
      setStatus({ type: 'error', message: 'Insufficient available balance' })
      return
    }

    const amountLamports = Math.floor(amount * 1_000_000)

    if (mfaEnabled && balance?.vault.vault_address) {
      setPendingWithdrawal(amountLamports)
      setShowMfaVerify(true)
      return
    }

    await executeWithdrawal(amountLamports)
  }

  const handleMfaVerifyAndWithdraw = async () => {
    if (!balance?.vault.vault_address || !pendingWithdrawal) return
    
    if (mfaCode.length !== 6) {
      setStatus({ type: 'error', message: 'Please enter a 6-digit code' })
      return
    }

    setLoading(true)
    try {
      const mfaRes = await api.checkMfa(balance.vault.vault_address, mfaCode)
      
      if (!mfaRes.data.valid) {
        setStatus({ type: 'error', message: 'Invalid MFA code' })
        setLoading(false)
        return
      }

      setShowMfaVerify(false)
      setMfaCode('')
      await executeWithdrawal(pendingWithdrawal)
      setPendingWithdrawal(null)
    } catch (err: any) {
      setStatus({ 
        type: 'error', 
        message: err.response?.data?.error || 'MFA verification failed' 
      })
      setLoading(false)
    }
  }

  const executeWithdrawal = async (amountLamports: number) => {
    if (!publicKey || !signTransaction) return

    setLoading(true)
    setStatus(null)

    try {
      const res = await api.buildWithdrawTx(publicKey.toBase58(), amountLamports)
      const { transaction_base64 } = res.data

      const txBytes = Uint8Array.from(atob(transaction_base64), c => c.charCodeAt(0))
      const tx = Transaction.from(txBytes)

      const connection = new Connection('https://api.devnet.solana.com')
      const { blockhash } = await connection.getLatestBlockhash('finalized')
      tx.recentBlockhash = blockhash
      tx.feePayer = publicKey

      const signedTx = await signTransaction(tx)

      const signature = await connection.sendRawTransaction(signedTx.serialize(), {
        skipPreflight: false,
        preflightCommitment: 'confirmed',
      })

      await connection.confirmTransaction(signature, 'confirmed')

      await api.syncTx(publicKey.toBase58(), signature, 'Withdrawal', amountLamports)
      
      await fetchBalance()
      setWithdrawAmount('')

      setStatus({ type: 'success', message: 'Withdrawal successful!' })
    } catch (err: any) {
      setStatus({ type: 'error', message: err.response?.data?.error || err.message || 'Failed to withdraw' })
    } finally {
      setLoading(false)
    }
  }

  if (!connected) {
    return (
      <VaultContainer>
        <Card style={{ padding: '2rem', textAlign: 'center' }}>
          <div style={{ color: 'var(--text-secondary)' }}>
            Please connect your Phantom wallet to manage your vault
          </div>
        </Card>
      </VaultContainer>
    )
  }

  return (
    <VaultContainer>
      {balance && (
        <>
          <Section>
            <SectionTitle>Vault Balance</SectionTitle>
            <BalanceDisplay>
              <BalanceItem>
                <BalanceLabel>Total Balance</BalanceLabel>
                <BalanceValue>{formatUSDT(balance.vault.total_balance)}</BalanceValue>
              </BalanceItem>
              <BalanceItem>
                <BalanceLabel>Available</BalanceLabel>
                <BalanceValue>{formatUSDT(balance.vault.available_balance)}</BalanceValue>
              </BalanceItem>
              <BalanceItem>
                <BalanceLabel>Locked</BalanceLabel>
                <BalanceValue>{formatUSDT(balance.vault.locked_balance)}</BalanceValue>
              </BalanceItem>
              <BalanceItem>
                <BalanceLabel>Total Deposited</BalanceLabel>
                <BalanceValue>{formatUSDT(balance.vault.total_deposited)}</BalanceValue>
              </BalanceItem>
            </BalanceDisplay>
          </Section>

          <MfaSection>
            <SectionTitle>Security Settings</SectionTitle>
            <MfaStatusBadge enabled={mfaEnabled}>
              {mfaEnabled ? '2FA Enabled' : '2FA Disabled'}
            </MfaStatusBadge>
            {!mfaEnabled && !showMfaSetup && (
              <>
                <p style={{ color: 'var(--text-secondary)', marginBottom: '1rem' }}>
                  Protect your vault with two-factor authentication for an extra layer of security.
                </p>
                <Button onClick={() => setShowMfaSetup(true)}>
                  Enable Two-Factor Authentication
                </Button>
              </>
            )}
            {showMfaSetup && (
              <MfaSetup 
                vaultAddress={balance.vault.vault_address} 
                onComplete={handleMfaSetupComplete}
              />
            )}
            {mfaEnabled && !showMfaSetup && (
              <Button variant="secondary" onClick={handleDisableMfa} disabled={loading}>
                Disable 2FA
              </Button>
            )}
          </MfaSection>
        </>
      )}

      {!balance && (
        <Section>
          <SectionTitle>Initialize Vault</SectionTitle>
          <p style={{ color: 'var(--text-secondary)', marginBottom: '1.5rem' }}>
            Initialize your vault to start managing collateral
          </p>
          <Button onClick={handleInitialize} disabled={loading}>
            {loading ? 'Initializing...' : 'Initialize Vault'}
          </Button>
        </Section>
      )}

      {balance && (
        <>
          <Section>
            <SectionTitle>Deposit Collateral</SectionTitle>
            <FormGroup>
              <Label>Amount (USDT)</Label>
              <Input
                type="number"
                value={depositAmount}
                onChange={(e) => setDepositAmount(e.target.value)}
                placeholder="0.00"
                step="0.000001"
                min="0"
              />
            </FormGroup>
            <Button onClick={handleDeposit} disabled={loading}>
              {loading ? 'Processing...' : 'Deposit'}
            </Button>
          </Section>

          <Section>
            <SectionTitle>Withdraw Collateral</SectionTitle>
            {mfaEnabled && (
              <MfaStatusBadge enabled={true} style={{ marginBottom: '1rem' }}>
                Protected by 2FA
              </MfaStatusBadge>
            )}
            <FormGroup>
              <Label>Amount (USDT)</Label>
              <Input
                type="number"
                value={withdrawAmount}
                onChange={(e) => setWithdrawAmount(e.target.value)}
                placeholder="0.00"
                step="0.000001"
                min="0"
                max={balance ? (balance.vault.available_balance / 1_000_000).toString() : undefined}
              />
            </FormGroup>
            <Button onClick={handleWithdraw} disabled={loading} variant="secondary">
              {loading ? 'Processing...' : 'Withdraw'}
            </Button>
          </Section>
        </>
      )}

      {status && (
        <StatusMessage type={status.type}>{status.message}</StatusMessage>
      )}

      {showMfaVerify && (
        <MfaModal onClick={() => !loading && setShowMfaVerify(false)}>
          <MfaModalContent onClick={(e) => e.stopPropagation()}>
            <MfaModalTitle>Two-Factor Authentication</MfaModalTitle>
            <p style={{ color: 'var(--text-secondary)', marginBottom: '1.5rem' }}>
              Enter the 6-digit code from your authenticator app to proceed with the withdrawal.
            </p>
            <FormGroup>
              <Label>Authentication Code</Label>
              <Input
                type="text"
                value={mfaCode}
                onChange={(e) => setMfaCode(e.target.value.replace(/\D/g, '').slice(0, 6))}
                placeholder="000000"
                maxLength={6}
                autoFocus
              />
            </FormGroup>
            <ButtonGroup>
              <Button 
                onClick={handleMfaVerifyAndWithdraw} 
                disabled={loading || mfaCode.length !== 6}
              >
                {loading ? 'Verifying...' : 'Verify & Withdraw'}
              </Button>
              <Button 
                variant="secondary" 
                onClick={() => {
                  setShowMfaVerify(false)
                  setMfaCode('')
                  setPendingWithdrawal(null)
                }}
                disabled={loading}
              >
                Cancel
              </Button>
            </ButtonGroup>
          </MfaModalContent>
        </MfaModal>
      )}
    </VaultContainer>
  )
}

