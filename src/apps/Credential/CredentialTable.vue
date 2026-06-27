<template>
    <div class="credential-table-wrapper">
        <el-empty v-if="total === 0 && !loading" :description="emptyDescription" />
        <el-table v-else v-loading="loading" :data="data" style="width: 100%" @row-dblclick="handleRowDblClick">
            <el-table-column :label="titleLabel" min-width="200">
                <template #default="{ row }">
                    <div class="cred-title min-w-0">
                        <el-icon class="shrink-0"><Key /></el-icon>
                        <span class="truncate flex-1" :title="row.title">{{ row.title }}</span>
                    </div>
                </template>
            </el-table-column>
            <el-table-column :label="usernameLabel" min-width="120">
                <template #default="{ row }">
                    <div class="truncate" :title="row.username || '-'">
                        {{ row.username || '-' }}
                    </div>
                </template>
            </el-table-column>
            <el-table-column :label="accountsCountLabel" width="80" align="center">
                <template #default="{ row }">
                    <el-tag v-if="row.accounts_count && row.accounts_count > 1" size="small" round>{{ row.accounts_count }}</el-tag>
                    <span v-else>1</span>
                </template>
            </el-table-column>
            <el-table-column :label="urlLabel" min-width="180">
                <template #default="{ row }">
                    <div class="flex min-w-0 items-center gap-1">
                        <span class="truncate flex-1" :title="row.url || '-'">
                            {{ row.url || '-' }}
                        </span>
                        <el-button v-if="row.url" link size="small" @click.stop="handleOpenUrl(row.url)">
                            <el-icon :size="14"><Link /></el-icon>
                        </el-button>
                    </div>
                </template>
            </el-table-column>
            <el-table-column :label="categoryLabel" width="120">
                <template #default="{ row }">
                    {{ row.category_name || '-' }}
                </template>
            </el-table-column>
            <el-table-column :label="updatedAtLabel" width="160">
                <template #default="{ row }">
                    {{ formatDate(row.updated_at) }}
                </template>
            </el-table-column>
            <el-table-column :label="actionsLabel" width="120" fixed="right">
                <template #default="{ row }">
                    <el-button type="primary" link size="small" @click="handleView(row)">
                        <el-icon :size="16"><View /></el-icon>
                    </el-button>
                    <el-button link size="small" @click="handleEdit(row)">
                        <el-icon :size="16"><Edit /></el-icon>
                    </el-button>
                    <el-button link size="small" type="danger" @click="handleDelete(row)">
                        <el-icon :size="16"><Delete /></el-icon>
                    </el-button>
                </template>
            </el-table-column>
        </el-table>
        <div v-if="total > 0" class="flex justify-end px-2 py-3">
            <el-pagination
                v-model:current-page="internalCurrentPage"
                v-model:page-size="internalPageSize"
                :page-sizes="[10, 20, 50, 100]"
                :total="total"
                layout="total, sizes, prev, pager, next, jumper"
                background
                small
                @size-change="handlePageSizeChange"
                @current-change="handleCurrentPageChange" />
        </div>
    </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { Delete, Edit, View, Key, Link } from '@element-plus/icons-vue';
import type { CredentialView } from '@/composables/useCredential';

interface Props {
    data: CredentialView[];
    loading: boolean;
    total: number;
    currentPage: number;
    pageSize: number;
    emptyDescription: string;
    titleLabel: string;
    usernameLabel: string;
    accountsCountLabel: string;
    urlLabel: string;
    categoryLabel: string;
    updatedAtLabel: string;
    actionsLabel: string;
}

interface Emits {
    (e: 'row-dblclick', row: CredentialView): void;
    (e: 'view', row: CredentialView): void;
    (e: 'edit', row: CredentialView): void;
    (e: 'delete', row: CredentialView): void;
    (e: 'open-url', url: string): void;
    (e: 'update:currentPage', page: number): void;
    (e: 'update:pageSize', size: number): void;
    (e: 'page-change', page: number): void;
    (e: 'size-change', size: number): void;
}

const props = withDefaults(defineProps<Props>(), {
    data: () => [],
    loading: false,
    total: 0,
    currentPage: 1,
    pageSize: 15,
});

const emit = defineEmits<Emits>();

const internalCurrentPage = computed({
    get: () => props.currentPage,
    set: (val) => emit('update:currentPage', val),
});

const internalPageSize = computed({
    get: () => props.pageSize,
    set: (val) => emit('update:pageSize', val),
});

const handleRowDblClick = (row: CredentialView) => {
    emit('row-dblclick', row);
};

const handleView = (row: CredentialView) => {
    emit('view', row);
};

const handleEdit = (row: CredentialView) => {
    emit('edit', row);
};

const handleDelete = (row: CredentialView) => {
    emit('delete', row);
};

const handleOpenUrl = (url: string) => {
    emit('open-url', url);
};

const handlePageSizeChange = (size: number) => {
    emit('size-change', size);
};

const handleCurrentPageChange = (page: number) => {
    emit('page-change', page);
};

const formatDate = (dateStr: string | null | undefined): string => {
    if (!dateStr) return '-';
    const d = new Date(dateStr);
    return `${d.toLocaleDateString()} ${d.toLocaleTimeString()}`;
};
</script>

<style scoped>
.cred-title {
    display: flex;
    align-items: center;
    gap: 6px;
}

.cred-title .el-icon {
    color: #e6a23c;
}

.truncate {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.credential-table-wrapper {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-height: 0;
}

.credential-table-wrapper :deep(.el-table) {
    flex: 1;
    min-height: 0;
}
</style>
