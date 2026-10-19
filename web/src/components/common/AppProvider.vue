<template>
  <n-config-provider
    wh-full
    :locale="naiveLocale.locale"
    :date-locale="naiveLocale.date"
    :theme="appStore.isDark ? darkTheme : undefined"
    :theme-overrides="naiveThemeOverrides"
  >
    <n-loading-bar-provider>
      <n-dialog-provider>
        <n-notification-provider>
          <n-message-provider>
            <slot></slot>
            <NaiveProviderContent />
          </n-message-provider>
        </n-notification-provider>
      </n-dialog-provider>
    </n-loading-bar-provider>
  </n-config-provider>
</template>

<script setup>
import { defineComponent, h } from 'vue'
import {
  zhCN,
  dateZhCN,
  enUS,
  dateEnUS,
  uzUZ,
  dateUzUZ,
  darkTheme,
  useLoadingBar,
  useDialog,
  useMessage,
  useNotification,
} from 'naive-ui'
import { useCssVar } from '@vueuse/core'
import { useI18n } from 'vue-i18n'
import { kebabCase } from 'lodash-es'
import { setupMessage, setupDialog } from '@/utils'
import { naiveThemeOverrides } from '~/settings'
import { useAppStore } from '@/stores'

const appStore = useAppStore()
const { t } = useI18n({ useScope: 'global' })

const NAIVE_LOCALES = {
  en: { locale: enUS, date: dateEnUS },
  uz: { locale: uzUZ, date: dateUzUZ },
  cn: { locale: zhCN, date: dateZhCN },
}
const naiveLocale = NAIVE_LOCALES[appStore.locale] || NAIVE_LOCALES.en

function setupCssVar() {
  const common = naiveThemeOverrides.common
  for (const key in common) {
    useCssVar(`--${kebabCase(key)}`, document.documentElement).value = common[key] || ''
    if (key === 'primaryColor') window.localStorage.setItem('__THEME_COLOR__', common[key] || '')
  }
}

// Expose the naive helpers on window so they work anywhere
function setupNaiveTools() {
  window.$loadingBar = useLoadingBar()
  window.$notification = useNotification()

  window.$message = setupMessage(useMessage())
  window.$dialog = setupDialog(useDialog(), t)
}

const NaiveProviderContent = defineComponent({
  setup() {
    setupCssVar()
    setupNaiveTools()
  },
  render() {
    return h('div')
  },
})
</script>
