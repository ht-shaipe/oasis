# 网站账号管理器设计文档

## 概述

为了解决"一个网站下有多套账号密码"的常见需求，设计了**网站账号管理器**组件。

## 使用场景

```
网站: GitHub (https://github.com)
├─ 账号1: user1@example.com / password1
├─ 账号2: user2@example.com / password2  
└─ 账号3: user3@example.com / password3

网站: Google (https://google.com)
├─ 账号1: personal@gmail.com / personal_pwd
└─ 账号2: work@company.com / work_pwd
```

## 数据结构设计

### 推荐方案：主从结构

```
Site (网站主记录)
├─ id: 数字
├─ name: "GitHub"
├─ url: "https://github.com"
├─ category_id: 分类ID
├─ tags: "开发,代码"
├─ notes: "主要用于代码托管"
└─ accounts: Account[] (账号列表)

Account (账号子记录)  
├─ username: "user1@example.com"
├─ password: "encrypted_password"
├─ api_key: "可选的API密钥"
└─ secret_key: "可选的Secret"
```

优点：
- 结构清晰：网站信息与账号信息分离
- 易于管理：可以独立添加/编辑/删除账号
- 查询高效：按网站分组，方便查找
- 灵活扩展：未来可以添加更多账号属性

## 功能特性

### 核心功能
1. **网站管理** — 添加/编辑网站信息（名称、网址、分类、标签、备注）
2. **账号管理** — 一个网站下可添加多个账号，每个账号独立管理用户名和密码
3. **账号操作** — 添加/编辑/删除账号（至少保留一个）
4. **数据验证** — 网站名称必填、分类必选、至少一个账号、用户名和密码必填

## 组件说明

### SiteAccountManager.vue

#### Props
```typescript
{
  modelValue: boolean,        // 对话框显示状态
  categories: Array<{         // 分类列表
    id: number,
    name: string,
    level: number
  }>,
  editingSite?: any,         // 编辑的网站数据（可选）
  dek?: string | null        // 加密密钥（可选）
}
```

#### Emits
```typescript
{
  'update:modelValue': (val: boolean) => void,
  'saved': (data: any) => void
}
```

## 数据存储

### 数据库表结构

```sql
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

## UI 设计特点

1. **两层结构** — 上层：网站基本信息，下层：账号列表
2. **账号卡片** — 显示账号序号、用户名（隐藏密码）、编辑/删除操作按钮
3. **内联编辑** — 点击编辑展开表单，保存后自动收起，取消可恢复原值
4. **状态管理** — 编辑账号时禁用其他操作，至少保留一个账号，完整表单验证

## 安全考虑

1. **加密存储** — 所有密码使用 DEK 加密，API 密钥和 Secret 也需加密，算法：AES-256-GCM
2. **访问控制** — 需要主密码才能查看敏感信息，复制操作需确认，自动锁定机制
3. **审计日志** — 记录账号访问、密码查看、导出操作

## 实现进度

### Phase 1: 界面实现 ✅
- [x] SiteAccountManager 组件
- [x] SiteList 组件
- [x] 国际化支持
- [x] 基础表单验证

### Phase 2: 后端集成 ✅
- [x] API 接口设计
- [x] 数据库表结构（sites 和 site_accounts 表）
- [x] 加密/解密逻辑
- [x] Rust 实现（models/db/commands）
- [x] 自动命令注册（build.rs）
- [x] 前端 composable: useCredential.ts
- [x] 国际化：中英文翻译

### Phase 3: 功能完善
- [ ] 列表展示
- [ ] 搜索过滤
- [ ] 导入/导出

### Phase 4: 优化改进
- [ ] 性能优化
- [ ] 体验优化
- [ ] 安全加固
