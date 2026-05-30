<template>
    <div class="desktop-icons">
        <template v-for="app in apps" :key="app.id">
            <div v-if="app.showOnDesktop" class="desktop-icon" @click="openApp(app.id)">
                <div class="icon-container">
                    <img :src="app.icon" :alt="t(app.nameKey || '')" />
                </div>
                <div class="icon-text">{{ t(app.nameKey || '') }}</div>
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
</style>
