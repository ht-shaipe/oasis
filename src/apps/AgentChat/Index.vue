<template>
  <MacWindow
    ref="macWindowRef"
    title="AI Agent"
    :is-minimized="isMinimized"
    @close="emit('close')"
    @minimize="emit('minimize')"
    :width="1000"
    :height="660"
  >
    <AgentChatView />
  </MacWindow>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import MacWindow from '@/components/common/MacWindow.vue'
import AgentChatView from './AgentChatView.vue'

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
