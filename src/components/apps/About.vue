<template>
    <MacWindow 
        :title="t('about.title')" 
        :isMinimized="isMinimized" 
        @close="closeApp" 
        @minimize="toggleMinimize"
        width="300"
    >
        <div class="about-content">
            <div class="app-logo">
                <img src="/assets/icons/WebAI.png" alt="WebAI" width="80">
            </div>
            <h2>WebAI</h2>
            <p>{{ t('about.version') }} 1.0.0</p>
            <p>{{ t('about.description') }}</p>
            <p>{{ t('about.features') }}</p>
        </div>
        <div class="copyright-info">
            <p>© 2026 <a href="https://htui.tech/" target="_blank">HongTui</a></p>
        </div>
    </MacWindow>
</template>

<script setup lang="ts">
import MacWindow from '@/components/common/MacWindow.vue';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();

// 定义属性
const props = defineProps({
    isMinimized: {
        type: Boolean,
        default: false
    }
});

// 事件发射
const emit = defineEmits(['close', 'minimize']);

// 关闭应用
const closeApp = () => {
    emit('close');
};

// 切换最小化状态
const toggleMinimize = () => {
    emit('minimize');
};
</script>

<style scoped>
/* 关于窗口 */
.about-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 30px 20px;
}

.app-logo {
    margin-bottom: 15px;
}

.app-logo img {
    width: 80px;
    height: 80px;
    object-fit: contain;
}

.about-content h2 {
    margin: 10px 0;
    font-size: 18px;
    color: var(--color-text-primary);
}

.about-content p {
    margin: 5px 0;
    font-size: 14px;
    color: var(--color-text-secondary);
    text-align: center;
}

.copyright-info {
    position: absolute;
    bottom: 10px;
    right: 10px;
    font-size: 12px;
    color: var(--color-text-tertiary);
    text-align: right;
}

.copyright-info a {
    color: var(--color-link);
    text-decoration: none;
}

.copyright-info a:hover {
    color: var(--color-link-hover);
}
</style> 