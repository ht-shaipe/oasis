<template>
    <div
        class="flex min-h-14 items-center gap-2.5 border-b border-[var(--color-window-titlebar-border)] bg-[var(--color-sidebar-bg)] px-4 py-2.5">
        <div class="max-w-[620px] flex-1">
            <el-input
                v-model="searchQuery"
                class="credential-search"
                :placeholder="searchPlaceholder"
                :prefix-icon="Search"
                clearable
                size="small" />
        </div>
        <el-button class="credential-toolbar-primary" type="primary" size="small" @click="emit('add')">
            <el-icon><Plus /></el-icon>
            {{ addLabel }}
        </el-button>
        <el-button class="credential-toolbar-lock" text @click="emit('lock')">
            <el-icon><Lock /></el-icon>
            {{ lockLabel }}
        </el-button>
    </div>
</template>

<script setup lang="ts">
import { Lock, Plus, Search } from '@element-plus/icons-vue';

const searchQuery = defineModel<string>({ default: '' });

defineProps<{
    searchPlaceholder: string;
    addLabel: string;
    lockLabel: string;
}>();

const emit = defineEmits<{
    (e: 'add'): void;
    (e: 'lock'): void;
}>();
</script>

<style scoped>
.credential-search :deep(.el-input__wrapper) {
    height: 38px;
    border-radius: 999px;
    background-color: rgba(0, 0, 0, 0.04);
    box-shadow: none !important;
    border: 1px solid rgba(0, 0, 0, 0.04);
    padding: 0 14px;
    transition: all 0.2s ease;
}

.credential-search :deep(.el-input__wrapper.is-focus) {
    background-color: #fff;
    border-color: rgba(64, 158, 255, 0.35);
    box-shadow: 0 0 0 3px var(--el-color-primary-light-9) !important;
}

.credential-search :deep(.el-input__inner) {
    font-size: 14px;
}

.credential-search :deep(.el-input__prefix) {
    color: #9aa4b2;
    margin-right: 4px;
}

.credential-toolbar-primary {
    height: 38px;
    border-radius: 10px;
    padding: 0 16px;
    font-weight: 600;
    border: none;
    background: linear-gradient(180deg, #45a2ff 0%, #1977f3 100%);
    box-shadow: 0 6px 14px rgba(25, 119, 243, 0.22);
}

.credential-toolbar-primary:hover {
    transform: translateY(-1px);
    box-shadow: 0 8px 16px rgba(25, 119, 243, 0.26);
}

.credential-toolbar-primary:active {
    transform: translateY(0);
}

.credential-toolbar-lock {
    height: 38px;
    padding: 0 14px;
    border-radius: 10px;
    color: #5f6368;
    background-color: rgba(0, 0, 0, 0.03);
    border: 1px solid rgba(0, 0, 0, 0.05);
}

.credential-toolbar-lock:hover {
    color: #2f3a4a;
    background-color: rgba(0, 0, 0, 0.06);
}
</style>
