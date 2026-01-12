import { useEffect, useState } from 'react'
import { api, AnalyticsOverview } from '../api/client'
import Card from '../components/Card'
import TvlChart from '../components/TvlChart'
import styled from 'styled-components'
import { formatNumber, formatUSDT } from '../utils/format'

const AnalyticsContainer = styled.div`
  display: flex;
  flex-direction: column;
  gap: 2rem;
`

const StatsGrid = styled.div`
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
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

const ChartSection = styled(Card)`
  padding: 2rem;
`

export default function Analytics() {
  const [analytics, setAnalytics] = useState<AnalyticsOverview | null>(null)
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    fetchAnalytics()
    const interval = setInterval(fetchAnalytics, 30000)
    return () => clearInterval(interval)
  }, [])

  const fetchAnalytics = async () => {
    try {
      const res = await api.getAnalyticsOverview()
      setAnalytics(res.data)
    } finally {
      setLoading(false)
    }
  }

  if (loading) {
    return <div>Loading analytics...</div>
  }

  if (!analytics) {
    return <div>Failed to load analytics</div>
  }

  return (
    <AnalyticsContainer>
      <ChartSection>
        <TvlChart />
      </ChartSection>

      <StatsGrid>
        <StatCard>
          <StatLabel>Total Value Locked</StatLabel>
          <StatValue>{formatUSDT(analytics.total_value_locked)}</StatValue>
        </StatCard>
        <StatCard>
          <StatLabel>Total Users</StatLabel>
          <StatValue>{formatNumber(analytics.total_users)}</StatValue>
        </StatCard>
        <StatCard>
          <StatLabel>Active Vaults</StatLabel>
          <StatValue>{formatNumber(analytics.active_vaults)}</StatValue>
        </StatCard>
        <StatCard>
          <StatLabel>Total Deposits</StatLabel>
          <StatValue>{formatUSDT(analytics.total_deposits)}</StatValue>
        </StatCard>
        <StatCard>
          <StatLabel>Total Withdrawals</StatLabel>
          <StatValue>{formatUSDT(analytics.total_withdrawals)}</StatValue>
        </StatCard>
        <StatCard>
          <StatLabel>Average Balance</StatLabel>
          <StatValue>{formatUSDT(analytics.average_balance)}</StatValue>
        </StatCard>
      </StatsGrid>
    </AnalyticsContainer>
  )
}

