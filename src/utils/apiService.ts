import { post, get } from './request';
import { mockData, mockSSEResponse } from './mockData';

// 后端服务已关闭，开启预览模式
let isPreviewMode = true;

// 后端接口
const API_BASE_URL = 'https://ai.moejue.cn';

// 类型定义
export interface User {
  id: string;
  fingerprint?: string;
  inviteCode?: string;
  username?: string;
  email?: string;
  avatarUrl?: string;
  memberLevel?: number;
  registrationDate?: number;
  lastLoginDate?: number;
  freeCredits?: number;
  paidCredits?: number;
  totalSignInDays?: number;
  lastSignInDate?: string;
  createdAt?: string;
  projects?: Project[];
}

export interface Project {
  id: string;
  projectName: string;
  description: string;
  versionsCount?: number;
  lastVersion?: string;
  updateTime?: number;
  createdAt: string;
  updatedAt: string;
}

export interface ProjectVersion {
  id: string;
  version: string;
  createdAt: string;
  description: string;
  size: number;
  code: string;
}

export interface ApiResponse<T = any> {
  success: boolean;
  message?: string;
  data?: T;
}

export interface RegisterResponse extends ApiResponse {
  data?: {
    user: User;
    token: string;
  };
}

export interface SignInStatus {
  hasSigned: boolean;
  consecutiveDays: number;
  signedDates: Array<{ date: number; month: number; year: number }>;
  nextReward: number;
}

export interface SignInResponse extends ApiResponse {
  data?: {
    creditsAdded: number;
    consecutiveDays: number;
    totalCredits: number;
  };
}

export interface CodeGenerationParams {
  description: string;
  model: string;
  style: string;
  uiLibrary: string;
  jsFramework: string;
  cdnProvider: string;
  deviceType: string;
  browserFingerprint: string;
}

export interface CodeGenerationData {
  code?: string;
  projectId?: string;
  versionId?: string;
  message?: string;
}

export interface ContinueConversationParams {
  projectId: string;
  originalPrompt: string;
  additionalPrompt: string;
  currentCode: string;
  model: string;
}

/**
 * 注册新用户
 * @param fingerprint - 浏览器指纹
 * @param inviteCode - 使用的邀请码
 * @returns API响应
 */
export const registerUser = (fingerprint: string, inviteCode: string): Promise<RegisterResponse> => {
  if (isPreviewMode) {
    return Promise.resolve(mockData.register as RegisterResponse);
  }
  return post('/api/register', { fingerprint, inviteCode });
};

/**
 * 获取当前用户信息
 * @returns 用户信息对象
 */
export const getCurrentUser = (): Promise<ApiResponse<{ user: User }>> => {
  if (isPreviewMode) {
    return Promise.resolve(mockData.getCurrentUser as ApiResponse<{ user: User }>);
  }
  return get('/api/me');
};

/**
 * 获取用户签到状态
 * @returns 签到状态信息，包含连续签到天数、已签到日期等
 */
export const getSignInStatus = (): Promise<ApiResponse<{ data: SignInStatus }>> => {
  if (isPreviewMode) {
    return Promise.resolve(mockData.getSignInStatus as ApiResponse<{ data: SignInStatus }>);
  }
  return get('/api/sign-in/status');
};

/**
 * 执行每日签到
 * @returns 签到结果，包含获得的积分和消息等信息
 */
export const signIn = (): Promise<SignInResponse> => {
  if (isPreviewMode) {
    return Promise.resolve(mockData.signIn as SignInResponse);
  }
  return post('/api/sign-in', {});
};

/**
 * 通过 Server-Sent Events (SSE) 流式生成代码。
 * @param params - 生成参数，包含 description, model, style, uiLibrary, jsFramework, cdnProvider, deviceType, browserFingerprint。
 * @param onData - 处理接收到代码片段的回调函数，接收解析后的JSON数据。
 * @param onComplete - 处理生成完成事件的回调函数，接收解析后的JSON数据（可能包含最终projectId）。
 * @param onError - 处理错误事件或网络错误的回调函数。
 */
export const generateCodeStream = async (
  params: CodeGenerationParams,
  onData: (data: CodeGenerationData) => void,
  onComplete: (data: CodeGenerationData) => void,
  onError: (error: string) => void
): Promise<void> => {
  // 检查预览模式
  if (isPreviewMode) {
    mockSSEResponse('generateCode', onData, onComplete, onError);
    return;
  }

  try {
    const response = await fetch(`${API_BASE_URL}/api/generate`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${localStorage.getItem('auth_token')}`
      },
      body: JSON.stringify(params)
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({ message: '网络请求错误' }));
      throw new Error(errorData.message || `网络请求错误: ${response.status} ${response.statusText}`);
    }

    const reader = response.body!.getReader();
    const decoder = new TextDecoder();
    let buffer = '';

    while (true) {
      const { done, value } = await reader.read();
      if (done) {
        // 正常结束，但如果SSE没有显式发送complete事件，可能需要在这里处理
        // 通常complete事件会处理结束逻辑
        break;
      }

      buffer += decoder.decode(value, { stream: true });
      const lines = buffer.split('\n\n');
      buffer = lines.pop() || ''; // 保留下一次可能不完整的消息

      lines.forEach(line => {
        if (!line) return;

        let event = 'message'; // 默认事件类型
        let data = '';

        const eventMatch = line.match(/^event: (.*)$/m);
        if (eventMatch) {
          event = eventMatch[1].trim();
        }

        const dataMatch = line.match(/^data: (.*)$/m);
        if (dataMatch) {
          data = dataMatch[1].trim();
        } else if (line.startsWith('data:')) { // 处理只有 data: 的情况
          data = line.substring(5).trim();
        }

        if (!data) return; // 没有有效数据

        try {
          const jsonData = JSON.parse(data);
          if (event === 'message' || event === 'data') { // 后端可能只发送 data
            onData(jsonData);
          } else if (event === 'complete') {
            onComplete(jsonData);
          } else if (event === 'error') {
            console.error('SSE Error Event:', jsonData);
            onError(jsonData.error || '未知SSE错误事件');
          }
        } catch (e) {
          console.error('解析SSE数据出错:', e, '原始数据:', data);
          // 如果解析失败，但事件是 error，仍然尝试调用 onError
          if (event === 'error') {
            onError('无法解析的SSE错误事件: ' + data);
          }
          // 可以选择忽略无法解析的非错误事件，或者将其作为普通文本处理
        }
      });
    }
    // 确保最后的 buffer 也被处理（如果需要）
    // 通常不需要，因为 complete 事件会标志结束

  } catch (error) {
    console.error('SSE 连接或处理错误:', error);
    onError((error as Error).message || '连接或处理SSE时发生未知错误');
  }
};

/**
 * 通过 Server-Sent Events (SSE) 流式处理继续对话请求
 * @param params - 参数对象，包含 projectId, originalPrompt, additionalPrompt, currentCode, model
 * @param onData - 处理接收到代码片段的回调函数
 * @param onComplete - 处理生成完成事件的回调函数
 * @param onError - 处理错误事件的回调函数
 */
export const continueConversationStream = async (
  params: ContinueConversationParams,
  onData: (data: CodeGenerationData) => void,
  onComplete: (data: CodeGenerationData) => void,
  onError: (error: string) => void
): Promise<void> => {
  // 检查预览模式
  if (isPreviewMode) {
    mockSSEResponse('continueConversation', onData, onComplete, onError);
    return;
  }

  try {
    const response = await fetch(`${API_BASE_URL}/api/continue-conversation`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${localStorage.getItem('auth_token')}`
      },
      body: JSON.stringify(params)
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({ message: '网络请求错误' }));
      throw new Error(errorData.message || `网络请求错误: ${response.status} ${response.statusText}`);
    }

    const reader = response.body!.getReader();
    const decoder = new TextDecoder();
    let buffer = '';

    while (true) {
      const { done, value } = await reader.read();
      if (done) {
        break;
      }

      buffer += decoder.decode(value, { stream: true });
      const lines = buffer.split('\n\n');
      buffer = lines.pop() || ''; // 保留可能不完整的消息

      lines.forEach(line => {
        if (!line) return;

        let event = 'message'; // 默认事件类型
        let data = '';

        const eventMatch = line.match(/^event: (.*)$/m);
        if (eventMatch) {
          event = eventMatch[1].trim();
        }

        const dataMatch = line.match(/^data: (.*)$/m);
        if (dataMatch) {
          data = dataMatch[1].trim();
        } else if (line.startsWith('data:')) {
          data = line.substring(5).trim();
        }

        if (!data) return;

        try {
          const jsonData = JSON.parse(data);
          if (event === 'message' || event === 'data') {
            onData(jsonData);
          } else if (event === 'complete') {
            onComplete(jsonData);
          } else if (event === 'error') {
            console.error('SSE Error Event:', jsonData);
            onError(jsonData.error || '未知SSE错误事件');
          }
        } catch (e) {
          console.error('解析SSE数据出错:', e, '原始数据:', data);
          if (event === 'error') {
            onError('无法解析的SSE错误事件: ' + data);
          }
        }
      });
    }
  } catch (error) {
    console.error('SSE 连接或处理错误:', error);
    onError((error as Error).message || '连接或处理SSE时发生未知错误');
  }
};

/**
 * 获取用户项目列表
 * @returns 项目列表响应
 */
export const getUserProjects = (): Promise<ApiResponse<{ projects: Project[]; total: number }>> => {
  if (isPreviewMode) {
    return Promise.resolve(mockData.getUserProjects as ApiResponse<{ projects: Project[]; total: number }>);
  }
  return get('/api/projects');
};

/**
 * 获取项目详情及其版本列表
 * @param projectId - 项目ID
 * @returns 项目详情响应
 */
export const getProjectById = (projectId: string): Promise<ApiResponse<{ project: Project & { versions: ProjectVersion[] } }>> => {
  if (isPreviewMode) {
    return Promise.resolve(mockData.getProjectById as ApiResponse<{ project: Project & { versions: ProjectVersion[] } }>);
  }
  return get(`/api/projects/${projectId}`);
};

/**
 * 获取项目版本的HTML代码
 * @param projectId - 项目ID
 * @param versionId - 版本ID
 * @returns HTML代码内容
 */
export const getProjectVersionCode = (projectId: string, versionId: string): Promise<string> => {
  if (isPreviewMode) {
    return Promise.resolve(mockData.getProjectVersionCode);
  }
  return get<string>(
    `/api/projects/${projectId}/versions/${versionId}/html`,
    {},
    { responseType: 'text' }
  );
};

/**
 * 更新项目
 * @param projectId - 项目ID
 * @param projectData - 更新的项目数据
 * @returns 更新结果
 */
export const updateProject = (projectId: string, projectData: Partial<Project>): Promise<ApiResponse<Project>> => {
  if (isPreviewMode) {
    return Promise.resolve(mockData.updateProject as ApiResponse<Project>);
  }
  return post(`/api/projects/${projectId}`, projectData, { method: 'PUT' });
};

/**
 * 删除项目
 * @param projectId - 要删除的项目ID
 * @returns 删除结果
 */
export const deleteProject = (projectId: string): Promise<ApiResponse> => {
  if (isPreviewMode) {
    return Promise.resolve(mockData.deleteProject);
  }
  return post(`/api/projects/${projectId}`, {}, { method: 'DELETE' });
};

/**
 * 获取所有AI模型的积分消耗配置
 * @returns 包含所有模型积分消耗配置的对象
 */
export const getModelCreditCosts = async (): Promise<ApiResponse<Record<string, number>>> => {
  if (isPreviewMode) {
    return Promise.resolve(mockData.getModelCreditCosts as ApiResponse<Record<string, number>>);
  }
  return await get('/api/credit-costs');
};
