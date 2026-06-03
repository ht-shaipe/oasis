<template>
    <el-config-provider :locale="elLocale">
        <RouterView />
    </el-config-provider>
</template>

<script setup lang="ts">
import { computed, watch } from 'vue';
import zhCn from 'element-plus/es/locale/lang/zh-cn';
import en from 'element-plus/es/locale/lang/en';
import { useLocaleStore } from '@/store/locale';
import { useFontSizeStore } from '@/store/fontSize';

const localeStore = useLocaleStore();
const elLocale = computed(() => (localeStore.locale === 'zh-CN' ? zhCn : en));

// 全局字体大小
const fontSizeStore = useFontSizeStore();
watch(() => fontSizeStore.currentSize, (val) => {
    document.documentElement.style.setProperty('--el-font-size-base', `${val}px`);
    document.documentElement.style.fontSize = `${val}px`;
}, { immediate: true });
</script>

<style>
html,
body,
#app {
    width: 100%;
    height: 100%;
    margin: 0;
    padding: 0;
    background: transparent;
    overflow: hidden;
}

body {
    box-sizing: border-box;
    transition:
        background-color 0.3s ease,
        color 0.3s ease;
}
</style>
