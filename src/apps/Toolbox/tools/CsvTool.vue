<template>
    <div class="tool-panel">
        <!-- 子标签栏 -->
        <div class="csv-tabs">
            <button
                v-for="tab in tabs"
                :key="tab.key"
                class="csv-tab"
                :class="{ active: activeTab === tab.key }"
                @click="activeTab = tab.key">
                {{ t(tab.labelKey) }}
            </button>
        </div>

        <div class="panel-body">
            <!-- ========== 目录统计 ========== -->
            <template v-if="activeTab === 'stats'">
                <div class="form-group">
                    <label class="form-label">{{ t('toolbox.dirPlaceholder') }}</label>
                    <el-input
                        v-model="dir"
                        size="large"
                        class="w-full [&_.el-input__wrapper]:!rounded-r-none [&_.el-input__wrapper]:!rounded-l-[10px] [&_.el-input__wrapper]:!shadow-none [&_.el-input-group__append]:!p-0 [&_.el-input-group__append]:!overflow-hidden [&_.el-input-group__append]:!rounded-r-[10px]"
                        :placeholder="t('toolbox.dirPlaceholder')">
                        <template #append>
                            <el-button
                                size="large"
                                class="w-[40px] min-w-[40px] px-0 rounded-none"
                                @click="pickFolder('dir')"
                                :icon="FolderOpened" />
                        </template>
                    </el-input>
                </div>
                <el-button type="primary" @click="runStats" :loading="loadingStats">
                    {{ t('toolbox.run') }}
                </el-button>
                <div v-if="resultStats" class="result-area">
                    <div class="result-summary">
                        <span class="result-badge">
                            {{ t('toolbox.totalLines') }}: <strong>{{ resultStats.total }}</strong>
                        </span>
                    </div>
                    <el-table :data="resultStats.entries" max-height="300" size="small" stripe>
                        <el-table-column prop="path" :label="t('toolbox.filePath')" show-overflow-tooltip />
                        <el-table-column prop="lines" :label="t('toolbox.lines')" width="100" align="center" />
                    </el-table>
                </div>
            </template>

            <!-- ========== 文件拆分 ========== -->
            <template v-if="activeTab === 'split'">
                <div class="form-group">
                    <label class="form-label">{{ t('toolbox.filePlaceholder') }}</label>
                    <el-input
                        v-model="inputPath"
                        size="large"
                        class="w-full [&_.el-input__wrapper]:!rounded-r-none [&_.el-input__wrapper]:!rounded-l-[10px] [&_.el-input__wrapper]:!shadow-none [&_.el-input-group__append]:!p-0 [&_.el-input-group__append]:!overflow-hidden [&_.el-input-group__append]:!rounded-r-[10px]"
                        :placeholder="t('toolbox.filePlaceholder')">
                        <template #append>
                            <el-button
                                size="large"
                                class="w-[40px] min-w-[40px] px-0 rounded-none"
                                @click="pickFile('inputPath')"
                                :icon="Folder" />
                        </template>
                    </el-input>
                </div>
                <div class="form-group">
                    <label class="form-label">{{ t('toolbox.outputDirPlaceholder') }}</label>
                    <el-input
                        v-model="outputDir"
                        size="large"
                        class="w-full [&_.el-input__wrapper]:!rounded-r-none [&_.el-input__wrapper]:!rounded-l-[10px] [&_.el-input__wrapper]:!shadow-none [&_.el-input-group__append]:!p-0 [&_.el-input-group__append]:!overflow-hidden [&_.el-input-group__append]:!rounded-r-[10px]"
                        :placeholder="t('toolbox.outputDirPlaceholder')">
                        <template #append>
                            <el-button
                                size="large"
                                class="w-[40px] min-w-[40px] px-0 rounded-none"
                                @click="pickFolder('outputDir')"
                                :icon="FolderOpened" />
                        </template>
                    </el-input>
                </div>
                <div class="form-group form-row-inline">
                    <label class="form-label">{{ t('toolbox.splitParts') }}</label>
                    <el-input-number v-model="parts" :min="2" :max="100" size="default" />
                </div>
                <el-button type="primary" @click="runSplit">{{ t('toolbox.run') }}</el-button>
                <p v-if="messageSplit" class="result-msg">{{ messageSplit }}</p>
            </template>

            <!-- ========== 格式转换 ========== -->
            <template v-if="activeTab === 'convert'">
                <div class="form-group">
                    <label class="form-label">{{ t('toolbox.inputFilePlaceholder') }}</label>
                    <el-input
                        v-model="inputPathCvt"
                        size="large"
                        class="w-full [&_.el-input__wrapper]:!rounded-r-none [&_.el-input__wrapper]:!rounded-l-[10px] [&_.el-input__wrapper]:!shadow-none [&_.el-input-group__append]:!p-0 [&_.el-input-group__append]:!overflow-hidden [&_.el-input-group__append]:!rounded-r-[10px]"
                        :placeholder="t('toolbox.inputFilePlaceholder')">
                        <template #append>
                            <el-button
                                size="large"
                                class="w-[40px] min-w-[40px] px-0 rounded-none"
                                @click="pickFile('inputPathCvt')"
                                :icon="Folder" />
                        </template>
                    </el-input>
                </div>
                <div class="form-group">
                    <label class="form-label">{{ t('toolbox.outputFilePlaceholder') }}</label>
                    <el-input
                        v-model="outputPathCvt"
                        size="large"
                        class="w-full [&_.el-input__wrapper]:!rounded-r-none [&_.el-input__wrapper]:!rounded-l-[10px] [&_.el-input__wrapper]:!shadow-none [&_.el-input-group__append]:!p-0 [&_.el-input-group__append]:!overflow-hidden [&_.el-input-group__append]:!rounded-r-[10px]"
                        :placeholder="t('toolbox.outputFilePlaceholder')">
                        <template #append>
                            <el-button
                                size="large"
                                class="w-[40px] min-w-[40px] px-0 rounded-none"
                                @click="pickFileSave()"
                                :icon="Document" />
                        </template>
                    </el-input>
                </div>
                <div class="form-group form-row-inline">
                    <label class="form-label">{{ t('toolbox.outputFormat') }}</label>
                    <el-select v-model="format" :placeholder="t('toolbox.outputFormat')" style="width: 160px">
                        <el-option label="CSV" value="csv" />
                        <el-option label="JSON" value="json" />
                        <el-option label="SQL" value="sql" />
                    </el-select>
                </div>
                <el-button type="primary" @click="runConvert">{{ t('toolbox.run') }}</el-button>
                <p v-if="messageCvt" class="result-msg">{{ messageCvt }}</p>
            </template>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { Folder, FolderOpened, Document } from '@element-plus/icons-vue';
import { useCsvStats } from '../composables/tools/useCsvStats';
import { useCsvSplit } from '../composables/tools/useCsvSplit';
import { useCsvConvert } from '../composables/tools/useCsvConvert';
import { useFileDialog } from '@/composables/useFileDialog';

const { t } = useI18n();
const { selectFile, selectFolder, selectFileSave } = useFileDialog();

// ── Tab state ──
const tabs = [
    { key: 'stats', labelKey: 'toolbox.csvStats' },
    { key: 'split', labelKey: 'toolbox.csvSplit' },
    { key: 'convert', labelKey: 'toolbox.formatConvert' },
];
const activeTab = ref('stats');

// ── Composables ──
const { dir, loading: loadingStats, result: resultStats, run: runStats } = useCsvStats();
const { inputPath, outputDir, parts, message: messageSplit, run: runSplit } = useCsvSplit();
const {
    inputPath: inputPathCvt,
    outputPath: outputPathCvt,
    format,
    message: messageCvt,
    run: runConvert,
} = useCsvConvert();

// ── File pickers ──
async function pickFile(target: 'inputPath' | 'inputPathCvt') {
    const path = await selectFile({ extensions: ['csv'] });
    if (path) {
        if (target === 'inputPath') inputPath.value = path;
        else inputPathCvt.value = path;
    }
}

async function pickFolder(target: 'dir' | 'outputDir') {
    const path = await selectFolder();
    if (path) {
        if (target === 'dir') dir.value = path;
        else outputDir.value = path;
    }
}

async function pickFileSave() {
    const path = await selectFileSave({ extensions: ['csv', 'json', 'sql'] });
    if (path) outputPathCvt.value = path;
}
</script>

<style scoped>
.csv-tabs {
    display: flex;
    gap: 0;
    border-bottom: 2px solid var(--color-card-border);
    margin-bottom: 20px;
}

.csv-tab {
    padding: 8px 18px;
    border: none;
    background: none;
    font-size: 13px;
    font-weight: 500;
    color: var(--color-text-tertiary);
    cursor: pointer;
    border-bottom: 2px solid transparent;
    margin-bottom: -2px;
    transition:
        color 0.15s,
        border-color 0.15s;
}

.csv-tab:hover {
    color: var(--color-text-primary);
}

.csv-tab.active {
    color: var(--color-menu-hover, #0078d7);
    border-bottom-color: var(--color-menu-hover, #0078d7);
}

.panel-body {
    display: flex;
    flex-direction: column;
    gap: 0;
}

.form-group {
    margin-bottom: 16px;
}

.form-label {
    display: block;
    font-size: 12px;
    font-weight: 500;
    color: var(--color-text-tertiary);
    margin-bottom: 6px;
    letter-spacing: 0.3px;
}

.form-row-inline {
    display: flex;
    align-items: center;
    gap: 12px;
}

.form-row-inline .form-label {
    margin-bottom: 0;
    white-space: nowrap;
}

.result-area {
    margin-top: 16px;
    padding: 16px;
    background: var(--color-card-bg);
    border-radius: 8px;
    border: 1px solid var(--color-card-border);
}

.result-summary {
    display: flex;
    gap: 16px;
    margin-bottom: 12px;
    flex-wrap: wrap;
}

.result-badge {
    font-size: 13px;
    color: var(--color-text-primary);
}

.result-badge strong {
    color: var(--color-menu-hover, #0078d7);
}

.result-msg {
    color: #2d8c3c;
    margin-top: 12px;
    font-size: 13px;
    font-weight: 500;
}
</style>
