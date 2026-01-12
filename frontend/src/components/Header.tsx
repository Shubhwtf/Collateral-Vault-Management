import { useWallet } from '@solana/wallet-adapter-react'
import { WalletMultiButton } from '@solana/wallet-adapter-react-ui'
import styled from 'styled-components'

const HeaderContainer = styled.header`
  background-color: var(--bg-secondary);
  border-bottom: 1px solid var(--border-color);
  padding: 1rem 2rem;
  display: flex;
  justify-content: space-between;
  align-items: center;
`

const Logo = styled.h1`
  font-size: 1.5rem;
  font-weight: 600;
  letter-spacing: -0.02em;
  color: var(--text-primary);
`

const WalletSection = styled.div`
  display: flex;
  align-items: center;
  gap: 1rem;
`

const WalletInfo = styled.div`
  color: var(--text-secondary);
  font-size: 0.875rem;
`

export default function Header() {
  const { publicKey, connected } = useWallet()

  return (
    <HeaderContainer>
      <Logo>Collateral Vault</Logo>
      <WalletSection>
        {connected && publicKey && (
          <WalletInfo>
            {publicKey.toBase58().slice(0, 4)}...{publicKey.toBase58().slice(-4)}
          </WalletInfo>
        )}
        <WalletMultiButton />
      </WalletSection>
    </HeaderContainer>
  )
}

