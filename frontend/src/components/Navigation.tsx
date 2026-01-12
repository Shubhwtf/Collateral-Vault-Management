import { NavLink } from 'react-router-dom'
import styled from 'styled-components'

const Nav = styled.nav`
  background-color: var(--bg-secondary);
  border-bottom: 1px solid var(--border-color);
  padding: 0 2rem;
`

const NavList = styled.ul`
  list-style: none;
  display: flex;
  gap: 0;
`

const NavItem = styled.li`
  margin: 0;
`

const StyledNavLink = styled(NavLink)`
  display: block;
  padding: 1rem 1.5rem;
  color: var(--text-secondary);
  text-decoration: none;
  border-bottom: 2px solid transparent;
  transition: all 0.2s ease;
  font-weight: 500;
  font-size: 0.9375rem;

  &:hover {
    color: var(--text-primary);
    background-color: var(--bg-tertiary);
  }

  &.active {
    color: var(--text-primary);
    border-bottom-color: var(--accent);
  }
`

export default function Navigation() {
  return (
    <Nav>
      <NavList>
        <NavItem>
          <StyledNavLink to="/">Dashboard</StyledNavLink>
        </NavItem>
        <NavItem>
          <StyledNavLink to="/vault">Vault</StyledNavLink>
        </NavItem>
        <NavItem>
          <StyledNavLink to="/transactions">Transactions</StyledNavLink>
        </NavItem>
        <NavItem>
          <StyledNavLink to="/analytics">Analytics</StyledNavLink>
        </NavItem>
        <NavItem>
          <StyledNavLink to="/yield">Yield</StyledNavLink>
        </NavItem>
      </NavList>
    </Nav>
  )
}

