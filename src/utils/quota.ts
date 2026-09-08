import type { QuotaStatus } from '../types'

export function getQuotaStatus(percent: number | null | undefined): QuotaStatus {
  if (percent === null || percent === undefined || !Number.isFinite(percent)) return 'gray'
  if (percent >= 50) return 'green'
  if (percent >= 20) return 'yellow'
  return 'red'
}

export function statusLabel(status: QuotaStatus): string {
  return {
    green: 'Healthy',
    yellow: 'Getting low',
    red: 'Low quota',
    gray: 'Unavailable',
  }[status]
}

export function formatResetTime(resetAt: number | null, now = Date.now()): string {
  if (!resetAt) return 'Reset time unavailable'
  const minutes = Math.max(0, Math.floor((resetAt * 1000 - now) / 60_000))
  const days = Math.floor(minutes / (24 * 60))
  const hours = Math.floor((minutes % (24 * 60)) / 60)
  const remainingMinutes = minutes % 60
  if (days > 0) return `Reset in ${days}d ${hours}h ${remainingMinutes}m`
  return `Reset in ${hours}h ${remainingMinutes}m`
}

export function formatUpdatedAt(timestamp: number): string {
  return new Intl.DateTimeFormat(undefined, {
    hour: '2-digit',
    minute: '2-digit',
  }).format(new Date(timestamp * 1000))
}

