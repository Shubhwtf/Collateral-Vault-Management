import axios from 'axios'

const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:8080'

export const apiClient = axios.create({
  baseURL: API_BASE_URL,
  headers: {
    'Content-Type': 'application/json',
  },
})

export interface BalanceResponse {
  vault: {
    owner: string
    vault_address: string
    total_balance: number
    locked_balance: number
    available_balance: number
    total_deposited: number
    total_withdrawn: number
    created_at: string
    updated_at: string
  }
}

export interface BuildTxResponse {
  transaction_base64: string
  recent_blockhash: string
  fee_payer: string
}

export interface TransactionsResponse {
  transactions: Array<{
    id: number
    vault_address: string
    transaction_type: 'deposit' | 'withdrawal' | 'lock' | 'unlock' | 'transfer'
    amount: number
    signature: string
    created_at: string
  }>
}

export interface TvlResponse {
  total_value_locked: number
}

export interface AnalyticsOverview {
  total_value_locked: number
  total_users: number
  total_deposits: number
  total_withdrawals: number
  active_vaults: number
  average_balance: number
  total_yield_earned: number
}

export interface TvlChartDataPoint {
  timestamp: string
  tvl: number
  user_count: number
}

export interface MfaSetupResponse {
  qr_code_svg: string
  secret: string
  backup_codes: string[]
}

export interface MfaVerifyResponse {
  success: boolean
  backup_codes: string[]
}

export interface MfaStatusResponse {
  mfa_enabled: boolean
}

export interface MfaCheckResponse {
  valid: boolean
}

export interface ForceSyncResponse {
  vault: {
    owner: string
    vault_address: string
    total_balance: number
    locked_balance: number
    available_balance: number
    total_deposited: number
    total_withdrawn: number
    created_at: string
    updated_at: string
  }
  recorded: boolean
}

export interface YieldInfoResponse {
  vault_address: string
  yield_enabled: boolean
  total_yield_earned: number
  last_yield_compound: number
  estimated_next_yield: number
  time_until_next_compound: number
}

export interface SyncYieldResponse {
  vault: {
    owner: string
    vault_address: string
    total_balance: number
    locked_balance: number
    available_balance: number
    total_deposited: number
    total_withdrawn: number
    created_at: string
    updated_at: string
  }
  success: boolean
}

export const api = {
  health: () => apiClient.get('/health'),
  
  getBalance: (userPubkey: string) =>
    apiClient.get<BalanceResponse>(`/vault/balance/${userPubkey}`),
  
  getTransactions: (userPubkey: string) =>
    apiClient.get<TransactionsResponse>(`/vault/transactions/${userPubkey}`),
  
  getTvl: () => apiClient.get<TvlResponse>('/vault/tvl'),
  
  buildInitializeTx: (userPubkey: string) =>
    apiClient.post<BuildTxResponse>('/vault/initialize', {
      user_pubkey: userPubkey,
    }),
  
  buildDepositTx: (userPubkey: string, amount: number) =>
    apiClient.post<BuildTxResponse>('/vault/deposit', {
      user_pubkey: userPubkey,
      amount,
    }),
  
  buildWithdrawTx: (userPubkey: string, amount: number) =>
    apiClient.post<BuildTxResponse>('/vault/withdraw', {
      user_pubkey: userPubkey,
      amount,
    }),
  
  syncTx: (userPubkey: string, signature: string, transactionType: string, amount?: number) =>
    apiClient.post('/vault/sync', {
      user_pubkey: userPubkey,
      signature,
      transaction_type: transactionType,
      amount,
    }),
  
  getAnalyticsOverview: () =>
    apiClient.get<AnalyticsOverview>('/analytics/overview'),
  
  // Historical TVL Chart
  getTvlChart: (days: number = 30) =>
    apiClient.get<TvlChartDataPoint[]>(`/analytics/chart/tvl?days=${days}`),
  
  // MFA Endpoints
  setupMfa: (vaultAddress: string) =>
    apiClient.post<MfaSetupResponse>('/mfa/setup', { vault_address: vaultAddress }),
  
  verifyMfaSetup: (vaultAddress: string, secret: string, code: string) =>
    apiClient.post<MfaVerifyResponse>('/mfa/verify-setup', {
      vault_address: vaultAddress,
      secret,
      code,
    }),
  
  getMfaStatus: (vaultAddress: string) =>
    apiClient.get<MfaStatusResponse>(`/mfa/status/${vaultAddress}`),
  
  checkMfa: (vaultAddress: string, code: string) =>
    apiClient.post<MfaCheckResponse>('/mfa/check', {
      vault_address: vaultAddress,
      code,
    }),
  
  disableMfa: (vaultAddress: string, code: string) =>
    apiClient.post<{ success: boolean }>('/mfa/disable', {
      vault_address: vaultAddress,
      code,
    }),
  
  // Force sync an existing on-chain vault to the database
  forceSyncVault: (userPubkey: string) =>
    apiClient.post<ForceSyncResponse>('/vault/force-sync', {
      user_pubkey: userPubkey,
    }),
  
  getPublicConfig: () => apiClient.get('/config/public'),
  
  // Yield Endpoints
  getYieldInfo: (userPubkey: string) =>
    apiClient.get<YieldInfoResponse>(`/yield/info/${userPubkey}`),
  
  buildCompoundYieldTx: (userPubkey: string) =>
    apiClient.post<BuildTxResponse>('/yield/compound', {
      user_pubkey: userPubkey,
    }),
  
  buildConfigureYieldTx: (userPubkey: string, enabled: boolean) =>
    apiClient.post<BuildTxResponse>('/yield/configure', {
      user_pubkey: userPubkey,
      enabled,
    }),
  
  syncYieldTx: (userPubkey: string, signature: string) =>
    apiClient.post<SyncYieldResponse>('/yield/sync', {
      user_pubkey: userPubkey,
      signature,
    }),
}

