<template>
    <div class="tool-panel">
        <div class="panel-body">
            <div class="form-group">
                <label class="form-label">{{ t('toolbox.jsonFilePlaceholder') }}</label>
                <el-input v-model="inputPath" :placeholder="t('toolbox.jsonFilePlaceholder')" />
            </div>
            <div class="form-row-split">
                <div class="form-group">
                    <label class="form-label">{{ t('toolbox.outputFilePlaceholder') }}</label>
                    <el-input v-model="outputPath" :placeholder="t('toolbox.outputFilePlaceholder')" />
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
import { useJsonConvert } from '../composables/tools/useJsonConvert';

const { t } = useI18n();
const { inputPath, outputPath, outputFormat, jsonPath, fields, message, run } = useJsonConvert();
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
    font-size: 13px;
    font-weight: 500;
}
</style>
