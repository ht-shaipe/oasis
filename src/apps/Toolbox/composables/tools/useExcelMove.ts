import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { ElMessage } from 'element-plus';
import { useI18n } from 'vue-i18n';
import { DEFAULTS } from '../../constants';
import type { ExcelMoveParams, ExcelPreviewResult } from '../../types';

// Make compatible with Tauri InvokeArgs
function toInvokeArgs(params: ExcelMoveParams): Record<string, unknown> {
    return {
        excelPath: params.excelPath,
        colHeader: params.colHeader,
        colIndex: params.colIndex,
        inputDir: params.inputDir,
        suffixes: params.suffixes,
        outputDir: params.outputDir,
    };
}

export function useExcelMove() {
    const { t } = useI18n();
    const excelPath = ref('');
    const colHeader = ref('');
    const inputDir = ref('');
    const outputDir = ref('');
    const suffixes = ref(DEFAULTS.EXCEL_SUFFIXES);
    const loading = ref(false);
    const preview = ref<ExcelPreviewResult | null>(null);
    const hasPlan = ref(false);

    function getParams(): ExcelMoveParams {
        return {
            excelPath: excelPath.value,
            colHeader: colHeader.value,
            colIndex: 0,
            inputDir: inputDir.value,
            suffixes: suffixes.value.split(',').map(s => s.trim()).filter(Boolean),
            outputDir: outputDir.value || inputDir.value,
        };
    }

    async function runPreview() {
        if (!excelPath.value || !inputDir.value) {
            ElMessage.warning(t('toolbox.pathRequired'));
            return;
        }

        loading.value = true;
        try {
            preview.value = await invoke<ExcelPreviewResult>('excel_move_preview', toInvokeArgs(getParams()));
            hasPlan.value = true;
        } catch (error: any) {
            ElMessage.error(error);
            preview.value = null;
            hasPlan.value = false;
        } finally {
            loading.value = false;
        }
    }

    async function apply() {
        if (!hasPlan.value) return;

        try {
            const msg = await invoke<string>('excel_move_apply', toInvokeArgs(getParams()));
            ElMessage.success(msg);
        } catch (error: any) {
            ElMessage.error(error);
        }
    }

    return {
        excelPath,
        colHeader,
        inputDir,
        outputDir,
        suffixes,
        loading,
        preview,
        hasPlan,
        runPreview,
        apply,
    };
}
