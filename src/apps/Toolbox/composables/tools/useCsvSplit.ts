import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { ElMessage } from 'element-plus';
import { useI18n } from 'vue-i18n';
import { DEFAULTS } from '../../constants';

export function useCsvSplit() {
    const { t } = useI18n();
    const inputPath = ref('');
    const outputDir = ref('');
    const parts = ref(DEFAULTS.CSV_SPLIT_PARTS);
    const message = ref('');

    async function run() {
        if (!inputPath.value || !outputDir.value) {
            ElMessage.warning(t('toolbox.pathRequired'));
            return;
        }

        try {
            await invoke('csv_split_file', {
                inputPath: inputPath.value,
                outputDir: outputDir.value,
                parts: parts.value,
            });
            message.value = t('toolbox.splitDone');
        } catch (error: any) {
            message.value = '';
            ElMessage.error(error);
        }
    }

    return {
        inputPath,
        outputDir,
        parts,
        message,
        run,
    };
}
