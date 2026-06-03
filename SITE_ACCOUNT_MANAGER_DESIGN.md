# 網站賬號管理器設計文檔

## 📋 概述

為了解決"一個網站下有多套賬號密碼"的常見需求，我們設計了一個新的**網站賬號管理器**組件。

## 🎯 使用場景

### 場景示例
```
網站: GitHub (https://github.com)
├─ 賬號1: user1@example.com / password1
├─ 賬號2: user2@example.com / password2  
└─ 賬號3: user3@example.com / password3

網站: Google (https://google.com)
├─ 賬號1: personal@gmail.com / personal_pwd
└─ 賬號2: work@company.com / work_pwd
```

## 📊 數據結構設計

### 方案對比

#### ✅ 推薦方案：主從結構
```
Site (網站主記錄)
├─ id: 數字
├─ name: "GitHub"
├─ url: "https://github.com"
├─ category_id: 分類ID
├─ tags: "開發,代碼"
├─ notes: "主要用於代碼托管"
└─ accounts: Account[] (賬號列表)

Account (賬號子記錄)  
├─ username: "user1@example.com"
├─ password: "encrypted_password"
├─ api_key: "可選的API密鑰"
└─ secret_key: "可選的Secret"
```

#### 優點
- **結構清晰**：網站信息與賬號信息分離
- **易於管理**：可以獨立添加/編輯/刪除賬號
- **查詢高效**：按網站分組，方便查找
- **靈活擴展**：未來可以添加更多賬號屬性

### 替代方案：嵌套結構（快速實現）
在現有Credential基礎上添加accounts數組字段。

## 🔧 功能特性

### 核心功能
1. **網站管理**
   - 添加/編輯網站信息
   - 網站名稱、網址、分類、標籤、備註

2. **賬號管理**
   - 一個網站下可添加多個賬號
   - 每個賬號獨立管理用戶名和密碼
   - 支持可選字段：API Key、Secret

3. **賬號操作**
   - 添加新賬號
   - 編輯現有賬號
   - 刪除賬號（至少保留一個）

4. **數據驗證**
   - 網站名稱必填
   - 分類必選
   - 至少需要一個賬號
   - 每個賬號的用戶名和密碼必填

## 💻 組件說明

### SiteAccountManager.vue

#### Props
```typescript
{
  modelValue: boolean,        // 對話框顯示狀態
  categories: Array<{         // 分類列表
    id: number,
    name: string,
    level: number
  }>,
  editingSite?: any,         // 編輯的網站數據（可選）
  dek?: string | null        // 加密密鑰（可選）
}
```

#### Emits
```typescript
{
  'update:modelValue': (val: boolean) => void,  // 更新顯示狀態
  'saved': (data: any) => void                   // 保存完成，返回數據
}
```

## 📝 使用示例

```vue
<template>
  <SiteAccountManager
    v-model="showDialog"
    :categories="categories"
    :editing-site="currentSite"
    :dek="dek"
    @saved="handleSiteSaved" />
</template>

<script setup>
const handleSiteSaved = (data) => {
  console.log('網站數據:', data);
  // {
  //   id: null,
  //   name: 'GitHub',
  //   url: 'https://github.com',
  //   category_id: 1,
  //   tags: '開發,代碼',
  //   notes: '代碼托管平台',
  //   accounts: [
  //     { username: 'user1@example.com', password: '***' },
  //     { username: 'user2@example.com', password: '***' }
  //   ]
  // }
};
</script>
```

## 🔄 與現有系統集成

### 數據存儲建議

#### 方案1：新建表（推薦）
```sql
-- 網站表
CREATE TABLE sites (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL,
  url TEXT,
  category_id INTEGER,
  tags TEXT,
  notes TEXT,
  created_at TIMESTAMP,
  updated_at TIMESTAMP
);

-- 賬號表
CREATE TABLE site_accounts (
  id INTEGER PRIMARY KEY,
  site_id INTEGER NOT NULL,
  username TEXT NOT NULL,
  password TEXT NOT NULL,
  api_key TEXT,
  secret_key TEXT,
  created_at TIMESTAMP,
  FOREIGN KEY (site_id) REFERENCES sites(id)
);
```

#### 方案2：擴展現有表
在現有credential表中添加字段：
```sql
ALTER TABLE credentials ADD COLUMN site_id INTEGER;
ALTER TABLE credentials ADD COLUMN is_site BOOLEAN DEFAULT 0;
ALTER TABLE credentials ADD COLUMN account_data JSON; -- 存儲賬號數組
```

### 查詢優化
```sql
-- 查詢某網站的所有賬號
SELECT s.*, sa.username, sa.password
FROM sites s
LEFT JOIN site_accounts sa ON s.id = sa.site_id
WHERE s.id = ?;

-- 搜索所有包含某個網站的賬號
SELECT DISTINCT s.name, s.url
FROM sites s
WHERE s.name LIKE '%github%' OR s.url LIKE '%github%';
```

## 🎨 UI設計特點

1. **兩層結構**
   - 上層：網站基本信息
   - 下層：賬號列表

2. **賬號卡片**
   - 顯示賬號序號
   - 顯示用戶名（隱藏密碼）
   - 編輯/刪除操作按鈕

3. **內聯編輯**
   - 點擊編輯展開表單
   - 保存後自動收起
   - 取消編輯可恢復原值

4. **狀態管理**
   - 編輯賬號時禁用其他操作
   - 至少保留一個賬號
   - 完整的表單驗證

## 📌 未來擴展

### 可能的功能增強
1. **賬號組**
   - 將賬號分組（工作/個人）
   - 組間拖拽排序

2. **批量操作**
   - 批量導入賬號
   - 批量修改密碼

3. **密碼生成**
   - 為新賬號生成強密碼
   - 密碼策略配置

4. **賬號狀態**
   - 標記常用賬號
   - 賬號過期提醒
   - 最後使用時間

5. **搜索過濾**
   - 按網站名稱搜索
   - 按賬號名稱搜索
   - 按標籤過濾

## 🔒 安全考慮

1. **加密存儲**
   - 所有密碼使用DEK加密
   - API密鑰和Secret也需要加密
   - 加密算法：AES-256-GCM

2. **訪問控制**
   - 需要主密碼才能查看敏感信息
   - 複製操作需要確認
   - 自動鎖定機制

3. **審計日誌**
   - 記錄賬號訪問
   - 記錄密碼查看
   - 記錄導出操作

## 🚀 實現步驟

### Phase 1: 界面實現 ✅
- [x] SiteAccountManager 組件
- [x] SiteList 組件
- [x] 國際化支持
- [x] 基礎表單驗證

### Phase 2: 後端集成 ✅
- [x] API接口設計
- [x] 數據庫表結構（sites 和 site_accounts 表）
- [x] 加密/解密邏輯
- [x] Rust 實現：
  - [x] models.rs: Site, SiteDetail, SiteAccount, NewSite, UpdateSite 數據結構
  - [x] db.rs: 數據庫表創建和 CRUD 操作函數
  - [x] commands.rs: Tauri 命令（list_sites, get_site, create_site, update_site, delete_site, search_sites）
  - [x] 自動注冊：build.rs 自動提取並注冊新命令
- [x] 前端 composable: useCredential.ts 已定義 Site 接口和 API 函數
- [x] 國際化：中英文翻譯補充

### Phase 3: 功能完善
- [ ] 列表展示
- [ ] 搜索過濾
- [ ] 導入/導出

### Phase 4: 優化改進
- [ ] 性能優化
- [ ] 體驗優化
- [ ] 安全加固
