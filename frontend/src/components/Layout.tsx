import { ReactNode } from 'react'
import Header from './Header'
import Navigation from './Navigation'
import styled from 'styled-components'

const LayoutContainer = styled.div`
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  background-color: var(--bg-primary);
`

const MainContent = styled.main`
  flex: 1;
  padding: 2rem;
  max-width: 1400px;
  width: 100%;
  margin: 0 auto;
`

interface LayoutProps {
  children: ReactNode
}

export default function Layout({ children }: LayoutProps) {
  return (
    <LayoutContainer>
      <Header />
      <Navigation />
      <MainContent>{children}</MainContent>
    </LayoutContainer>
  )
}

