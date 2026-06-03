<template>
  <div class="conversation-sidebar">
    <div class="sidebar-header">
      <h3 class="sidebar-title">对话</h3>
      <button class="new-chat-btn" @click="$emit('new')" title="新建对话">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="12" y1="5" x2="12" y2="19"/>
          <line x1="5" y1="12" x2="19" y2="12"/>
        </svg>
      </button>
    </div>
    <div class="conversation-list">
      <div
        v-for="conv in conversations"
        :key="conv.id"
        class="conv-item"
        :class="{ active: conv.id === activeId }"
        @click="$emit('select', conv.id)"
      >
        <div class="conv-icon">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/>
          </svg>
        </div>
        <div class="conv-content">
          <div class="conv-title">{{ conv.title }}</div>
          <div class="conv-meta">
            {{ conv.messages.length }} 条消息 · {{ formatDate(conv.updatedAt) }}
          </div>
        </div>
        <button
          class="conv-delete"
          @click.stop="$emit('delete', conv.id)"
          title="删除对话"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="3 6 5 6 21 6"/>
            <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
          </svg>
        </button>
      </div>
      <div v-if="conversations.length === 0" class="conv-empty">
        暂无对话，点击 + 开始
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Conversation } from '@/store/chat'

defineProps<{
  conversations: Conversation[]
  activeId: string | null
}>()

defineEmits<{
  (e: 'select', id: string): void
  (e: 'delete', id: string): void
  (e: 'new'): void
}>()

function formatDate(ts: number): string {
  const d = new Date(ts)
  const now = new Date()
  if (d.toDateString() === now.toDateString()) {
    return d.toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' })
  }
  return d.toLocaleDateString('zh-CN', { month: 'short', day: 'numeric' })
}
</script>

<style scoped>
.conversation-sidebar {
  width: 240px;
  flex-shrink: 0;
  border-right: 1px solid var(--color-sidebar-border);
  background: var(--color-sidebar-bg);
  display: flex;
  flex-direction: column;
  height: 100%;
}

.sidebar-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 16px 10px;
}

.sidebar-title {
  font-size: var(--app-font-13);
  font-weight: 600;
  color: var(--color-text-primary);
  margin: 0;
}

.new-chat-btn {
  width: 30px;
  height: 30px;
  border-radius: 6px;
  border: none;
  background: transparent;
  color: var(--color-text-secondary);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s;
}

.new-chat-btn:hover {
  background: var(--color-sidebar-item-hover);
  color: var(--color-text-primary);
}

.conversation-list {
  flex: 1;
  overflow-y: auto;
  padding: 0 8px 8px;
}

.conv-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 10px;
  border-radius: 8px;
  cursor: pointer;
  transition: background-color 0.15s;
  margin-bottom: 2px;
}

.conv-item:hover {
  background: var(--color-sidebar-item-hover);
}

.conv-item.active {
  background: var(--color-sidebar-item-active);
}

.conv-icon {
  flex-shrink: 0;
  color: var(--color-text-tertiary);
  display: flex;
}

.conv-content {
  flex: 1;
  min-width: 0;
}

.conv-title {
  font-size: var(--app-font-13);
  color: var(--color-text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.conv-meta {
  font-size: var(--app-font-11);
  color: var(--color-text-tertiary);
  margin-top: 2px;
}

.conv-delete {
  flex-shrink: 0;
  opacity: 0;
  width: 24px;
  height: 24px;
  border-radius: 4px;
  border: none;
  background: transparent;
  color: var(--color-text-tertiary);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s;
}

.conv-item:hover .conv-delete {
  opacity: 1;
}

.conv-delete:hover {
  background: rgba(245, 108, 108, 0.15);
  color: var(--color-danger);
}

.conv-empty {
  padding: 20px 12px;
  text-align: center;
  font-size: var(--app-font-12);
  color: var(--color-text-tertiary);
}
</style>
