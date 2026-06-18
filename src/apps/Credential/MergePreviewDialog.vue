<template>
    <AppDialog
        v-model="visible"
        title="整理凭证"
        width="700px"
        append-to-body
        destroy-on-close
        @closed="handleClosed">
        <div v-if="loading" class="flex items-center justify-center py-8">
            <el-icon class="is-loading" :size="20"><Loading /></el-icon>
            <span class="ml-2 text-gray-500">正在分析凭证...</span>
        </div>

        <template v-else-if="preview">
            <div class="merge-stats">
                <div class="stat-item">
                    <span class="stat-value">{{ preview.total_credentials }}</span>
                    <span class="stat-label">凭证总数</span>
                </div>
                <div class="stat-item">
                    <span class="stat-value text-amber-500">{{ preview.duplicates_count }}</span>
                    <span class="stat-label">重复凭证</span>
                </div>
                <div class="stat-item">
                    <span class="stat-value text-blue-500">{{ displayGroups.length }}</span>
                    <span class="stat-label">可合并分组</span>
                </div>
            </div>

            <div class="merge-options">
                <el-checkbox v-model="filterIntranet">
                    过滤内网地址
                    <template v-if="preview.intranet_groups_count > 0">
                        ({{ preview.intranet_groups_count }} 组 / {{ preview.intranet_credential_count }} 条)
                    </template>
                </el-checkbox>
            </div>

            <div v-if="displayGroups.length === 0 && preview.duplicates_count === 0" class="no-merge-tip">
                <el-icon :size="32" color="#9ca3af"><CircleCheckFilled /></el-icon>
                <p>凭证已经整理完毕，无需操作</p>
            </div>

            <template v-else>
                <div v-if="preview.duplicates_count > 0" class="dedup-info">
                    <el-icon color="#e6a23c" :size="16"><WarningFilled /></el-icon>
                    <span>将去除 <strong>{{ preview.duplicates_count }}</strong> 条完全重复的凭证（URL + 用户名 + 密码相同）</span>
                </div>

                <div v-if="displayGroups.length > 0" class="merge-group-list">
                    <div class="group-list-header">同域名多账号分组（合并为站点）：</div>
                    <div
                        v-for="group in displayGroups"
                        :key="group.hostname"
                        class="group-card">
                        <div class="group-host">
                            <span class="hostname">{{ group.hostname }}</span>
                            <el-tag size="small" type="info">{{ group.credential_count }} 条凭证</el-tag>
                            <el-tag v-if="group.is_intranet" size="small" type="warning">内网</el-tag>
                        </div>
                        <div class="group-usernames">
                            <el-tag
                                v-for="name in group.usernames.slice(0, 5)"
                                :key="name"
                                size="small"
                                class="username-tag">
                                {{ name }}
                            </el-tag>
                            <span v-if="group.usernames.length > 5" class="text-gray-400 text-xs">
                                +{{ group.usernames.length - 5 }} 个
                            </span>
                        </div>
                    </div>
                </div>
            </template>

            <div class="merge-footer">
                <el-button @click="visible = false">取消</el-button>
                <el-button
                    type="primary"
                    :disabled="preview.duplicates_count === 0 && displayGroups.length === 0"
                    :loading="merging"
                    @click="handleMerge">
                    确认整理
                </el-button>
            </div>
        </template>
    </AppDialog>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { ElMessage } from 'element-plus';
import { Loading, WarningFilled, CircleCheckFilled } from '@element-plus/icons-vue';
import { useCredential } from '@/composables/useCredential';
import AppDialog from '@/components/common/AppDialog.vue';

interface MergeGroup {
    hostname: string;
    credential_count: number;
    usernames: string[];
    is_intranet: boolean;
    sample_url: string;
}

interface MergePreview {
    total_credentials: number;
    duplicates_count: number;
    merge_groups: MergeGroup[];
    intranet_groups_count: number;
    intranet_credential_count: number;
}

const props = defineProps<{
    modelValue: boolean;
    dek: string | null;
}>();

const emit = defineEmits<{
    (e: 'update:modelValue', value: boolean): void;
    (e: 'merged'): void;
}>();

const visible = computed({
    get: () => props.modelValue,
    set: (v) => emit('update:modelValue', v),
});

const { previewMergeByUrl, mergeCredentialsByUrl } = useCredential();

const loading = ref(false);
const merging = ref(false);
const preview = ref<MergePreview | null>(null);
const filterIntranet = ref(true);

const displayGroups = computed(() => {
    if (!preview.value) return [];
    if (!filterIntranet.value) return preview.value.merge_groups;
    return preview.value.merge_groups.filter((g) => !g.is_intranet);
});

const loadPreview = async () => {
    if (!props.dek) return;
    loading.value = true;
    try {
        preview.value = await previewMergeByUrl(props.dek);
    } catch (err: unknown) {
        ElMessage.error(err instanceof Error ? err.message : '分析凭证失败');
        preview.value = null;
    } finally {
        loading.value = false;
    }
};

watch(
    () => props.modelValue,
    (val) => {
        if (val) loadPreview();
    },
);

const handleMerge = async () => {
    if (!props.dek) {
        ElMessage.warning('保险库未解锁');
        return;
    }

    merging.value = true;
    try {
        const result = await mergeCredentialsByUrl(filterIntranet.value);
        const parts: string[] = [];
        if (result.duplicates_removed > 0) parts.push(`去重 ${result.duplicates_removed} 条`);
        if (result.sites_created > 0) parts.push(`创建 ${result.sites_created} 个站点`);
        if (result.accounts_created > 0) parts.push(`${result.accounts_created} 个账号`);
        if (result.intranet_skipped > 0) parts.push(`内网跳过 ${result.intranet_skipped} 条`);

        if (result.duplicates_removed > 0 || result.sites_created > 0) {
            ElMessage.success(`整理完成：${parts.join('，')}`);
        } else if (result.intranet_skipped > 0) {
            ElMessage.info(`无需要整理的凭证（内网跳过 ${result.intranet_skipped} 条）`);
        } else {
            ElMessage.info('凭证已经整理完毕');
        }

        visible.value = false;
        emit('merged');
    } catch (err: unknown) {
        ElMessage.error(err instanceof Error ? err.message : '整理失败');
    } finally {
        merging.value = false;
    }
};

const handleClosed = () => {
    preview.value = null;
    filterIntranet.value = true;
    loading.value = false;
    merging.value = false;
};
</script>

<style scoped>
.merge-stats {
    display: flex;
    gap: 24px;
    margin-bottom: 16px;
    padding: 16px;
    background: #f9fafb;
    border-radius: 8px;
}

.stat-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
}

.stat-value {
    font-size: 24px;
    font-weight: 600;
    line-height: 1;
}

.stat-label {
    font-size: 12px;
    color: #9ca3af;
}

.merge-options {
    margin-bottom: 16px;
    padding-bottom: 12px;
    border-bottom: 1px solid #e5e7eb;
}

.dedup-info {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 10px 12px;
    background: #fdf6ec;
    border-radius: 6px;
    font-size: 13px;
    color: #6b4c00;
    margin-bottom: 12px;
}

.merge-group-list {
    max-height: 320px;
    overflow-y: auto;
}

.group-list-header {
    font-size: 13px;
    font-weight: 500;
    color: #374151;
    margin-bottom: 8px;
}

.group-card {
    padding: 10px 12px;
    border: 1px solid #e5e7eb;
    border-radius: 6px;
    margin-bottom: 6px;
}

.group-host {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 6px;
}

.hostname {
    font-size: 13px;
    font-weight: 500;
    color: #1f2937;
    word-break: break-all;
}

.group-usernames {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
}

.username-tag {
    max-width: 160px;
    overflow: hidden;
    text-overflow: ellipsis;
}

.no-merge-tip {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 32px 0;
    color: #9ca3af;
    font-size: 14px;
}

.merge-footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 16px;
    padding-top: 12px;
    border-top: 1px solid #e5e7eb;
}
</style>
