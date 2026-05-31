<template>
    <div class="mac-dock">
        <template v-for="app in apps" :key="app.id">
            <div v-if="app.showInDock" class="dock-item" @click="openApp(app.id)">
                <img :src="app.icon" :alt="t(app.nameKey || '')" :title="t(app.nameKey || '')" />
            </div>
        </template>
    </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import type { AppConfig } from '@/config/apps';

const { t } = useI18n();

// Props 定义
defineProps<{
    apps: AppConfig[];
}>();

// 事件发射
const emit = defineEmits(['openApp']);

// 打开App
const openApp = (app: string) => {
    emit('openApp', app);
};
</script>


<style scoped>
/* Mac Dock栏 */
.mac-dock {
    position: absolute;
    bottom: 20px;
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    gap: 15px;
    padding: 10px 20px;
    background-color: var(--color-dock-bg);
    border-radius: 16px;
    backdrop-filter: blur(10px);
    box-shadow: 0 8px 16px var(--color-dock-shadow);
    z-index: 1000;
}

.dock-item {
    width: 60px;
    height: 60px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all 0.3s;
}

.dock-item img {
    width: 100%;
    height: 100%;
    object-fit: contain;
}

.dock-item:hover {
    transform: translateY(-8px) scale(1.1);
}
</style>
