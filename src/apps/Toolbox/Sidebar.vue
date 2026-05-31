<template>
    <aside class="toolbox-sidebar">
        <div
            v-for="tool in tools"
            :key="tool.id"
            class="tool-nav-item"
            :class="{ active: activeTool === tool.id }"
            @click="setActiveTool(tool.id)">
            <img :src="tool.icon" class="tool-icon" :alt="t(tool.labelKey)" />
            <span class="tool-name">{{ t(tool.labelKey) }}</span>
        </div>
    </aside>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import type { Tool } from './types';

defineProps<{
    tools: Tool[];
    activeTool: string;
}>();

const emit = defineEmits<{
    'set-tool': [toolId: string];
}>();

const { t } = useI18n();

function setActiveTool(toolId: string) {
    emit('set-tool', toolId);
}
</script>

<style scoped>
.toolbox-sidebar {
    flex-shrink: 0;
    overflow-y: auto;
    overflow-x: hidden;
    background-color: var(--color-sidebar-bg, rgba(245, 245, 245, 0.95));
    padding: 12px 0;
}

.tool-nav-item {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 16px;
    margin: 0 6px 2px;
    border-radius: 6px;
    cursor: pointer;
    font-size: 13px;
    color: var(--color-text-secondary, #666);
    transition: all 0.15s ease;
}

.tool-nav-item:hover {
    background: var(--color-sidebar-item-hover, rgba(0, 0, 0, 0.05));
    color: var(--color-text-primary, #333);
}

.tool-nav-item.active {
    background: var(--color-sidebar-item-active, rgba(0, 122, 255, 0.1));
    color: var(--color-text-primary, #333);
}

.tool-icon {
    width: 18px;
    height: 18px;
    flex-shrink: 0;
    opacity: 0.7;
    transition: opacity 0.15s;
}

.tool-nav-item.active .tool-icon {
    opacity: 1;
}

.tool-nav-item:hover .tool-icon {
    opacity: 0.9;
}
</style>
