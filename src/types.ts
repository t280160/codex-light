export interface CodexUsage {
  fiveHourRemaining: number
  weeklyRemaining: number
  fiveHourResetAt: number | null
  weeklyResetAt: number | null
  updatedAt: number
  source: 'api' | 'logs' | string
  stale: boolean
}

export type QuotaStatus = 'green' | 'yellow' | 'red' | 'gray'

