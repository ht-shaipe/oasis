<template>
    <div class="tool-panel">
        <div class="panel-body">
            <div class="form-group">
                <label class="form-label">{{ t('toolbox.inputDirPlaceholder') }}</label>
                <el-input
                    v-model="inputDir"
                    size="large"
                    class="w-full [&_.el-input__wrapper]:!rounded-r-none [&_.el-input__wrapper]:!rounded-l-[10px] [&_.el-input__wrapper]:!shadow-none [&_.el-input-group__append]:!p-0 [&_.el-input-group__append]:!overflow-hidden [&_.el-input-group__append]:!rounded-r-[10px]"
                    :placeholder="t('toolbox.inputDirPlaceholder')">
                    <template #append>
                        <el-button
                            size="large"
                            class="w-[40px] min-w-[40px] px-0 rounded-none"
                            @click="pickInputDir"
                            :icon="FolderOpened" />
                    </template>
                </el-input>
            </div>
            <div class="form-group">
                <label class="form-label">{{ t('toolbox.outputFilePlaceholder') }}</label>
                <el-input
                    v-model="outputPath"
                    size="large"
                    class="w-full [&_.el-input__wrapper]:!rounded-r-none [&_.el-input__wrapper]:!rounded-l-[10px] [&_.el-input__wrapper]:!shadow-none [&_.el-input-group__append]:!p-0 [&_.el-input-group__append]:!overflow-hidden [&_.el-input-group__append]:!rounded-r-[10px]"
                    :placeholder="t('toolbox.outputFilePlaceholder')">
                    <template #append>
                        <el-button
                            size="large"
                            class="w-[40px] min-w-[40px] px-0 rounded-none"
                            @click="pickOutputPath"
                            :icon="Document" />
                    </template>
                </el-input>
            </div>
            <div class="form-group">
                <label class="form-label">{{ t('toolbox.jsonPathPlaceholder') }}</label>
                <el-input v-model="jsonPath" :placeholder="t('toolbox.jsonPathPlaceholder')" />
            </div>
            <div class="form-group">
                <el-button type="primary" @click="run">{{ t('toolbox.run') }}</el-button>
            </div>
            <p v-if="message" class="result-msg">{{ message }}</p>
        </div>
    </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import { FolderOpened, Document } from '@element-plus/icons-vue';
import { useJsonMerge } from '../composables/tools/useJsonMerge';
import { useFileDialog } from '@/composables/useFileDialog';

const { t } = useI18n();
const { inputDir, outputPath, jsonPath, message, run } = useJsonMerge();
const { selectFolder, selectFileSave } = useFileDialog();

async function pickInputDir() {
    const path = await selectFolder();
    if (path) inputDir.value = path;
}
async function pickOutputPath() {
    const path = await selectFileSave({ extensions: ['json'] });
    if (path) outputPath.value = path;
}
</script>

<style scoped>
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
    color: var(--color-text-tertiary, #999);
    margin-bottom: 6px;
    letter-spacing: 0.3px;
}

.result-msg {
    color: #2d8c3c;
    margin-top: 12px;
    font-size: 13px;
    font-weight: 500;
}
</style>
