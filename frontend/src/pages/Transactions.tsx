import { useEffect, useState } from 'react'
import { useWallet } from '@solana/wallet-adapter-react'
import { api, TransactionsResponse } from '../api/client'
import Card from '../components/Card'
import styled from 'styled-components'
import { formatUSDT } from '../utils/format'

const TransactionsContainer = styled.div`
  max-width: 1000px;
`

const Table = styled.table`
  width: 100%;
  border-collapse: collapse;
`

const TableHeader = styled.thead`
  border-bottom: 1px solid var(--border-color);
`

const TableRow = styled.tr`
  border-bottom: 1px solid var(--border-color);
  
  &:hover {
    background-color: var(--bg-tertiary);
  }
`

const TableHeaderCell = styled.th`
  padding: 1rem;
  text-align: left;
  color: var(--text-secondary);
  font-size: 0.875rem;
  font-weight: 500;
  text-transform: uppercase;
  letter-spacing: 0.05em;
`

const TableCell = styled.td`
  padding: 1rem;
  color: var(--text-primary);
  font-size: 0.9375rem;
`

const TypeBadge = styled.span<{ type: string }>`
  display: inline-block;
  padding: 0.25rem 0.75rem;
  border-radius: 4px;
  font-size: 0.75rem;
  font-weight: 500;
  text-transform: uppercase;
  background-color: var(--bg-tertiary);
  color: var(--text-primary);
  border: 1px solid var(--border-color);
`

const SignatureLink = styled.a`
  color: var(--text-secondary);
  text-decoration: none;
  font-family: monospace;
  font-size: 0.875rem;
  
  &:hover {
    color: var(--text-primary);
    text-decoration: underline;
  }
`

export default function Transactions() {
  const { publicKey, connected } = useWallet()
  const [transactions, setTransactions] = useState<TransactionsResponse | null>(null)
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    if (connected && publicKey) {
      fetchTransactions()
    } else {
      setLoading(false)
    }
  }, [connected, publicKey])

  const fetchTransactions = async () => {
    if (!publicKey) return
    setLoading(true)
    try {
      const res = await api.getTransactions(publicKey.toBase58())
      setTransactions(res.data)
    } finally {
      setLoading(false)
    }
  }

  if (!connected) {
    return (
      <TransactionsContainer>
        <Card style={{ padding: '2rem', textAlign: 'center' }}>
          <div style={{ color: 'var(--text-secondary)' }}>
            Please connect your Phantom wallet to view transactions
          </div>
        </Card>
      </TransactionsContainer>
    )
  }

  if (loading) {
    return <div>Loading transactions...</div>
  }

  return (
    <TransactionsContainer>
      <Card style={{ padding: '2rem' }}>
        <h2 style={{ marginBottom: '1.5rem', fontSize: '1.5rem', fontWeight: 600 }}>
          Transaction History
        </h2>
        {transactions && transactions.transactions.length > 0 ? (
          <Table>
            <TableHeader>
              <TableRow>
                <TableHeaderCell>Type</TableHeaderCell>
                <TableHeaderCell>Amount</TableHeaderCell>
                <TableHeaderCell>Signature</TableHeaderCell>
                <TableHeaderCell>Time</TableHeaderCell>
              </TableRow>
            </TableHeader>
            <tbody>
              {transactions.transactions.map((tx) => (
                <TableRow key={tx.id}>
                  <TableCell>
                    <TypeBadge type={tx.transaction_type}>
                      {tx.transaction_type}
                    </TypeBadge>
                  </TableCell>
                  <TableCell>{formatUSDT(tx.amount)}</TableCell>
                  <TableCell>
                    <SignatureLink
                      href={`https://solscan.io/tx/${tx.signature}?cluster=devnet`}
                      target="_blank"
                      rel="noopener noreferrer"
                    >
                      {tx.signature.slice(0, 8)}...{tx.signature.slice(-8)}
                    </SignatureLink>
                  </TableCell>
                  <TableCell>
                    {new Date(tx.created_at).toLocaleString()}
                  </TableCell>
                </TableRow>
              ))}
            </tbody>
          </Table>
        ) : (
          <div style={{ color: 'var(--text-secondary)', textAlign: 'center', padding: '2rem' }}>
            No transactions found
          </div>
        )}
      </Card>
    </TransactionsContainer>
  )
}

