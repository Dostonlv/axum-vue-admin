<template>
  <AppPage>
    <h1>{{ t('views.top_menu.title') }}</h1>
  </AppPage>
</template>

<script setup>
import { useI18n } from 'vue-i18n'

const { t } = useI18n({ useScope: 'global' })
</script>
