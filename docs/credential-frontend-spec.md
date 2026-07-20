# Credential Manager - Vue Frontend Spec

## 项目
Oasis Tauri 2 应用，项目根目录: /Users/shaipe/workspace/rust/tools/oasis
前端代码目录: src/

## 概述
凭证管理应用的前端部分，包括 Vue 组件、Tauri IPC 封装、i18n 文本。

## 1. 文件结构

```
src/
├── apps/
│   └── Credential/
│       ├── Index.vue               # 主组件（MacWindow 包裹，三视图状态机）
│       ├── AuthCard.vue            # 认证卡片
│       ├── Sidebar.vue             # 分类导航
│       ├── Toolbar.vue             # 搜索与操作工具栏
│       ├── CredentialFormDialog.vue # 凭证编辑/新增弹窗
│       ├── CredentialTable.vue     # 凭证列表表格
│       ├── SiteAccountManager.vue  # 网站账号管理器
│       ├── SiteList.vue            # 网站列表
│       ├── BrowserImportDialog.vue # 浏览器数据导入
│       ├── MergePreviewDialog.vue  # 合并预览
│       ├── TemplateManager.vue     # 模板管理
│       └── credentialForm.ts       # 表单逻辑
├── composables/
│   └── useCredential.ts            # Tauri invoke 封装 + DEK 内存管理
├── locales/
│   ├── en.json                     # 追加 credential.* i18n 键
│   └── zh-CN.json                  # 追加 credential.* i18n 键
└── config/
    └── apps.ts                     # 应用注册（credential-manager）
```

## 2. useCredential.ts - Tauri IPC 封装

```typescript
import { invoke } from '@tauri-apps/api/core'
import { ref, computed } from 'vue'

const dek = ref<string | null>(null)
const isLocked = computed(() => dek.value === null)

export function useCredential() {
  const isMasterKeySet = async (): Promise<boolean> => {
    return invoke('is_master_key_set')
  }

  const setupMasterKey = async (password: string): Promise<void> => {
    const dekBase64 = await invoke<string>('setup_master_key', { password })
    dek.value = dekBase64
  }

  const unlock = async (password: string): Promise<void> => {
    const dekBase64 = await invoke<string>('verify_master_key', { password })
    dek.value = dekBase64
  }

  const lock = () => { dek.value = null }

  const listCategories = async () => invoke<Category[]>('list_categories')
  const createCategory = async (name: string, icon?: string) =>
    invoke<Category>('create_category', { name, icon })

  const listCredentials = async (categoryId?: number) =>
    invoke<CredentialView[]>('list_credentials', { categoryId: categoryId ?? null })
  const getCredential = async (id: number) => {
    if (!dek.value) throw new Error('Vault is locked')
    return invoke<CredentialDetail>('get_credential', { id, dekBase64: dek.value })
  }
  const createCredential = async (data: CreateCredentialRequest) => {
    if (!dek.value) throw new Error('Vault is locked')
    return invoke<CredentialView>('create_credential', {
      credential: { ...data, dekBase64: dek.value }
    })
  }
  const updateCredential = async (data: UpdateCredentialRequest) => {
    if (!dek.value) throw new Error('Vault is locked')
    return invoke<CredentialView>('update_credential', {
      credential: { ...data, dekBase64: dek.value }
    })
  }
  const deleteCredential = async (id: number) =>
    invoke('delete_credential', { id })

  const changeMasterKey = async (oldPassword: string, newPassword: string) => {
    const dekBase64 = await invoke<string>('change_master_key', { oldPassword, newPassword })
    dek.value = dekBase64
  }

  // Site management
  const listSites = async (categoryId?: number) =>
    invoke<Site[]>('list_sites', { categoryId: categoryId ?? null })
  const getSite = async (id: number) => {
    if (!dek.value) throw new Error('Vault is locked')
    return invoke<SiteDetail>('get_site', { id, dekBase64: dek.value })
  }
  const createSite = async (data: CreateSiteRequest) => {
    if (!dek.value) throw new Error('Vault is locked')
    return invoke<Site>('create_site', { site: { ...data, dekBase64: dek.value } })
  }
  const updateSite = async (data: UpdateSiteRequest) => {
    if (!dek.value) throw new Error('Vault is locked')
    return invoke<Site>('update_site', { site: { ...data, dekBase64: dek.value } })
  }
  const deleteSite = async (id: number) => invoke('delete_site', { id })
  const searchSites = async (query: string) =>
    invoke<Site[]>('search_sites', { query })

  return {
    dek, isLocked,
    isMasterKeySet, setupMasterKey, unlock, lock,
    listCategories, createCategory,
    listCredentials, getCredential, createCredential, updateCredential, deleteCredential,
    changeMasterKey,
    listSites, getSite, createSite, updateSite, deleteSite, searchSites,
  }
}

// 类型定义
export interface Category {
  id: number
  name: string
  icon: string | null
  sort_order: number
  created_at: string
}

export interface CredentialView {
  id: number
  category_id: number
  title: string
  username: string | null
  url: string | null
  tags: string | null
  notes: string | null
  created_at: string
  updated_at: string
  category_name: string | null
}

export interface CredentialDetail extends CredentialView {
  sensitive_data: SensitiveData
}

export interface SensitiveData {
  password?: string
  secret_key?: string
  access_token?: string
  refresh_token?: string
  api_key?: string
  custom_fields?: Record<string, string>
}

export interface Site {
  id: number
  name: string
  url: string | null
  category_id: number
  tags: string | null
  notes: string | null
  created_at: string
  updated_at: string
}

export interface SiteDetail extends Site {
  accounts: SiteAccount[]
}

export interface SiteAccount {
  id: number
  username: string
  password: string
  api_key?: string
  secret_key?: string
}

export interface CreateCredentialRequest {
  category_id: number
  title: string
  username?: string
  url?: string
  sensitive_data_json: string
  tags?: string
  notes?: string
}

export interface UpdateCredentialRequest {
  id: number
  category_id?: number
  title?: string
  username?: string
  url?: string
  sensitive_data_json?: string
  tags?: string
  notes?: string
}

export interface CreateSiteRequest {
  name: string
  url?: string
  category_id: number
  tags?: string
  notes?: string
  accounts_json: string
}

export interface UpdateSiteRequest {
  id: number
  name?: string
  url?: string
  category_id?: number
  tags?: string
  notes?: string
  accounts_json?: string
}
```

## 3. Credential/Index.vue - 主组件

### 三视图状态机
```
viewState: 'setup' | 'unlock' | 'main'
```

- 打开时调用 `isMasterKeySet()`
  - false → setup (首次设置主密钥)
  - true → unlock (输入主密钥解锁)
  - 解锁成功 → main

### MainView - 三栏布局
```
┌──────────────────────────────────────────────┐
│ 🔑 Credential Manager          [🔒 锁定]     │
├──────────┬───────────────────────────────────┤
│ 分类导航  │  搜索框 [+ 新增]                   │
│          │                                    │
│ ▸ 全部   │  ┌──────────────────────────────┐  │
│   社交媒体│  │ 🔑 GitHub  grahamc  ****    │  │
│   邮箱   │  │ 🔑 AWS     root     ****    │  │
│   开发工具│  │ 🔑 Gmail   user@    ****    │  │
│   API密钥 │  └──────────────────────────────┘  │
│   云服务  │                                    │
│   数据库  │                                    │
│   自定义  │                                    │
└──────────┴───────────────────────────────────┘
```

## 4. 关键注意事项

- **DEK 安全**: dek 仅存于 composable 的 ref 中，不存 localStorage/sessionStorage
- **敏感字段展示**: 默认用 type="password" 遮罩，点击 eye 切换
- **复制功能**: 使用 `navigator.clipboard.writeText()`，复制后 30s 清空剪贴板（可选）
- **组件风格**: 严格参照 Finder.vue 的 macOS 风格，使用项目已有的 CSS 变量
- **Element Plus**: 使用项目已有的 el-* 组件
- **i18n**: 使用 `useI18n()` 的 `t()` 函数，所有用户可见文本都走 i18n
