<template>
    <MacWindow ref="macWindowRef" :title="t('toolbox.title')" width="840" height="560" @close="emit('close')" @minimize="emit('minimize')">
        <div class="toolbox-container flex h-full">
            <!-- <div>sidebarSize: {{ sidebarSize }} {{ SIDEBAR_MIN }} - {{ SIDEBAR_MAX }}</div> -->
            <el-splitter class="flex-1">
                <el-splitter-panel v-model:size="sidebarSize" :min="SIDEBAR_MIN" :max="SIDEBAR_MAX">
                    <Sidebar :tools="tools" :active-tool="activeTool" @set-tool="setActiveTool" />
                </el-splitter-panel>
                <el-splitter-panel>
                    <main class="toolbox-main overflow-y-auto">
                        <Transition name="panel-fade" mode="out-in">
                            <CsvTool v-if="activeTool === 'csv-tool'" :key="'csv-tool'" />
                            <ExcelMove v-else-if="activeTool === 'excel-move'" :key="'excel-move'" />
                            <JsonConvert v-else-if="activeTool === 'json-convert'" :key="'json-convert'" />
                            <JsonMerge v-else-if="activeTool === 'json-merge'" :key="'json-merge'" />
                            <NetworkScan v-else-if="activeTool === 'network-scan'" :key="'network-scan'" />
                        </Transition>
                    </main>
                </el-splitter-panel>
            </el-splitter>
        </div>
    </MacWindow>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import MacWindow from '@/components/common/MacWindow.vue';
import Sidebar from './Sidebar.vue';
import { useToolbox } from './composables/useToolbox';
import { SIDEBAR_CONFIG } from './constants';
import CsvTool from './tools/CsvTool.vue';
import ExcelMove from './tools/ExcelMove.vue';
import JsonConvert from './tools/JsonConvert.vue';
import JsonMerge from './tools/JsonMerge.vue';
import NetworkScan from './tools/NetworkScan.vue';

const { t } = useI18n();
const emit = defineEmits<{
    close: [];
    minimize: [];
}>();

const SIDEBAR_MIN = SIDEBAR_CONFIG.MIN_WIDTH;
const SIDEBAR_MAX = SIDEBAR_CONFIG.MAX_WIDTH;

// 使用组合式函数
const { activeTool, tools, setActiveTool } = useToolbox();

// 侧边栏宽度 - 直接使用像素值
function loadSidebarWidth(): number {
    try {
        const saved = localStorage.getItem(SIDEBAR_CONFIG.STORAGE_KEY);
        let width = saved ? parseInt(saved, 10) : SIDEBAR_CONFIG.DEFAULT_WIDTH;
        // Ensure the width is within the valid range
        width = Math.max(SIDEBAR_MIN, Math.min(SIDEBAR_MAX, width));
        return width;
    } catch {
        return SIDEBAR_CONFIG.DEFAULT_WIDTH;
    }
}

// 侧边栏尺寸（像素值）
const sidebarSize = ref(loadSidebarWidth());

// 监听尺寸变化，保存到 localStorage
watch(sidebarSize, (newSize) => {
    try {
        localStorage.setItem(SIDEBAR_CONFIG.STORAGE_KEY, String(newSize));
    } catch (error) {
        console.warn('Failed to save sidebar width:', error);
    }
});

// MacWindow 组件引用
const macWindowRef = ref<InstanceType<typeof MacWindow> | null>(null);

// 暴露 bringToFront 方法
defineExpose({
    bringToFront: () => macWindowRef.value?.bringToFront()
});

defineOptions({ name: 'ToolboxApp' });
</script>

<style scoped>
.toolbox-container {
    color: var(--color-text-primary);
}

.toolbox-main {
    flex: 1;
    padding: 24px 28px;
    background: var(--color-bg);
}

/* ── Panel transitions ── */
.panel-fade-enter-active,
.panel-fade-leave-active {
    transition:
        opacity 0.15s ease,
        transform 0.15s ease;
}

.panel-fade-enter-from {
    opacity: 0;
    transform: translateY(6px);
}

.panel-fade-leave-to {
    opacity: 0;
    transform: translateY(-6px);
}
</style>
