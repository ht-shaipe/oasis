<template>
    <div class="desktop-icons" :class="modeClass">
        <template v-for="app in sortedApps" :key="app.id">
            <div v-if="app.showOnDesktop" class="desktop-icon" :class="modeClass" @click="openApp(app.id)">
                <div class="icon-container" :class="modeClass">
                    <img :src="app.icon" :alt="t(app.nameKey || '')" />
                </div>
                <div class="icon-text" :class="modeClass">{{ t(app.nameKey || '') }}</div>
            </div>
        </template>
    </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import type { AppConfig } from '@/config/apps';

const { t } = useI18n();

const props = defineProps<{
    apps: AppConfig[];
    viewMode?: number;
    sortMode?: number;
}>();

const emit = defineEmits<{
    openApp: [app: string];
}>();

/** 根据 viewMode 计算 CSS class */
const modeClass = computed(() => {
    const m = props.viewMode ?? 0;
    if (m === 1) return 'view-large';
    if (m === 2) return 'view-list';
    return 'view-medium';
});

/** 根据 sortMode 排序桌面图标 */
const sortedApps = computed(() => {
    const apps = [...props.apps];
    if (props.sortMode === 1) {
        apps.sort((a, b) => {
            const nameA = t(a.nameKey || '');
            const nameB = t(b.nameKey || '');
            return nameA.localeCompare(nameB, 'zh');
        });
    }
    return apps;
});

const openApp = (app: string) => {
    emit('openApp', app);
};
</script>

<style scoped>
/* 桌面图标容器 */
.desktop-icons {
    display: flex;
    flex-wrap: wrap;
    gap: 20px;
    padding: 45px 0 0 20px;
    margin: 0;
}

/* 桌面图标项 */
.desktop-icon {
    display: flex;
    flex-direction: column;
    align-items: center;
    cursor: pointer;
    transition: all 0.2s;
    width: 80px;
    position: relative;
    margin: 0;
    padding: 0;
}

.desktop-icon:hover {
    transform: scale(1.05);
}

.desktop-icon:active {
    transform: scale(0.98);
}

/* 移除伪元素 - 只针对伪元素，不影响内容 */
.desktop-icon::before,
.desktop-icon::after,
.icon-container::before,
.icon-container::after,
.icon-text::before,
.icon-text::after {
    display: none !important;
    content: none !important;
}

/* 图标容器 */
.icon-container {
    width: 60px;
    height: 60px;
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: 5px;
    background: transparent;
}

.icon-container img {
    width: 100%;
    height: 100%;
    object-fit: contain;
    display: block;
    border: none;
    outline: none;
    box-shadow: none;
    background: transparent;
}

/* 图标文字 */
.icon-text {
    color: white;
    font-size: 12px;
    font-weight: 400;
    text-shadow: 0 1px 3px rgba(0, 0, 0, 0.5);
    text-align: center;
    margin: 2px 0 0 0;
    line-height: 1.2;
    display: block;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 100%;
    padding: 0;
    border: none;
    background: transparent;
    text-decoration: none;
    position: relative;
}

/* 移除首字母装饰 */
.icon-text::first-letter {
    margin-left: 0;
    padding-left: 0;
    border-left: none;
}

/* 交互状态 */
.icon-text:hover,
.icon-text:focus {
    outline: none;
    border: none;
    text-decoration: none;
    background: transparent;
}

/* 滚动条隐藏 */
::-webkit-scrollbar {
    display: none;
}

/* ---- 视图模式 ---- */

/* 大图标模式 */
.desktop-icons.view-large {
    gap: 28px;
    padding: 40px 0 0 20px;
}

.desktop-icon.view-large {
    width: 100px;
}
.desktop-icon.view-large .icon-container {
    width: 80px;
    height: 80px;
    margin-bottom: 8px;
}
.desktop-icon.view-large .icon-text {
    font-size: 14px;
}

/* 列表模式 */
.desktop-icons.view-list {
    flex-direction: column;
    gap: 8px;
    padding: 40px 0 0 16px;
    align-items: flex-start;
}

.desktop-icon.view-list {
    flex-direction: row;
    align-items: center;
    width: auto;
    gap: 10px;
}
.desktop-icon.view-list .icon-container {
    width: 28px;
    height: 28px;
    margin-bottom: 0;
}
.desktop-icon.view-list .icon-text {
    font-size: 13px;
    text-align: left;
    white-space: nowrap;
    overflow: visible;
}
</style>
