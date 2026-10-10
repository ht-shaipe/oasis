import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { ElMessage } from 'element-plus';
import { useI18n } from 'vue-i18n';
import { DEFAULTS } from '../../constants';

export type ScanStatus = 'pending' | 'scanning' | 'open' | 'closed';

export interface ScanTarget {
    ip: string;
    port: number;
    status: ScanStatus;
    latencyMs: number | null;
}

export function useNetworkScan() {
    const { t } = useI18n();
    const ipRange = ref(DEFAULTS.SCAN_IP_RANGE);
    const ports = ref(DEFAULTS.SCAN_PORTS);
    const timeout = ref(DEFAULTS.SCAN_TIMEOUT);
    const showClosed = ref(false);
    const loading = ref(false);
    const targets = ref<ScanTarget[]>([]);

    const CONCURRENCY = 32;

    const openCount = computed(() => targets.value.filter((i) => i.status === 'open').length);
    const closedCount = computed(() => targets.value.filter((i) => i.status === 'closed').length);
    const doneCount = computed(() => targets.value.filter((i) => i.status === 'open' || i.status === 'closed').length);
    const progress = computed(() =>
        targets.value.length > 0 ? Math.round((doneCount.value / targets.value.length) * 100) : 0,
    );

    async function scanSingle(index: number): Promise<void> {
        const item = targets.value[index];
        if (!item || item.status !== 'pending') return;

        item.status = 'scanning';
        try {
            const result = await invoke<{ ip: string; port: number; open: boolean; latency_ms: number | null }>(
                'network_scan_single',
                {
                    ip: item.ip,
                    port: item.port,
                    timeoutMs: timeout.value,
                },
            );
            item.status = result.open ? 'open' : 'closed';
            item.latencyMs = result.latency_ms;
        } catch {
            item.status = 'closed';
            item.latencyMs = null;
        }
    }

    async function run() {
        if (!ipRange.value.trim() || !ports.value.trim()) {
            ElMessage.warning(t('toolbox.dirRequired'));
            return;
        }

        let parsed: [string, number][];
        try {
            parsed = await invoke<[string, number][]>('network_scan_parse', {
                ipRange: ipRange.value,
                portsStr: ports.value,
            });
        } catch (error: any) {
            ElMessage.error(error);
            return;
        }

        if (parsed.length === 0) {
            ElMessage.warning(t('toolbox.dirRequired'));
            return;
        }

        targets.value = parsed.map(([ip, port]) => ({
            ip,
            port,
            status: 'pending' as ScanStatus,
            latencyMs: null,
        }));

        loading.value = true;

        let cursor = 0;
        async function next(): Promise<void> {
            while (cursor < targets.value.length) {
                const idx = cursor++;
                await scanSingle(idx);
            }
        }

        const workers = Array.from({ length: Math.min(CONCURRENCY, targets.value.length) }, () => next());
        await Promise.all(workers);

        loading.value = false;

        ElMessage.success(
            t('toolbox.scanDone', {
                success: openCount.value,
                fail: closedCount.value,
            }),
        );
    }

    function stop() {
        loading.value = false;
        targets.value = targets.value.map((t) =>
            t.status === 'pending' || t.status === 'scanning' ? { ...t, status: 'closed' as ScanStatus } : t,
        );
    }

    return {
        ipRange,
        ports,
        timeout,
        showClosed,
        loading,
        targets,
        openCount,
        closedCount,
        doneCount,
        progress,
        run,
        stop,
    };
}
