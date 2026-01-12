import { BrowserRouter as Router, Routes, Route } from 'react-router-dom'
import { WalletProvider } from './contexts/WalletContext'
import Layout from './components/Layout'
import Dashboard from './pages/Dashboard'
import Vault from './pages/Vault'
import Transactions from './pages/Transactions'
import Analytics from './pages/Analytics'
import Yield from './pages/Yield'

function App() {
  return (
    <WalletProvider>
      <Router>
        <Layout>
          <Routes>
            <Route path="/" element={<Dashboard />} />
            <Route path="/vault" element={<Vault />} />
            <Route path="/transactions" element={<Transactions />} />
            <Route path="/analytics" element={<Analytics />} />
            <Route path="/yield" element={<Yield />} />
          </Routes>
        </Layout>
      </Router>
    </WalletProvider>
  )
}

export default App

