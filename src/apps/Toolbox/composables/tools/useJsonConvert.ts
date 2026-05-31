import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { ElMessage } from 'element-plus';
import { useI18n } from 'vue-i18n';
import { DEFAULTS } from '../../constants';

export function useJsonConvert() {
    const { t } = useI18n();
    const inputPath = ref('');
    const outputPath = ref('');
    const outputFormat = ref<'csv' | 'excel'>(DEFAULTS.JSON_OUTPUT_FORMAT as any);
    const jsonPath = ref('');
    const fields = ref('');
    const message = ref('');

    async function run() {
        if (!inputPath.value || !outputPath.value) {
            ElMessage.warning(t('toolbox.pathRequired'));
            return;
        }

        try {
            await invoke('json_convert_file', {
                params: {
                    input_path: inputPath.value,
                    output_path: outputPath.value,
                    output_format: outputFormat.value,
                    json_path: jsonPath.value,
                    fields: fields.value.split(',').map(s => s.trim()).filter(Boolean),
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
        outputFormat,
        jsonPath,
        fields,
        message,
        run,
    };
}
