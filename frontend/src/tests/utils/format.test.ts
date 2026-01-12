import { describe, it, expect } from 'vitest'
import { formatUSDT, formatNumber } from '../../utils/format'

describe('format utilities', () => {
  describe('formatUSDT', () => {
    it('formats USDT amounts correctly', () => {
      expect(formatUSDT(1234560000)).toBe('1,234.56')
      expect(formatUSDT(1000000000000)).toBe('1,000,000.00')
    })

    it('handles zero values', () => {
      expect(formatUSDT(0)).toBe('0.00')
    })

    it('formats short version for large amounts', () => {
      expect(formatUSDT(1000000000000, true)).toBe('$1.0M')
      expect(formatUSDT(5000000000, true)).toBe('$5.0K')
    })

    it('formats small amounts in short mode', () => {
      expect(formatUSDT(500000000, true)).toBe('$500')
    })
  })

  describe('formatNumber', () => {
    it('formats numbers with commas', () => {
      expect(formatNumber(1234567)).toBe('1,234,567')
      expect(formatNumber(1000)).toBe('1,000')
    })

    it('handles zero', () => {
      expect(formatNumber(0)).toBe('0')
    })
  })
})
