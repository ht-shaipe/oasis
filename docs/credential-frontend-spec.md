# Credential Manager - Vue Frontend Spec

## Project
Oasis Tauri 2 应用，项目根目录: /Users/shaipe/workspace/rust/tools/oasis
前端代码目录: src/

## Task
实现凭证管理应用的前端部分，包括 Vue 组件、Tauri IPC 封装、i18n 文本。

## 1. 文件结构

```
src/
├── apps/
│   └── CredentialManager/
│       ├── index.vue             # 主组件（MacWindow 包裹，三视图状态机）
│       ├── AuthCard.vue          # 认证卡片
│       ├── Sidebar.vue           # 分类导航
│       └── Toolbar.vue           # 搜索与操作工具栏
├── composables/
│   └── useCredential.ts          # Tauri invoke 封装 + DEK 内存管理
├── locales/
│   ├── en.json                   # 追加 credential.* i18n 键
│   └── zh-CN.json                # 追加 credential.* i18n 键
├── views/
│   └── HomeView.vue              # 修改：注册 CredentialManager 组件
└── components/system/
    └── Dock.vue                  # 修改：添加凭证管理 Dock 图标
```

## 2. useCredential.ts - Tauri IPC 封装

```typescript
import { invoke } from '@tauri-apps/api/core'
import { ref, computed } from 'vue'

// DEK 仅存内存，不持久化
const dek = ref<string | null>(null)
const isLocked = computed(() => dek.value === null)

// 主密钥管理
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

  const lock = () => {
    dek.value = null
  }

  // 分类
  const listCategories = async () => {
    return invoke<Category[]>('list_categories')
  }

  const createCategory = async (name: string, icon?: string) => {
    return invoke<Category>('create_category', { name, icon })
  }

  // 凭证
  const listCredentials = async (categoryId?: number) => {
    return invoke<CredentialView[]>('list_credentials', { categoryId: categoryId ?? null })
  }

  const getCredential = async (id: number) => {
    if (!dek.value) throw new Error('Vault is locked')
    return invoke<CredentialDetail>('get_credential', { id, dekBase64: dek.value })
  }

  const createCredential = async (data: CreateCredentialRequest) => {
    if (!dek.value) throw new Error('Vault is locked')
    return invoke<CredentialView>('create_credential', {
      credential: {
        ...data,
        dekBase64: dek.value,
      }
    })
  }

  const updateCredential = async (data: UpdateCredentialRequest) => {
    if (!dek.value) throw new Error('Vault is locked')
    return invoke<CredentialView>('update_credential', {
      credential: {
        ...data,
        dekBase64: dek.value,
      }
    })
  }

  const deleteCredential = async (id: number) => {
    return invoke('delete_credential', { id })
  }

  const changeMasterKey = async (oldPassword: string, newPassword: string) => {
    const dekBase64 = await invoke<string>('change_master_key', { oldPassword, newPassword })
    dek.value = dekBase64
  }

  return {
    dek, isLocked,
    isMasterKeySet, setupMasterKey, unlock, lock,
    listCategories, createCategory,
    listCredentials, getCredential, createCredential, updateCredential, deleteCredential,
    changeMasterKey,
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
```

## 3. CredentialManager/index.vue - 主组件

### 三视图状态机
```
viewState: 'setup' | 'unlock' | 'main'
```

- 打开时调用 `isMasterKeySet()`
  - false → setup (首次设置主密钥)
  - true → unlock (输入主密钥解锁)
  - 解锁成功 → main

### SetupView
- 输入框：主密钥 + 确认主密钥
- 校验：两次一致 + 最少 8 字符
- 按钮：「设置密钥」
- 成功后自动切换到 main

### UnlockView
- 输入框：主密钥
- 按钮：「解锁」
- 错误提示：密钥不正确
- 失败 5 次后增加 2s 延迟（指数退避）
- 成功后切换到 main

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

#### 左侧分类导航 (200px 宽)
- 使用 el-menu 垂直模式
- 第一项「全部凭证」(category_id = null)
- 其余从 `listCategories()` 获取
- 底部「+ 新增分类」按钮
- 选中分类高亮

#### 右侧凭证列表
- 顶部：搜索框 (el-input) + 新增按钮 (el-button)
- 使用 el-table 展示
- 列：标题 | 用户名 | URL | 分类 | 更新时间 | 操作
- 操作：查看(eye) / 编辑(edit) / 删除(delete)
- 点击「查看」或双击行 → 弹出详情弹窗
- 点击「编辑」→ 弹出编辑弹窗
- 点击「删除」→ 确认后删除

#### 凭证编辑弹窗 (el-dialog)
- 基本信息区：
  - 标题 (必填)
  - 分类 (el-select)
  - 用户名
  - URL
  - 标签 (逗号分隔)
  - 备注
- 敏感信息区：
  - 密码 (带显示/隐藏切换 + 复制按钮)
  - API Key (同上)
  - Secret Key (同上)
  - Access Token (同上)
  - Refresh Token (同上)
  - 自定义字段 (key-value 动态行)
- 敏感字段默认 type="password"，点击 eye 图标切换显示

### 样式
- 延续项目已有的 macOS 风格（参考 Finder.vue）
- 使用 CSS 变量: `var(--color-sidebar-bg)`, `var(--color-text-*)`, `var(--color-input-bg)` 等
- 左侧边栏背景 `var(--color-sidebar-bg)`，右侧内容区 `var(--color-input-bg)`
- 整体使用 MacWindow 包裹，width="900" height="600"

### 自动锁定
- 组件内 setTimeout 30 分钟无操作自动 lock
- 窗口关闭时 lock
- 切换到其他 app 时可选 lock（后续优化）

## 4. HomeView.vue 修改

添加:
```typescript
import CredentialManager from '@/apps/CredentialManager/index.vue'

// 新增状态
const showCredentialManager = ref(false)
const isCredentialManagerMinimized = ref(false)

// openApp 的 switch 中新增:
case 'credential-manager':
    showCredentialManager.value = true
    isCredentialManagerMinimized.value = false
    break
```

Template 中新增（其他 Teleport 块旁边）:
```html
<Teleport to="body">
    <CredentialManager
        v-if="showCredentialManager"
        :isMinimized="isCredentialManagerMinimized"
        @close="showCredentialManager = false"
        @minimize="isCredentialManagerMinimized = !isCredentialManagerMinimized"
    />
</Teleport>
```

## 5. Dock.vue 修改

在 Dock 中添加一个凭证管理图标，放在 Settings 图标之前：
```html
<div class="dock-item" @click="openApp('credential-manager')">
    <img src="/assets/icons/Notes.png" :alt="t('dock.credentialManager')" :title="t('dock.credentialManager')">
</div>
```
（暂时用 Notes.png 图标，后续可替换为专用图标）

## 6. i18n 文本

### zh-CN.json 追加
```json
{
  "credential": {
    "title": "凭证管理",
    "setup": {
      "title": "设置主密钥",
      "password": "主密钥",
      "confirmPassword": "确认主密钥",
      "passwordHint": "至少8个字符，请妥善保管，丢失无法恢复",
      "submit": "设置密钥",
      "mismatch": "两次输入的密钥不一致",
      "tooShort": "密钥至少需要8个字符"
    },
    "unlock": {
      "title": "解锁凭证库",
      "password": "输入主密钥",
      "submit": "解锁",
      "wrongPassword": "密钥不正确",
      "tooManyAttempts": "尝试次数过多，请稍后再试"
    },
    "category": {
      "all": "全部凭证",
      "add": "新增分类",
      "name": "分类名称",
      "social": "社交媒体",
      "email": "邮箱",
      "devtools": "开发工具",
      "apikey": "API密钥",
      "cloud": "云服务",
      "database": "数据库",
      "custom": "自定义"
    },
    "list": {
      "search": "搜索凭证...",
      "add": "新增凭证",
      "empty": "暂无凭证",
      "title": "标题",
      "username": "用户名",
      "url": "网址",
      "category": "分类",
      "updatedAt": "更新时间",
      "actions": "操作",
      "view": "查看",
      "edit": "编辑",
      "delete": "删除",
      "deleteConfirm": "确定删除此凭证吗？此操作不可恢复。"
    },
    "detail": {
      "title": "凭证详情",
      "editTitle": "编辑凭证",
      "createTitle": "新增凭证",
      "basicInfo": "基本信息",
      "sensitiveInfo": "敏感信息",
      "password": "密码",
      "apiKey": "API Key",
      "secretKey": "Secret Key",
      "accessToken": "Access Token",
      "refreshToken": "Refresh Token",
      "customFields": "自定义字段",
      "addField": "添加字段",
      "tags": "标签",
      "notes": "备注",
      "save": "保存",
      "cancel": "取消",
      "copy": "复制",
      "copied": "已复制",
      "show": "显示",
      "hide": "隐藏"
    },
    "lock": "锁定",
    "autoLockWarning": "凭证库已自动锁定"
  }
}
```

### en.json 追加
```json
{
  "credential": {
    "title": "Credential Manager",
    "setup": {
      "title": "Set Master Key",
      "password": "Master Key",
      "confirmPassword": "Confirm Master Key",
      "passwordHint": "At least 8 characters. Keep it safe — it cannot be recovered if lost.",
      "submit": "Set Key",
      "mismatch": "Passwords do not match",
      "tooShort": "Password must be at least 8 characters"
    },
    "unlock": {
      "title": "Unlock Vault",
      "password": "Enter Master Key",
      "submit": "Unlock",
      "wrongPassword": "Incorrect master key",
      "tooManyAttempts": "Too many attempts, please try again later"
    },
    "category": {
      "all": "All Credentials",
      "add": "Add Category",
      "name": "Category Name",
      "social": "Social Media",
      "email": "Email",
      "devtools": "Developer Tools",
      "apikey": "API Keys",
      "cloud": "Cloud Services",
      "database": "Databases",
      "custom": "Custom"
    },
    "list": {
      "search": "Search credentials...",
      "add": "Add Credential",
      "empty": "No credentials yet",
      "title": "Title",
      "username": "Username",
      "url": "URL",
      "category": "Category",
      "updatedAt": "Updated",
      "actions": "Actions",
      "view": "View",
      "edit": "Edit",
      "delete": "Delete",
      "deleteConfirm": "Are you sure you want to delete this credential? This cannot be undone."
    },
    "detail": {
      "title": "Credential Detail",
      "editTitle": "Edit Credential",
      "createTitle": "New Credential",
      "basicInfo": "Basic Info",
      "sensitiveInfo": "Sensitive Info",
      "password": "Password",
      "apiKey": "API Key",
      "secretKey": "Secret Key",
      "accessToken": "Access Token",
      "refreshToken": "Refresh Token",
      "customFields": "Custom Fields",
      "addField": "Add Field",
      "tags": "Tags",
      "notes": "Notes",
      "save": "Save",
      "cancel": "Cancel",
      "copy": "Copy",
      "copied": "Copied",
      "show": "Show",
      "hide": "Hide"
    },
    "lock": "Lock",
    "autoLockWarning": "Vault has been auto-locked"
  }
}
```

## 7. 关键注意事项

- **DEK 安全**: dek 仅存于 composable 的 ref 中，不存 localStorage/sessionStorage
- **敏感字段展示**: 默认用 type="password" 遮罩，点击 eye 切换
- **复制功能**: 使用 `navigator.clipboard.writeText()`，复制后 30s 清空剪贴板（可选）
- **组件风格**: 严格参照 Finder.vue 的 macOS 风格，使用项目已有的 CSS 变量
- **Element Plus**: 使用项目已有的 el-* 组件
- **i18n**: 使用 `useI18n()` 的 `t()` 函数，所有用户可见文本都走 i18n
- **不要修改已有组件的逻辑**，只在 HomeView.vue 和 Dock.vue 中添加注册和图标

## 8. 验证

前端代码写完后不需要构建验证（后端可能还没完成），但确保：
- TypeScript 类型正确
- 组件 import 路径正确
- i18n key 格式正确
