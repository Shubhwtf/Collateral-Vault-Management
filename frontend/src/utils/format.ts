export function formatUSDT(lamports: number, short: boolean = false): string {
  const usdt = lamports / 1_000_000 // USDT has 6 decimals
  
  if (short) {
    // For chart labels - shortened format
    if (usdt >= 1_000_000) {
      return `$${(usdt / 1_000_000).toFixed(1)}M`
    } else if (usdt >= 1_000) {
      return `$${(usdt / 1_000).toFixed(1)}K`
    } else {
      return `$${usdt.toFixed(0)}`
    }
  }
  
  return new Intl.NumberFormat('en-US', {
    minimumFractionDigits: 2,
    maximumFractionDigits: 6,
  }).format(usdt)
}

export function formatNumber(num: number): string {
  return new Intl.NumberFormat('en-US').format(num)
}

