<template>
    <div class="tool-panel">
        <div class="panel-body">
            <div class="toolbar">
                <div class="toolbar-row">
                    <el-input v-model="ipRange" :placeholder="t('toolbox.ipRangePlaceholder')" style="flex: 1" />
                    <el-input v-model="ports" :placeholder="t('toolbox.portsPlaceholder')" style="width: 150px" />
                    <div class="inline-controls">
                        <el-input-number v-model="timeout" :min="100" :max="10000" :step="100" style="width: 90px" />
                        <span class="unit-text">ms</span>
                    </div>
                    <el-checkbox v-model="showClosed">{{ t('toolbox.showClosed') }}</el-checkbox>
                    <el-button v-if="!loading" type="primary" @click="run" :disabled="!ipRange.trim() || !ports.trim()">
                        {{ t('toolbox.startScan') }}
                    </el-button>
                    <el-button v-else type="danger" @click="stop">
                        {{ t('toolbox.scanning') }}
                    </el-button>
                </div>
            </div>

            <div v-if="targets.length > 0" class="progress-bar-area">
                <el-progress :percentage="progress" :stroke-width="4" :show-text="true" :format="() => `${doneCount} / ${targets.length}`" />
                <span class="progress-summary">
                    <span class="stat-open">{{ openCount }} {{ t('toolbox.openPorts') }}</span>
                    <span class="stat-closed">{{ closedCount }} {{ t('toolbox.closedPorts') }}</span>
                </span>
            </div>

            <div v-if="groupedTargets.length > 0" class="scan-list">
                <div v-for="group in groupedTargets" :key="group.ip" class="ip-group">
                    <div class="ip-header" @click="toggleGroup(group.ip)">
                        <el-icon class="collapse-icon" :class="{ expanded: expandedGroups.has(group.ip) }"><ArrowRight /></el-icon>
                        <span class="ip-addr">{{ group.ip }}</span>
                        <span class="ip-badges">
                            <span v-if="group.openPorts.length > 0" class="badge-open">{{ group.openPorts.length }} {{ t('toolbox.openPorts') }}</span>
                            <span v-if="showClosed && group.closedPorts.length > 0" class="badge-closed">{{ group.closedPorts.length }} {{ t('toolbox.closedPorts') }}</span>
                            <span v-if="group.pendingPorts.length > 0" class="badge-pending">{{ group.pendingPorts.length }}</span>
                        </span>
                    </div>
                    <div v-if="expandedGroups.has(group.ip)" class="ip-ports">
                        <div
                            v-for="item in group.allPorts"
                            :key="item.port"
                            class="port-row"
                            :class="{ open: item.status === 'open', closed: item.status === 'closed', scanning: item.status === 'scanning', clickable: item.status === 'open' }"
                            @click="openUrl(item)">
                            <span class="port-num">:{{ item.port }}</span>
                            <span class="port-status">
                                <template v-if="item.status === 'pending'">
                                    <span class="status-waiting">—</span>
                                </template>
                                <template v-else-if="item.status === 'scanning'">
                                    <el-icon class="is-loading"><Loading /></el-icon>
                                </template>
                                <template v-else-if="item.status === 'open'">
                                    <span class="status-dot open-dot" />
                                    <span v-if="item.latencyMs != null" class="status-latency">{{ item.latencyMs }}ms</span>
                                    <el-icon class="link-icon"><Link /></el-icon>
                                </template>
                                <template v-else-if="item.status === 'closed'">
                                    <span class="status-dot closed-dot" />
                                </template>
                            </span>
                        </div>
                    </div>
                </div>
            </div>

            <div v-else-if="!loading" class="empty-hint">
                {{ t('toolbox.networkScanPlaceholder') }}
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { Loading, Link, ArrowRight } from '@element-plus/icons-vue';
import { open } from '@tauri-apps/plugin-shell';
import { useI18n } from 'vue-i18n';
import { useNetworkScan } from '../composables/tools/useNetworkScan';
import type { ScanTarget } from '../composables/tools/useNetworkScan';

const { t } = useI18n();
const { ipRange, ports, timeout, showClosed, loading, targets, openCount, closedCount, doneCount, progress, run, stop } =
    useNetworkScan();

const expandedGroups = ref<Set<string>>(new Set());

interface IpGroup {
    ip: string;
    openPorts: ScanTarget[];
    closedPorts: ScanTarget[];
    pendingPorts: ScanTarget[];
    allPorts: ScanTarget[];
}

const groupedTargets = computed(() => {
    const map = new Map<string, ScanTarget[]>();
    for (const item of targets.value) {
        if (!showClosed.value && item.status === 'closed') continue;
        const list = map.get(item.ip) || [];
        list.push(item);
        map.set(item.ip, list);
    }

    const groups: IpGroup[] = [];
    for (const [ip, allPorts] of map) {
        groups.push({
            ip,
            openPorts: allPorts.filter((p) => p.status === 'open'),
            closedPorts: allPorts.filter((p) => p.status === 'closed'),
            pendingPorts: allPorts.filter((p) => p.status === 'pending' || p.status === 'scanning'),
            allPorts,
        });
    }

    groups.sort((a, b) => {
        const aHasOpen = a.openPorts.length > 0 || a.pendingPorts.length > 0;
        const bHasOpen = b.openPorts.length > 0 || b.pendingPorts.length > 0;
        if (aHasOpen !== bHasOpen) return aHasOpen ? -1 : 1;
        return 0;
    });

    return groups;
});

watch(groupedTargets, (groups) => {
    for (const g of groups) {
        if (g.openPorts.length > 0 && !expandedGroups.value.has(g.ip)) {
            expandedGroups.value.add(g.ip);
        }
    }
});

function toggleGroup(ip: string) {
    if (expandedGroups.value.has(ip)) {
        expandedGroups.value.delete(ip);
    } else {
        expandedGroups.value.add(ip);
    }
}

async function openUrl(item: ScanTarget) {
    if (item.status !== 'open') return;
    await open(`http://${item.ip}:${item.port}`);
}
</script>

<style scoped>
.panel-body {
    display: flex;
    flex-direction: column;
    gap: 0;
    height: 100%;
    overflow: hidden;
}

.toolbar {
    padding-bottom: 10px;
    border-bottom: 1px solid var(--color-card-border, rgba(0, 0, 0, 0.06));
    flex-shrink: 0;
}

.toolbar-row {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-wrap: wrap;
}

.inline-controls {
    display: flex;
    align-items: center;
    gap: 2px;
}

.unit-text {
    font-size: var(--app-font-12, 12px);
    color: var(--color-text-tertiary, #999);
}

.progress-bar-area {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 0;
    flex-shrink: 0;
}

.progress-bar-area :deep(.el-progress) {
    flex: 1;
}

.progress-bar-area :deep(.el-progress__text) {
    font-size: var(--app-font-12, 12px) !important;
    min-width: 50px;
}

.progress-summary {
    display: flex;
    gap: 10px;
    font-size: var(--app-font-12, 12px);
    white-space: nowrap;
}

.stat-open {
    color: #2d8c3c;
    font-weight: 500;
}

.stat-closed {
    color: var(--color-text-tertiary, #999);
}

.scan-list {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding-top: 6px;
}

.ip-group {
    margin-bottom: 2px;
}

.ip-header {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 5px 6px;
    border-radius: 5px;
    cursor: pointer;
    transition: background 0.1s;
    user-select: none;
}

.ip-header:hover {
    background: var(--color-sidebar-item-hover, rgba(0, 0, 0, 0.04));
}

.collapse-icon {
    font-size: 12px;
    color: var(--color-text-tertiary, #999);
    transition: transform 0.15s;
    flex-shrink: 0;
}

.collapse-icon.expanded {
    transform: rotate(90deg);
}

.ip-addr {
    font-size: var(--app-font-13, 13px);
    font-weight: 500;
    font-family: 'SF Mono', Monaco, 'Cascadia Code', monospace;
    color: var(--color-text-primary, #333);
}

.ip-badges {
    margin-left: auto;
    display: flex;
    gap: 4px;
}

.badge-open {
    font-size: 11px;
    font-weight: 600;
    color: #2d8c3c;
    background: rgba(45, 140, 60, 0.1);
    padding: 1px 6px;
    border-radius: 9px;
}

.badge-closed {
    font-size: 11px;
    color: var(--color-text-tertiary, #999);
    background: var(--color-sidebar-item-hover, rgba(0, 0, 0, 0.04));
    padding: 1px 6px;
    border-radius: 9px;
}

.badge-pending {
    font-size: 11px;
    color: var(--color-menu-hover, #0078d7);
    background: rgba(0, 122, 255, 0.08);
    padding: 1px 6px;
    border-radius: 9px;
}

.ip-ports {
    padding-left: 20px;
}

.port-row {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 2px 8px;
    border-radius: 3px;
    font-size: var(--app-font-13, 13px);
    font-family: 'SF Mono', Monaco, 'Cascadia Code', monospace;
    transition: background 0.1s;
}

.port-row:hover {
    background: var(--color-sidebar-item-hover, rgba(0, 0, 0, 0.03));
}

.port-row.clickable {
    cursor: pointer;
}

.port-row.clickable:hover {
    background: rgba(0, 122, 255, 0.08);
}

.port-row.open {
    background: rgba(45, 140, 60, 0.04);
}

.port-row.scanning {
    color: var(--color-menu-hover, #0078d7);
}

.port-num {
    color: var(--color-text-secondary, #666);
    min-width: 50px;
}

.port-status {
    margin-left: auto;
    display: flex;
    align-items: center;
    gap: 4px;
}

.status-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
}

.open-dot {
    background: #2d8c3c;
}

.closed-dot {
    background: var(--color-text-tertiary, #ccc);
}

.status-waiting {
    color: var(--color-text-tertiary, #ccc);
    font-size: var(--app-font-12, 12px);
}

.status-latency {
    color: var(--color-text-tertiary, #999);
    font-size: var(--app-font-12, 12px);
}

.link-icon {
    color: var(--color-menu-hover, #0078d7);
    font-size: 11px;
    opacity: 0;
    transition: opacity 0.15s;
}

.port-row.clickable:hover .link-icon {
    opacity: 1;
}

.empty-hint {
    flex: 1;
    min-height: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--color-text-tertiary, #999);
    font-size: var(--app-font-13, 13px);
}
</style>
