import { useEffect, useState } from 'react'
import { useWallet } from '@solana/wallet-adapter-react'
import { api, BalanceResponse, AnalyticsOverview } from '../api/client'
import styled from 'styled-components'
import Card from '../components/Card'
import { formatUSDT } from '../utils/format'

const DashboardContainer = styled.div`
  display: flex;
  flex-direction: column;
  gap: 2rem;
`

const StatsGrid = styled.div`
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 1.5rem;
`

const StatCard = styled(Card)`
  padding: 1.5rem;
`

const StatLabel = styled.div`
  color: var(--text-secondary);
  font-size: 0.875rem;
  margin-bottom: 0.5rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
`

const StatValue = styled.div`
  font-size: 2rem;
  font-weight: 600;
  color: var(--text-primary);
`

const Section = styled.section`
  margin-top: 2rem;
`

const SectionTitle = styled.h2`
  font-size: 1.5rem;
  font-weight: 600;
  margin-bottom: 1.5rem;
  color: var(--text-primary);
`

const BalanceCard = styled(Card)`
  padding: 2rem;
`

const BalanceGrid = styled.div`
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 1.5rem;
  margin-top: 1.5rem;
`

const BalanceItem = styled.div`
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
`

const BalanceLabel = styled.div`
  color: var(--text-secondary);
  font-size: 0.875rem;
`

const BalanceValue = styled.div`
  font-size: 1.5rem;
  font-weight: 600;
  color: var(--text-primary);
`

const ErrorMessage = styled.div`
  color: var(--error);
  padding: 1rem;
  background-color: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: 4px;
`

export default function Dashboard() {
  const { publicKey, connected } = useWallet()
  const [balance, setBalance] = useState<BalanceResponse | null>(null)
  const [tvl, setTvl] = useState<number>(0)
  const [analytics, setAnalytics] = useState<AnalyticsOverview | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    const fetchData = async () => {
      setLoading(true)
      setError(null)

      try {
        const [tvlRes, analyticsRes] = await Promise.all([
          api.getTvl(),
          api.getAnalyticsOverview(),
        ])

        setTvl(tvlRes.data.total_value_locked)
        setAnalytics(analyticsRes.data)

        if (connected && publicKey) {
          try {
            const balanceRes = await api.getBalance(publicKey.toBase58())
            setBalance(balanceRes.data)
          } catch (err: any) {
            // Silently handle 404 when vault doesn't exist yet
          }
        }
      } catch (err: any) {
        setError(err.message || 'Failed to load data')
      } finally {
        setLoading(false)
      }
    }

    fetchData()
    const interval = setInterval(fetchData, 30000) // Refresh every 30 seconds
    return () => clearInterval(interval)
  }, [connected, publicKey])

  if (loading) {
    return <div>Loading...</div>
  }

  return (
    <DashboardContainer>
      <StatsGrid>
        <StatCard>
          <StatLabel>Total Value Locked</StatLabel>
          <StatValue>{formatUSDT(tvl)}</StatValue>
        </StatCard>
        <StatCard>
          <StatLabel>Total Users</StatLabel>
          <StatValue>{analytics?.total_users || 0}</StatValue>
        </StatCard>
        <StatCard>
          <StatLabel>Active Vaults</StatLabel>
          <StatValue>{analytics?.active_vaults || 0}</StatValue>
        </StatCard>
        <StatCard>
          <StatLabel>Total Deposits</StatLabel>
          <StatValue>{formatUSDT(analytics?.total_deposits || 0)}</StatValue>
        </StatCard>
      </StatsGrid>

      {error && <ErrorMessage>{error}</ErrorMessage>}

      {connected && publicKey && (
        <Section>
          <SectionTitle>Your Vault</SectionTitle>
          <BalanceCard>
            {balance ? (
              <BalanceGrid>
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
              </BalanceGrid>
            ) : (
              <div style={{ color: 'var(--text-secondary)' }}>
                No vault found. Initialize your vault to get started.
              </div>
            )}
          </BalanceCard>
        </Section>
      )}

      {!connected && (
        <Card style={{ padding: '2rem', textAlign: 'center' }}>
          <div style={{ color: 'var(--text-secondary)', marginBottom: '1rem' }}>
            Connect your Phantom wallet to view your vault
          </div>
        </Card>
      )}
    </DashboardContainer>
  )
}

