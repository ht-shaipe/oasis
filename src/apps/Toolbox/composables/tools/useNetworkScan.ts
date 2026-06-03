import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { ElMessage } from 'element-plus';
import { useI18n } from 'vue-i18n';
import { DEFAULTS } from '../../constants';
import type { NetworkScanResult } from '../../types';

export function useNetworkScan() {
    // I18n instance available for future use
    useI18n();
    const ipRange = ref(DEFAULTS.SCAN_IP_RANGE);
    const ports = ref(DEFAULTS.SCAN_PORTS);
    const timeout = ref(DEFAULTS.SCAN_TIMEOUT);
    const showClosed = ref(false);
    const loading = ref(false);
    const result = ref<NetworkScanResult | null>(null);

    async function run() {
        loading.value = true;
        try {
            result.value = await invoke<NetworkScanResult>('network_scan_ports', {
                ipRange: ipRange.value,
                portsStr: ports.value,
                timeoutMs: timeout.value,
                showClosed: showClosed.value,
            });
        } catch (error: any) {
            ElMessage.error(error);
            result.value = null;
        } finally {
            loading.value = false;
        }
    }

    return {
        ipRange,
        ports,
        timeout,
        showClosed,
        loading,
        result,
        run,
    };
}
