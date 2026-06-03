<template>
    <MacWindow
        ref="macWindowRef"
        :title="t('knowledge.title')"
        :isMinimized="isMinimized"
        @close="handleClose"
        @minimize="emit('minimize')"
        width="1000"
        height="600">
        <div class="flex flex-col h-full min-h-0 bg-[var(--color-sidebar-bg)] font-sans">
            <div class="flex-1 flex flex-col overflow-hidden min-h-0 bg-[var(--color-input-bg)] p-5 items-center justify-center">
                <h2 class="text-2xl font-semibold mb-2">{{ t('knowledge.title') }}</h2>
                <p class="text-gray-500 mb-4">{{ t('knowledge.description') }}</p>
                <el-divider />
                <el-result
                    icon="info"
                    :title="t('knowledge.title')"
                    :sub-title="t('knowledge.description')">
                    <template #extra>
                        <el-button type="primary" @click="showMessage">
                            {{ t('app.help') }}
                        </el-button>
                    </template>
                </el-result>
            </div>
        </div>
    </MacWindow>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import { ElMessage } from 'element-plus';
import MacWindow from '@/components/common/MacWindow.vue';

const { t } = useI18n();

const props = defineProps<{ isMinimized: boolean }>();
const emit = defineEmits<{
    (e: 'close'): void;
    (e: 'minimize'): void;
}>();

const handleClose = () => {
    emit('close');
};

const showMessage = () => {
    ElMessage.info('知识库功能正在开发中...');
};

// MacWindow 组件引用
const macWindowRef = ref<InstanceType<typeof MacWindow> | null>(null);

// 暴露 bringToFront 方法
defineExpose({
    bringToFront: () => macWindowRef.value?.bringToFront()
});
</script>