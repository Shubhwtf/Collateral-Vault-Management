import { useState } from 'react'
import { api } from '../api/client'
import styled from 'styled-components'
import Button from './Button'
import Input from './Input'

const MfaContainer = styled.div`
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
`

const QrCodeContainer = styled.div`
  display: flex;
  justify-content: center;
  padding: 1.5rem;
  background-color: white;
  border-radius: 8px;
  margin: 1rem 0;
  
  svg {
    max-width: 250px;
    height: auto;
  }
`

const SecretDisplay = styled.div`
  padding: 1rem;
  background-color: var(--bg-tertiary);
  border-radius: 4px;
  font-family: monospace;
  font-size: 0.875rem;
  word-break: break-all;
  margin: 1rem 0;
`

const BackupCodesContainer = styled.div`
  background-color: var(--bg-tertiary);
  border: 2px solid var(--warning, #f59e0b);
  border-radius: 8px;
  padding: 1.5rem;
  margin: 1rem 0;
`

const BackupCodesList = styled.div`
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 0.5rem;
  margin-top: 1rem;
  font-family: monospace;
  font-size: 0.875rem;
`

const BackupCode = styled.div`
  padding: 0.5rem;
  background-color: var(--bg-secondary);
  border-radius: 4px;
  text-align: center;
`

const Warning = styled.div`
  color: var(--warning, #f59e0b);
  font-weight: 600;
  margin-bottom: 0.5rem;
`

const InfoText = styled.p`
  color: var(--text-secondary);
  font-size: 0.875rem;
  line-height: 1.5;
  margin: 0.5rem 0;
`

const StepTitle = styled.h3`
  font-size: 1.125rem;
  font-weight: 600;
  margin-bottom: 1rem;
  color: var(--text-primary);
`

const FormGroup = styled.div`
  margin-bottom: 1rem;
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
    props.type === 'success' ? 'var(--text-primary)' :
    props.type === 'error' ? 'var(--error, #ef4444)' :
    'var(--text-secondary)'
  };
  font-weight: ${props => props.type === 'success' ? '600' : 'normal'};
`

interface MfaSetupProps {
  vaultAddress: string
  onComplete?: () => void
}

export default function MfaSetup({ vaultAddress, onComplete }: MfaSetupProps) {
  const [step, setStep] = useState<'init' | 'verify' | 'complete'>('init')
  const [qrCode, setQrCode] = useState('')
  const [secret, setSecret] = useState('')
  const [backupCodes, setBackupCodes] = useState<string[]>([])
  const [code, setCode] = useState('')
  const [loading, setLoading] = useState(false)
  const [status, setStatus] = useState<{ type: 'success' | 'error' | 'info', message: string } | null>(null)

  const handleSetup = async () => {
    setLoading(true)
    setStatus(null)
    
    try {
      const response = await api.setupMfa(vaultAddress)
      const data = response.data
      
      setQrCode(data.qr_code_svg)
      setSecret(data.secret)
      setBackupCodes(data.backup_codes)
      setStep('verify')
    } catch (err: any) {
      setStatus({ 
        type: 'error', 
        message: err.response?.data?.error || 'Failed to setup MFA. Please try again.' 
      })
    } finally {
      setLoading(false)
    }
  }

  const handleVerify = async () => {
    if (code.length !== 6) {
      setStatus({ type: 'error', message: 'Please enter a 6-digit code' })
      return
    }

    setLoading(true)
    setStatus(null)
    
    try {
      const response = await api.verifyMfaSetup(vaultAddress, secret, code)
      
      if (response.data.success) {
        setStep('complete')
        setStatus({ type: 'success', message: 'MFA enabled successfully!' })
        setTimeout(() => {
          onComplete?.()
        }, 2000)
      } else {
        setStatus({ type: 'error', message: 'Invalid code. Please try again.' })
      }
    } catch (err: any) {
      setStatus({ 
        type: 'error', 
        message: err.response?.data?.error || 'Verification failed. Please try again.' 
      })
    } finally {
      setLoading(false)
    }
  }

  if (step === 'init') {
    return (
      <MfaContainer>
        <StepTitle>Enable Two-Factor Authentication</StepTitle>
        <InfoText>
          Add an extra layer of security to your vault by enabling two-factor authentication (2FA).
          You'll need an authenticator app like Google Authenticator, Authy, or Microsoft Authenticator.
        </InfoText>
        <Button onClick={handleSetup} disabled={loading}>
          {loading ? 'Setting up...' : 'Enable 2FA'}
        </Button>
        {status && <StatusMessage type={status.type}>{status.message}</StatusMessage>}
      </MfaContainer>
    )
  }

  if (step === 'verify') {
    return (
      <MfaContainer>
        <StepTitle>Scan QR Code</StepTitle>
        <InfoText>
          Scan this QR code with your authenticator app, or enter the secret key manually:
        </InfoText>
        
        <QrCodeContainer>
          <div dangerouslySetInnerHTML={{ __html: qrCode }} />
        </QrCodeContainer>
        
        <div>
          <Label>Manual Entry Key:</Label>
          <SecretDisplay>{secret}</SecretDisplay>
        </div>
        
        <BackupCodesContainer>
          <Warning>Save Your Backup Codes</Warning>
          <InfoText>
            Write down these backup codes and store them in a safe place. 
            You can use them to access your account if you lose your authenticator device.
            Each code can only be used once.
          </InfoText>
          <BackupCodesList>
            {backupCodes.map((code, index) => (
              <BackupCode key={index}>{code}</BackupCode>
            ))}
          </BackupCodesList>
        </BackupCodesContainer>
        
        <FormGroup>
          <Label>Enter 6-digit code from your authenticator app:</Label>
          <Input
            type="text"
            value={code}
            onChange={(e) => setCode(e.target.value.replace(/\D/g, '').slice(0, 6))}
            placeholder="000000"
            maxLength={6}
            pattern="\d{6}"
          />
        </FormGroup>
        
        <ButtonGroup>
          <Button onClick={handleVerify} disabled={loading || code.length !== 6}>
            {loading ? 'Verifying...' : 'Verify & Enable'}
          </Button>
          <Button variant="secondary" onClick={() => setStep('init')}>
            Cancel
          </Button>
        </ButtonGroup>
        
        {status && <StatusMessage type={status.type}>{status.message}</StatusMessage>}
      </MfaContainer>
    )
  }

  if (step === 'complete') {
    return (
      <MfaContainer>
        <StepTitle>Two-Factor Authentication Enabled</StepTitle>
        <InfoText>
          Your vault is now protected with two-factor authentication. 
          You'll need to enter a code from your authenticator app when performing sensitive operations.
        </InfoText>
        <BackupCodesContainer>
          <Warning>Remember Your Backup Codes!</Warning>
          <InfoText>
            Make sure you've saved your backup codes in a secure location. 
            You won't be able to see them again.
          </InfoText>
          <BackupCodesList>
            {backupCodes.map((code, index) => (
              <BackupCode key={index}>{code}</BackupCode>
            ))}
          </BackupCodesList>
        </BackupCodesContainer>
        {status && <StatusMessage type={status.type}>{status.message}</StatusMessage>}
      </MfaContainer>
    )
  }

  return null
}
