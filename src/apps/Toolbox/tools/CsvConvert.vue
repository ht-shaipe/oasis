<template>
    <div class="tool-panel">
        <div class="panel-body">
            <div class="form-group">
                <label class="form-label">{{ t('toolbox.inputFilePlaceholder') }}</label>
                <el-input v-model="inputPath" :placeholder="t('toolbox.inputFilePlaceholder')" />
            </div>
            <div class="form-group">
                <label class="form-label">{{ t('toolbox.outputFilePlaceholder') }}</label>
                <el-input v-model="outputPath" :placeholder="t('toolbox.outputFilePlaceholder')" />
            </div>
            <div class="form-group form-row-inline">
                <label class="form-label">{{ t('toolbox.outputFormat') }}</label>
                <el-select v-model="format" :placeholder="t('toolbox.outputFormat')" style="width: 160px">
                    <el-option label="CSV" value="csv" />
                    <el-option label="JSON" value="json" />
                    <el-option label="SQL" value="sql" />
                </el-select>
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
import { useCsvConvert } from '../composables/tools/useCsvConvert';

const { t } = useI18n();
const { inputPath, outputPath, format, message, run } = useCsvConvert();
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

.form-row-inline {
    display: flex;
    align-items: center;
    gap: 12px;
}

.form-row-inline .form-label {
    margin-bottom: 0;
    white-space: nowrap;
}

.result-msg {
    color: #2d8c3c;
    margin-top: 12px;
    font-size: 13px;
    font-weight: 500;
}
</style>
