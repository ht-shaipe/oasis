<template>
    <div class="site-list-container">
        <!-- 搜索和工具栏 -->
        <div class="toolbar">
            <el-input
                v-model="searchQuery"
                :placeholder="t('siteManager.searchPlaceholder')"
                class="search-input"
                @input="handleSearch">
                <template #prefix>
                    <el-icon><Search /></el-icon>
                </template>
            </el-input>
            <el-button type="primary" @click="handleAddSite">
                <el-icon><Plus /></el-icon>
                {{ t('siteManager.addSite') }}
            </el-button>
        </div>

        <!-- 分类筛选 -->
        <div v-if="categories.length > 0" class="category-filter">
            <el-tag
                :type="selectedCategoryId === null ? 'primary' : 'info'"
                class="category-tag"
                @click="filterByCategory(null)">
                {{ t('credential.category.all') }}
            </el-tag>
            <el-tag
                v-for="cat in flattenedCategories"
                :key="cat.id"
                :type="selectedCategoryId === cat.id ? 'primary' : 'info'"
                class="category-tag"
                @click="filterByCategory(cat.id)">
                {{ cat.name }}
            </el-tag>
        </div>

        <!-- 网站列表 -->
        <div v-if="loading" class="loading-state">
            <el-skeleton :rows="3" animated />
        </div>

        <div v-else-if="filteredSites.length === 0" class="empty-state">
            <el-empty :description="emptyDescription" />
        </div>

        <div v-else class="site-grid">
            <div
                v-for="site in filteredSites"
                :key="site.id"
                class="site-card"
                @click="handleViewSite(site)">
                <div class="site-card-header">
                    <div class="site-info">
                        <h5 class="site-name">{{ site.name }}</h5>
                        <a v-if="site.url" :href="site.url" target="_blank" class="site-url" @click.stop>
                            {{ getDomain(site.url) }}
                        </a>
                    </div>
                    <div class="site-actions" @click.stop>
                        <el-button-group size="small">
                            <el-button @click="handleEditSite(site)">
                                <el-icon><Edit /></el-icon>
                            </el-button>
                            <el-button type="danger" @click="handleDeleteSite(site)">
                                <el-icon><Delete /></el-icon>
                            </el-button>
                        </el-button-group>
                    </div>
                </div>

                <div class="site-card-body">
                    <div class="site-meta">
                        <el-tag v-if="site.category_name" size="small" type="info">
                            {{ site.category_name }}
                        </el-tag>
                        <span v-if="site.accounts_count" class="accounts-count">
                            {{ t('siteManager.accountCount', { count: site.accounts_count }) }}
                        </span>
                    </div>

                    <div v-if="site.tags" class="site-tags">
                        <el-tag
                            v-for="tag in site.tags.split(',')"
                            :key="tag"
                            size="small"
                            class="tag-item">
                            {{ tag.trim() }}
                        </el-tag>
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { ElMessage, ElMessageBox } from 'element-plus';
import { Search, Edit, Delete, Plus } from '@element-plus/icons-vue';
import { useCredential, type Site, type Category } from '@/composables/useCredential';

const { t } = useI18n();
const {
    listSites,
    deleteSite,
    searchSites,
} = useCredential();

// Props
const props = defineProps<{
    categories: Category[];
}>();

// Emits
const emit = defineEmits<{
    addSite: [];
    editSite: [site: Site];
    viewSite: [site: Site];
}>();

// State
const loading = ref(false);
const sites = ref<Site[]>([]);
const searchQuery = ref('');
const selectedCategoryId = ref<number | null>(null);

// 计算属性
const flattenedCategories = computed(() => {
    const result: Array<{ id: number; name: string; level: number }> = [];
    const traverse = (nodes: Category[], level: number) => {
        nodes.forEach((node) => {
            result.push({ id: node.id, name: node.name, level });
            // 假设没有嵌套分类，如果需要可以递归处理子分类
        });
    };
    traverse(props.categories, 0);
    return result;
});

const filteredSites = computed(() => {
    let result = sites.value;

    // 分类筛选
    if (selectedCategoryId.value !== null) {
        result = result.filter((site) => site.category_id === selectedCategoryId.value);
    }

    // 搜索筛选
    if (searchQuery.value.trim()) {
        const query = searchQuery.value.toLowerCase();
        result = result.filter(
            (site) =>
                site.name.toLowerCase().includes(query) ||
                (site.url && site.url.toLowerCase().includes(query)) ||
                (site.tags && site.tags.toLowerCase().includes(query))
        );
    }

    return result;
});

const emptyDescription = computed(() => {
    if (searchQuery.value.trim()) {
        return t('siteManager.noSearchResults');
    }
    if (selectedCategoryId.value !== null) {
        return t('siteManager.noSitesInCategory');
    }
    return t('siteManager.noSites');
});

// 方法
const getDomain = (url: string): string => {
    try {
        const urlObj = new URL(url);
        return urlObj.hostname;
    } catch {
        return url;
    }
};

const loadSites = async () => {
    loading.value = true;
    try {
        sites.value = await listSites();
    } catch (error) {
        console.error('Failed to load sites:', error);
        ElMessage.error(t('siteManager.loadFailed'));
    } finally {
        loading.value = false;
    }
};

const handleSearch = async () => {
    if (!searchQuery.value.trim()) {
        await loadSites();
        return;
    }

    try {
        sites.value = await searchSites(searchQuery.value);
    } catch (error) {
        console.error('Search failed:', error);
    }
};

const filterByCategory = (categoryId: number | null) => {
    selectedCategoryId.value = categoryId;
};

const handleAddSite = () => {
    emit('addSite');
};

const handleEditSite = (site: Site) => {
    emit('editSite', site);
};

const handleViewSite = (site: Site) => {
    emit('viewSite', site);
};

const handleDeleteSite = async (site: Site) => {
    try {
        await ElMessageBox.confirm(
            t('siteManager.confirmDeleteSite', { name: site.name }),
            t('siteManager.deleteSite'),
            {
                confirmButtonText: t('app.confirm'),
                cancelButtonText: t('app.cancel'),
                type: 'warning',
            }
        );

        await deleteSite(site.id);
        ElMessage.success(t('siteManager.siteDeleted'));
        await loadSites();
    } catch (error) {
        if (error !== 'cancel') {
            console.error('Delete failed:', error);
        }
    }
};

// 生命周期
onMounted(() => {
    loadSites();
});
</script>

<style scoped>
.site-list-container {
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 16px;
    height: 100%;
}

.toolbar {
    display: flex;
    gap: 12px;
    align-items: center;
}

.search-input {
    flex: 1;
}

.category-filter {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
}

.category-tag {
    cursor: pointer;
    transition: all 0.2s;
}

.category-tag:hover {
    transform: translateY(-1px);
}

.loading-state {
    padding: 20px;
}

.empty-state {
    display: flex;
    justify-content: center;
    padding: 40px 20px;
}

.site-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: 16px;
}

.site-card {
    padding: 16px;
    border: 1px solid var(--color-input-border);
    border-radius: 8px;
    background-color: var(--color-card-bg);
    cursor: pointer;
    transition: all 0.2s;
}

.site-card:hover {
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
    border-color: var(--color-primary);
}

.site-card-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 12px;
}

.site-info {
    flex: 1;
    min-width: 0;
}

.site-name {
    margin: 0 0 4px 0;
    font-size: 16px;
    font-weight: 600;
    color: var(--color-text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.site-url {
    font-size: 12px;
    color: var(--color-link);
    text-decoration: none;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    display: block;
}

.site-url:hover {
    text-decoration: underline;
}

.site-card-body {
    display: flex;
    flex-direction: column;
    gap: 8px;
}

.site-meta {
    display: flex;
    align-items: center;
    gap: 8px;
}

.accounts-count {
    font-size: 12px;
    color: var(--color-text-secondary);
}

.site-tags {
    display: flex;
    gap: 4px;
    flex-wrap: wrap;
}

.tag-item {
    font-size: 11px;
}

/* 响应式 */
@media (max-width: 768px) {
    .site-grid {
        grid-template-columns: 1fr;
    }

    .toolbar {
        flex-direction: column;
        align-items: stretch;
    }

    .search-input {
        width: 100%;
    }
}
</style>
