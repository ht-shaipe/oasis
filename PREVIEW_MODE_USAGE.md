# 预览模式使用说明

## 功能概述

预览模式是一个开发调试功能，允许开发者在没有后端 API 的情况下测试前端功能。当预览模式开启时，所有 API 请求都会返回模拟数据，而不会发送实际的网络请求。

## 如何开启/关闭预览模式

### 方法1：直接修改代码

在 `src/utils/apiService.ts` 文件中，修改 `isPreviewMode` 变量：

```typescript
let isPreviewMode = true;   // 开启预览模式
// let isPreviewMode = false; // 关闭预览模式
```

### 方法2：通过浏览器控制台

在浏览器控制台中执行以下代码：

```javascript
window.isPreviewMode = true;   // 开启预览模式
window.isPreviewMode = false;  // 关闭预览模式
```

## 模拟数据说明

### 签到相关 API
- `getSignInStatus`: 返回签到状态，包含连续签到天数、已签到日期等
- `signIn`: 返回签到结果，包含获得的积分

### 用户相关 API
- `registerUser`: 返回用户注册数据
- `getCurrentUser`: 返回用户信息

### 项目相关 API
- `getUserProjects`: 返回用户项目列表
- `getProjectById`: 返回项目详情
- `getProjectVersionCode`: 返回项目版本代码
- `updateProject`: 返回项目更新结果
- `deleteProject`: 返回项目删除结果

### 流式 API
- `generateCodeStream`: 模拟代码生成流式响应
- `continueConversationStream`: 模拟对话继续流式响应

## 文件结构

- `src/utils/apiService.ts`: 主要的 API 服务文件，包含预览模式开关
- `src/utils/mockData.ts`: 模拟数据文件，包含所有 API 的模拟数据

## 注意事项

1. 预览模式仅用于开发调试，不应在生产环境中使用
2. 模拟数据可能与真实 API 响应存在差异
3. 建议定期更新模拟数据以保持与真实 API 的一致性
4. 流式 API 的模拟延迟可能与真实情况不同
