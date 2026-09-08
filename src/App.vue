<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { CodexUsage } from './types'
import { formatResetTime, formatUpdatedAt, getQuotaStatus, statusLabel } from './utils/quota'

const usage = ref<CodexUsage | null>(null)
const loading = ref(false)
const error = ref<string | null>(null)
const settingsError = ref<string | null>(null)
const now = ref(Date.now())
const refreshIntervalMinutes = ref(5)
const refreshIntervalOptions = [1, 5, 10, 15, 30, 60]
const savingRefreshInterval = ref(false)
let ticker: number | undefined
let stopUsageListener: UnlistenFn | undefined

const status = computed(() => getQuotaStatus(usage.value?.fiveHourRemaining))
const statusText = computed(() => statusLabel(status.value))

async function refresh() {
  if (loading.value) return
  loading.value = true
  error.value = null
  try {
    usage.value = await invoke<CodexUsage>('refresh_codex_usage')
  } catch (reason) {
    error.value = typeof reason === 'string' ? reason : 'Unable to refresh Codex usage'
  } finally {
    loading.value = false
  }
}

async function closePopup() {
  await getCurrentWindow().hide()
}

async function loadRefreshInterval() {
  try {
    refreshIntervalMinutes.value = await invoke<number>('get_refresh_interval_minutes')
  } catch (reason) {
    settingsError.value = typeof reason === 'string' ? reason : 'Unable to load refresh interval'
  }
}

async function changeRefreshInterval(event: Event) {
  const previous = refreshIntervalMinutes.value
  const minutes = Number((event.target as HTMLSelectElement).value)
  refreshIntervalMinutes.value = minutes
  savingRefreshInterval.value = true
  settingsError.value = null
  try {
    refreshIntervalMinutes.value = await invoke<number>('set_refresh_interval_minutes', { minutes })
  } catch (reason) {
    refreshIntervalMinutes.value = previous
    settingsError.value = typeof reason === 'string' ? reason : 'Unable to save refresh interval'
  } finally {
    savingRefreshInterval.value = false
  }
}

function resetText(resetAt: number | null | undefined) {
  return formatResetTime(resetAt ?? null, now.value)
}

onMounted(async () => {
  stopUsageListener = await listen<CodexUsage>('codex-usage-updated', (event) => {
    usage.value = event.payload
    error.value = null
  })
  refresh()
  loadRefreshInterval()
  ticker = window.setInterval(() => {
    now.value = Date.now()
  }, 30_000)
})

onBeforeUnmount(() => {
  if (ticker) window.clearInterval(ticker)
  stopUsageListener?.()
})
</script>

<template>
  <main class="popup-shell">
    <header class="topbar" data-tauri-drag-region="deep">
      <div class="brand-lockup">
        <span>CodexLight</span>
      </div>
      <button class="icon-button" type="button" aria-label="Close" @click="closePopup">×</button>
    </header>

    <section v-if="usage" class="quota-stack" aria-live="polite">
      <div class="status-banner" :class="`banner-${status}`">
        <span class="traffic-dot" :class="`dot-${status}`" aria-hidden="true"></span>
        <span>{{ statusText }}</span>
        <strong>{{ usage.fiveHourRemaining }}%</strong>
      </div>

      <article class="quota-card">
        <div class="quota-heading">
          <span>5 Hour</span>
          <strong>{{ usage.fiveHourRemaining }}%</strong>
        </div>
        <p>{{ resetText(usage.fiveHourResetAt) }}</p>
      </article>

      <article class="quota-card">
        <div class="quota-heading">
          <span>Weekly</span>
          <strong>{{ usage.weeklyRemaining }}%</strong>
        </div>
        <p>{{ resetText(usage.weeklyResetAt) }}</p>
      </article>

      <div class="meta-row">
        <label class="refresh-setting">
          <span>Auto</span>
          <select
            :value="refreshIntervalMinutes"
            :disabled="savingRefreshInterval"
            aria-label="Automatic refresh interval"
            @change="changeRefreshInterval"
          >
            <option v-for="minutes in refreshIntervalOptions" :key="minutes" :value="minutes">
              {{ minutes }} min
            </option>
          </select>
        </label>
        <p v-if="usage.stale" class="notice warning">Last known · may be stale</p>
        <p v-else class="updated">Updated {{ formatUpdatedAt(usage.updatedAt) }}</p>
      </div>
      <p v-if="settingsError" class="settings-error">{{ settingsError }}</p>
    </section>

    <section v-else class="empty-state" aria-live="polite">
      <span class="traffic-dot dot-gray" aria-hidden="true"></span>
      <strong>{{ loading ? 'Checking Codex…' : 'Codex usage unavailable' }}</strong>
      <p>{{ error ?? 'No usage data yet.' }}</p>
    </section>

    <footer class="actions">
      <button class="refresh-button" type="button" :disabled="loading" @click="refresh">
        <span>{{ loading ? 'Refreshing…' : 'Refresh' }}</span>
        <span v-if="!loading" aria-hidden="true">↻</span>
      </button>
    </footer>
  </main>
</template>
