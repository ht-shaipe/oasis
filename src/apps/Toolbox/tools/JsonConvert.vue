<template>
    <div class="tool-panel">
        <div class="panel-body">
            <div class="form-group">
                <label class="form-label">{{ t('toolbox.jsonFilePlaceholder') }}</label>
                <el-input
                    v-model="inputPath"
                    size="large"
                    class="w-full [&_.el-input__wrapper]:!rounded-r-none [&_.el-input__wrapper]:!rounded-l-[10px] [&_.el-input__wrapper]:!shadow-none [&_.el-input-group__append]:!p-0 [&_.el-input-group__append]:!overflow-hidden [&_.el-input-group__append]:!rounded-r-[10px]"
                    :placeholder="t('toolbox.jsonFilePlaceholder')">
                    <template #append>
                        <el-button
                            size="large"
                            class="w-[40px] min-w-[40px] px-0 rounded-none"
                            @click="pickInputPath"
                            :icon="Folder" />
                    </template>
                </el-input>
            </div>
            <div class="form-row-split">
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
                    <label class="form-label">{{ t('toolbox.outputFormat') }}</label>
                    <el-select v-model="outputFormat" style="width: 100%">
                        <el-option label="CSV" value="csv" />
                        <el-option label="Excel" value="excel" />
                    </el-select>
                </div>
            </div>
            <div class="form-row-split">
                <div class="form-group">
                    <label class="form-label">{{ t('toolbox.jsonPathPlaceholder') }}</label>
                    <el-input v-model="jsonPath" :placeholder="t('toolbox.jsonPathPlaceholder')" />
                </div>
                <div class="form-group">
                    <label class="form-label">{{ t('toolbox.fieldsPlaceholder') }}</label>
                    <el-input v-model="fields" :placeholder="t('toolbox.fieldsPlaceholder')" />
                </div>
            </div>
            <div class="form-group">
                <el-button type="primary" @click="run">{{ t('toolbox.singleConvert') }}</el-button>
            </div>
            <p v-if="message" class="result-msg">{{ message }}</p>
        </div>
    </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import { Folder, Document } from '@element-plus/icons-vue';
import { useJsonConvert } from '../composables/tools/useJsonConvert';
import { useFileDialog } from '@/composables/useFileDialog';

const { t } = useI18n();
const { inputPath, outputPath, outputFormat, jsonPath, fields, message, run } = useJsonConvert();
const { selectFile, selectFileSave } = useFileDialog();

async function pickInputPath() {
    const path = await selectFile({ extensions: ['json'] });
    if (path) inputPath.value = path;
}
async function pickOutputPath() {
    const path = await selectFileSave({ extensions: ['csv', 'xlsx'] });
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
    font-size: var(--app-font-12);
    font-weight: 500;
    color: var(--color-text-tertiary, #999);
    margin-bottom: 6px;
    letter-spacing: 0.3px;
}

.form-row-split {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
    margin-bottom: 16px;
}

.form-row-split .form-group {
    margin-bottom: 0;
}

.result-msg {
    color: #2d8c3c;
    margin-top: 12px;
    font-size: var(--app-font-13);
    font-weight: 500;
}
</style>
