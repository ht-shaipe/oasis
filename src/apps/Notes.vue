<template>
    <MacWindow
        ref="macWindowRef"
        :title="t('notes.helpTitle')"
        :isMinimized="isMinimized"
        @close="closeApp"
        @minimize="toggleMinimize"
        width="800"
        height="600"
    >
        <div class="flex h-full bg-[#f5f5f5] rounded-b-2.5">
            <div class="w-52.5 bg-[var(--color-sidebar-bg)] border-r border-[var(--color-window-titlebar-border)] flex flex-col">
                <div class="p-3.75 flex justify-between items-center border-b border-[var(--color-window-titlebar-border)]">
                    <h3 class="m-0 text-[var(--color-text-primary)]">{{ t('notes.title') }}</h3>
                    <button class="bg-none border-0 text-[var(--app-font-18)] text-[#007aff] cursor-pointer" :title="t('notes.newNote')">
                        <i class="el-icon-plus"></i>
                    </button>
                </div>
                <div class="flex-1 overflow-y-auto">
                    <div class="p-3.75 border-b border-[var(--color-window-titlebar-border)] cursor-pointer hover:bg-[var(--color-input-bg)] bg-[#e2e2e2]">
                        <div class="font-bold text-[var(--color-text-primary)] mb-1.25">{{ t('notes.helpTitle') }}</div>
                        <div class="text-[var(--color-text-secondary)] text-[var(--app-font-12)] whitespace-nowrap overflow-hidden text-ellipsis">{{ t('notes.helpPreview') }}</div>
                    </div>
                </div>
            </div>
            <div class="flex-1 flex flex-col overflow-hidden">
                <div class="flex-1 p-5 bg-white overflow-y-auto leading-relaxed">
                    <h1 class="text-[var(--color-text-primary)] text-[var(--app-font-24)] mt-0 mb-5">{{ t('notes.mainTitle') }}</h1>

                    <h2 class="text-[#444] text-[var(--app-font-20)] mt-6.25 mb-3.75 border-b border-[#eee] pb-1.25">{{ t('notes.introduction') }}</h2>
                    <p class="mb-3.75">{{ t('notes.introDesc') }}</p>

                    <h2 class="text-[#444] text-[var(--app-font-20)] mt-6.25 mb-3.75 border-b border-[#eee] pb-1.25">{{ t('help.basicUsage') }}</h2>
                    <ol class="mb-3.75 pl-6.25">
                        <li class="mb-2"><strong>{{ t('notes.step0') }}</strong></li>
                        <li class="mb-2"><strong>{{ t('notes.openCodeGenerator') }}</strong></li>
                        <li class="mb-2"><strong>{{ t('notes.describeNeeds') }}</strong>{{ t('notes.step2Desc') }}</li>
                        <li class="mb-2"><strong>{{ t('notes.selectConfig') }}</strong></li>
                        <li class="mb-2"><strong>{{ t('notes.step4Desc') }}</strong></li>
                        <li class="mb-2"><strong>{{ t('notes.step5Desc') }}</strong></li>
                    </ol>

                    <h2 class="text-[#444] text-[var(--app-font-20)] mt-6.25 mb-3.75 border-b border-[#eee] pb-1.25">{{ t('help.features') }}</h2>
                    <h3 class="text-[#555] text-[var(--app-font-16)] mt-5 mb-2.5">{{ t('notes.codeGeneratorTitle') }}</h3>
                    <p class="mb-3.75">{{ t('notes.codeGeneratorDesc') }}</p>
                    <ul class="mb-3.75 pl-6.25">
                        <li class="mb-2"><strong>{{ t('notes.aiModel') }}</strong></li>
                        <li class="mb-2"><strong>{{ t('notes.uiFramework') }}</strong></li>
                        <li class="mb-2"><strong>{{ t('notes.jsFramework') }}</strong></li>
                        <li class="mb-2"><strong>{{ t('notes.cdnSource') }}</strong></li>
                        <li class="mb-2"><strong>{{ t('notes.deviceAdapt') }}</strong></li>
                    </ul>

                    <h3 class="text-[#555] text-[var(--app-font-16)] mt-5 mb-2.5">{{ t('notes.editorTitle') }}</h3>
                    <p class="mb-3.75">{{ t('notes.editorDesc') }}</p>
                    <ul class="mb-3.75 pl-6.25">
                        <li class="mb-2">{{ t('codeEditor.syntaxHighlight') }}</li>
                        <li class="mb-2">{{ t('generator.codeFormatting') }}</li>
                        <li class="mb-2">{{ t('generator.autoSave') }}</li>
                    </ul>

                    <h3 class="text-[#555] text-[var(--app-font-16)] mt-5 mb-2.5">{{ t('notes.previewTitle') }}</h3>
                    <p class="mb-3.75">{{ t('notes.previewDesc') }}</p>

                    <h3 class="text-[#555] text-[var(--app-font-16)] mt-5 mb-2.5">{{ t('notes.versionTitle') }}</h3>
                    <p class="mb-3.75">{{ t('notes.versionDesc') }}</p>

                    <h2 class="text-[#444] text-[var(--app-font-20)] mt-6.25 mb-3.75 border-b border-[#eee] pb-1.25">{{ t('help.tips') }}</h2>
                    <ul class="mb-3.75 pl-6.25">
                        <li class="mb-2"><strong>{{ t('notes.detailDesc') }}</strong></li>
                        <li class="mb-2"><strong>{{ t('notes.specifyStack') }}</strong></li>
                        <li class="mb-2"><strong>{{ t('notes.continueConv') }}</strong></li>
                        <li class="mb-2"><strong>{{ t('notes.saveVersion') }}</strong></li>
                    </ul>

                    <h2 class="text-[#444] text-[var(--app-font-20)] mt-6.25 mb-3.75 border-b border-[#eee] pb-1.25">{{ t('help.faq') }}</h2>
                    <p class="mb-3.75"><strong>{{ t('notes.faq3FullQ') }}</strong></p>
                    <p class="mb-3.75">{{ t('notes.faq3FullA') }}</p>

                    <p class="mb-3.75"><strong>{{ t('notes.faq1FullQ') }}</strong></p>
                    <p class="mb-3.75">{{ t('notes.faq1FullA') }}</p>

                    <p class="mb-3.75"><strong>{{ t('notes.faq2FullQ') }}</strong></p>
                    <p class="mb-3.75">{{ t('notes.faq2FullA') }}</p>
                </div>
            </div>
        </div>
    </MacWindow>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useI18n } from 'vue-i18n';
import MacWindow from '@/components/common/MacWindow.vue';

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

// MacWindow 组件引用
const macWindowRef = ref<InstanceType<typeof MacWindow> | null>(null);

// 暴露 bringToFront 方法
defineExpose({
    bringToFront: () => macWindowRef.value?.bringToFront()
});
</script>

<style scoped>
/* Notes组件样式已全部转换为UnoCSS工具类 */
</style>
