<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { invoke } from '@tauri-apps/api/core'
import type { CodexUsage } from './types'
import { formatResetTime, formatUpdatedAt, getQuotaStatus, statusLabel } from './utils/quota'

const usage = ref<CodexUsage | null>(null)
const loading = ref(false)
const error = ref<string | null>(null)
const now = ref(Date.now())
let ticker: number | undefined
let refreshTicker: number | undefined

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

function resetText(resetAt: number | null | undefined) {
  return formatResetTime(resetAt ?? null, now.value)
}

onMounted(() => {
  refresh()
  ticker = window.setInterval(() => {
    now.value = Date.now()
  }, 30_000)
  refreshTicker = window.setInterval(refresh, 5 * 60_000)
})

onBeforeUnmount(() => {
  if (ticker) window.clearInterval(ticker)
  if (refreshTicker) window.clearInterval(refreshTicker)
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

      <p v-if="usage.stale" class="notice warning">Last known usage · may be out of date</p>
      <p v-else class="updated">Updated {{ formatUpdatedAt(usage.updatedAt) }}</p>
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
