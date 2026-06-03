import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { ElMessage } from 'element-plus';
import { useI18n } from 'vue-i18n';
import { DEFAULTS } from '../../constants';

export function useCsvConvert() {
    const { t } = useI18n();
    const inputPath = ref('');
    const outputPath = ref('');
    const format = ref<'csv' | 'json' | 'sql'>(DEFAULTS.CONVERT_FORMAT as any);
    const message = ref('');

    async function run() {
        if (!inputPath.value || !outputPath.value) {
            ElMessage.warning(t('toolbox.pathRequired'));
            return;
        }

        try {
            await invoke('csv_convert_file', {
                params: {
                    input_path: inputPath.value,
                    output_path: outputPath.value,
                    format: format.value,
                },
            });
            message.value = t('toolbox.convertDone');
        } catch (error: any) {
            message.value = '';
            ElMessage.error(error);
        }
    }

    return {
        inputPath,
        outputPath,
        format,
        message,
        run,
    };
}
