<template>
  <MacWindow
    ref="macWindowRef"
    :title="t('chat.title')"
    :isMinimized="isMinimized"
    @close="emit('close')"
    @minimize="emit('minimize')"
    :width="1000"
    :height="600"
  >
    <ChatView />
  </MacWindow>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import MacWindow from '@/components/common/MacWindow.vue'
import ChatView from './ChatView.vue'

const { t } = useI18n()

defineProps<{ isMinimized: boolean }>()
const emit = defineEmits<{
  (e: 'close'): void
  (e: 'minimize'): void
}>()

const macWindowRef = ref<InstanceType<typeof MacWindow> | null>(null)

defineExpose({
  bringToFront: () => macWindowRef.value?.bringToFront()
})
</script>
