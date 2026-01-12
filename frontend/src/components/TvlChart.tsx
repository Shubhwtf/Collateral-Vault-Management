import { useEffect, useState } from 'react'
import { api, TvlChartDataPoint } from '../api/client'
import styled from 'styled-components'
import { formatUSDT, formatNumber } from '../utils/format'

const ChartContainer = styled.div`
  display: flex;
  flex-direction: column;
  gap: 1rem;
`

const ChartHeader = styled.div`
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1rem;
`

const ChartTitle = styled.h3`
  font-size: 1.125rem;
  font-weight: 600;
  color: var(--text-primary);
`

const TimeRangeSelector = styled.div`
  display: flex;
  gap: 0.5rem;
`

const TimeRangeButton = styled.button<{ active: boolean }>`
  padding: 0.5rem 1rem;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  background-color: ${props => props.active ? 'var(--primary, #3b82f6)' : 'var(--bg-secondary)'};
  color: ${props => props.active ? 'white' : 'var(--text-secondary)'};
  cursor: pointer;
  font-size: 0.875rem;
  font-weight: 500;
  transition: all 0.2s;

  &:hover {
    background-color: ${props => props.active ? 'var(--primary-dark, #2563eb)' : 'var(--bg-tertiary)'};
  }
`

const ChartSvg = styled.svg`
  width: 100%;
  height: 300px;
  background-color: var(--bg-secondary);
  border-radius: 8px;
  padding: 1rem;
`

const Tooltip = styled.div<{ x: number; y: number }>`
  position: absolute;
  left: ${props => props.x}px;
  top: ${props => props.y}px;
  background-color: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  padding: 0.75rem;
  pointer-events: none;
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
  z-index: 1000;
  white-space: nowrap;
`

const TooltipLabel = styled.div`
  font-size: 0.75rem;
  color: var(--text-secondary);
  margin-bottom: 0.25rem;
`

const TooltipValue = styled.div`
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--text-primary);
`

const LoadingMessage = styled.div`
  text-align: center;
  padding: 2rem;
  color: var(--text-secondary);
`

const ErrorMessage = styled.div`
  text-align: center;
  padding: 2rem;
  color: var(--error, #ef4444);
`

const NoDataMessage = styled.div`
  text-align: center;
  padding: 2rem;
  color: var(--text-secondary);
`

export default function TvlChart() {
  const [data, setData] = useState<TvlChartDataPoint[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [timeRange, setTimeRange] = useState<7 | 30 | 90>(30)
  const [tooltip, setTooltip] = useState<{ x: number; y: number; data: TvlChartDataPoint } | null>(null)

  useEffect(() => {
    fetchChartData()
  }, [timeRange])

  const fetchChartData = async () => {
    setLoading(true)
    setError(null)
    
    try {
      const response = await api.getTvlChart(timeRange)
      setData(response.data)
    } catch (err: any) {
      setError(err.response?.data?.error || 'Failed to load chart data')
    } finally {
      setLoading(false)
    }
  }

  const renderChart = () => {
    if (loading) return <LoadingMessage>Loading chart data...</LoadingMessage>
    if (error) return <ErrorMessage>{error}</ErrorMessage>
    if (data.length === 0) return <NoDataMessage>No data available yet. Check back later!</NoDataMessage>

    // Handle single data point case
    if (data.length === 1) {
      const point = data[0]
      const date = new Date(point.timestamp)
      return (
        <div style={{ textAlign: 'center', padding: '3rem', backgroundColor: 'var(--bg-secondary)', borderRadius: '8px' }}>
          <div style={{ fontSize: '2rem', fontWeight: '600', color: 'var(--text-primary)', marginBottom: '0.5rem' }}>
            {formatUSDT(point.tvl)}
          </div>
          <div style={{ color: 'var(--text-secondary)', marginBottom: '0.25rem' }}>
            Total Value Locked
          </div>
          <div style={{ color: 'var(--text-secondary)', fontSize: '0.875rem' }}>
            {date.toLocaleDateString('en-US', { month: 'long', day: 'numeric', year: 'numeric' })}
          </div>
          <div style={{ color: 'var(--text-secondary)', fontSize: '0.875rem', marginTop: '1rem' }}>
            {point.user_count} {point.user_count === 1 ? 'user' : 'users'}
          </div>
        </div>
      )
    }

    const width = 800 // Use fixed pixel width
    const height = 300
    const padding = { top: 20, right: 20, bottom: 40, left: 70 }

    const maxTvl = Math.max(...data.map(d => d.tvl))
    const minTvl = Math.min(...data.map(d => d.tvl))
    const tvlRange = maxTvl - minTvl || maxTvl * 0.1 || 1 // Handle flat line case

    const xScale = (index: number) => 
      padding.left + ((width - padding.left - padding.right) * index) / (data.length - 1)
    
    const yScale = (value: number) => 
      height - padding.bottom - ((height - padding.top - padding.bottom) * (value - minTvl)) / tvlRange

    // Generate path for the line
    const pathData = data.map((point, index) => {
      const x = xScale(index)
      const y = yScale(point.tvl)
      return `${index === 0 ? 'M' : 'L'} ${x} ${y}`
    }).join(' ')

    // Generate area fill
    const areaData = `${pathData} L ${xScale(data.length - 1)} ${height - padding.bottom} L ${padding.left} ${height - padding.bottom} Z`

    return (
      <div style={{ position: 'relative' }}>
        <ChartSvg viewBox={`0 0 ${width} ${height}`} preserveAspectRatio="xMidYMid meet">
          {/* Grid lines */}
          {[0, 0.25, 0.5, 0.75, 1].map((ratio, i) => {
            const y = padding.top + (height - padding.top - padding.bottom) * ratio
            return (
              <line
                key={i}
                x1={padding.left}
                y1={y}
                x2={width - padding.right}
                y2={y}
                stroke="var(--border-color)"
                strokeWidth="1"
                opacity="0.3"
              />
            )
          })}

          {/* Area fill */}
          <path
            d={areaData}
            fill="var(--primary, #3b82f6)"
            opacity="0.1"
          />

          {/* Line */}
          <path
            d={pathData}
            fill="none"
            stroke="var(--primary, #3b82f6)"
            strokeWidth="2"
          />

          {/* Data points */}
          {data.map((point, index) => (
            <circle
              key={index}
              cx={xScale(index)}
              cy={yScale(point.tvl)}
              r="4"
              fill="var(--primary, #3b82f6)"
              style={{ cursor: 'pointer' }}
              onMouseEnter={(e) => {
                const rect = e.currentTarget.getBoundingClientRect()
                setTooltip({
                  x: rect.left + window.scrollX,
                  y: rect.top + window.scrollY - 80,
                  data: point
                })
              }}
              onMouseLeave={() => setTooltip(null)}
            />
          ))}

          {/* Y-axis labels */}
          {[0, 0.25, 0.5, 0.75, 1].map((ratio, i) => {
            const value = minTvl + tvlRange * (1 - ratio)
            const y = padding.top + (height - padding.top - padding.bottom) * ratio
            return (
              <text
                key={i}
                x={padding.left - 10}
                y={y}
                textAnchor="end"
                fill="var(--text-secondary)"
                fontSize="11"
                dominantBaseline="middle"
              >
                {formatUSDT(value, true)}
              </text>
            )
          })}

          {/* X-axis labels */}
          {data.filter((_, i) => i % Math.ceil(data.length / Math.min(5, data.length)) === 0).map((point, i) => {
            const index = data.indexOf(point)
            const x = xScale(index)
            const date = new Date(point.timestamp)
            return (
              <text
                key={i}
                x={x}
                y={height - padding.bottom + 20}
                textAnchor="middle"
                fill="var(--text-secondary)"
                fontSize="11"
              >
                {date.toLocaleDateString('en-US', { month: 'short', day: 'numeric' })}
              </text>
            )
          })}
        </ChartSvg>

        {tooltip && (
          <Tooltip x={tooltip.x} y={tooltip.y}>
            <TooltipLabel>{new Date(tooltip.data.timestamp).toLocaleDateString()}</TooltipLabel>
            <TooltipValue>TVL: {formatUSDT(tooltip.data.tvl)}</TooltipValue>
            <TooltipValue>Users: {formatNumber(tooltip.data.user_count)}</TooltipValue>
          </Tooltip>
        )}
      </div>
    )
  }

  return (
    <ChartContainer>
      <ChartHeader>
        <ChartTitle>Total Value Locked Over Time</ChartTitle>
        <TimeRangeSelector>
          <TimeRangeButton active={timeRange === 7} onClick={() => setTimeRange(7)}>
            7D
          </TimeRangeButton>
          <TimeRangeButton active={timeRange === 30} onClick={() => setTimeRange(30)}>
            30D
          </TimeRangeButton>
          <TimeRangeButton active={timeRange === 90} onClick={() => setTimeRange(90)}>
            90D
          </TimeRangeButton>
        </TimeRangeSelector>
      </ChartHeader>
      {renderChart()}
    </ChartContainer>
  )
}
