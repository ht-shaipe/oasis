<template>
    <div
        class="flex min-h-14 items-center gap-2.5 border-b border-[var(--color-window-titlebar-border)] bg-[var(--color-sidebar-bg)] px-4 py-2.5">
        <div class="max-w-[750px] flex-1">
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
        <el-dropdown trigger="click" @command="handleCommand">
            <el-button class="credential-toolbar-more" size="small">
                <el-icon><MoreFilled /></el-icon>
                {{ moreLabel }}
            </el-button>
            <template #dropdown>
                <el-dropdown-menu>
                    <el-dropdown-item command="import-browser">
                        <svg
                            class="credential-dropdown-icon"
                            xmlns="http://www.w3.org/2000/svg"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="2"
                            stroke-linecap="round"
                            stroke-linejoin="round">
                            <circle cx="12" cy="12" r="10" />
                            <line x1="2" y1="12" x2="22" y2="12" />
                            <path
                                d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z" />
                        </svg>
                        从浏览器导入
                    </el-dropdown-item>
                    <el-dropdown-item command="merge">
                        <svg
                            class="credential-dropdown-icon"
                            xmlns="http://www.w3.org/2000/svg"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="2"
                            stroke-linecap="round"
                            stroke-linejoin="round">
                            <path d="M16 16l4-4-4-4" />
                            <path d="M20 12H9" />
                            <path d="M4 20V4" />
                        </svg>
                        {{ mergeLabel }}
                    </el-dropdown-item>
                    <el-dropdown-item command="lock">
                        <el-icon><Lock /></el-icon>
                        {{ lockLabel }}
                    </el-dropdown-item>
                </el-dropdown-menu>
            </template>
        </el-dropdown>
    </div>
</template>

<script setup lang="ts">
import { Lock, MoreFilled, Plus, Search } from '@element-plus/icons-vue';

const searchQuery = defineModel<string>({ default: '' });

defineProps<{
    searchPlaceholder: string;
    addLabel: string;
    lockLabel: string;
    mergeLabel: string;
    moreLabel?: string;
}>();

const emit = defineEmits<{
    (e: 'add'): void;
    (e: 'lock'): void;
    (e: 'import-browser'): void;
    (e: 'merge'): void;
}>();

const handleCommand = (command: string) => {
    if (command === 'import-browser') emit('import-browser');
    else if (command === 'merge') emit('merge');
    else if (command === 'lock') emit('lock');
};
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
    font-size: var(--app-font-14);
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

.credential-toolbar-more {
    height: 38px;
    padding: 0 14px;
    border-radius: 10px;
    color: #374151;
    background-color: rgba(0, 0, 0, 0.03);
    border: 1px solid rgba(0, 0, 0, 0.08);
    font-weight: 500;
}

.credential-toolbar-more:hover {
    color: #1977f3;
    background-color: rgba(25, 119, 243, 0.06);
    border-color: rgba(25, 119, 243, 0.2);
}

.credential-dropdown-icon {
    width: 16px;
    height: 16px;
    margin-right: 8px;
    vertical-align: -3px;
}
</style>
