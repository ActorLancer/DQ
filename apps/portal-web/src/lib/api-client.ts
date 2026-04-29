import { useAuthStore } from '@/store/useAuthStore'

// API 基础 URL（从环境变量读取）
const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080/api'

// API 错误类
export class ApiError extends Error {
  constructor(
    public status: number,
    public statusText: string,
    public data?: any
  ) {
    super(`API Error: ${status} ${statusText}`)
    this.name = 'ApiError'
  }
}

// API 客户端配置
interface ApiClientConfig {
  headers?: Record<string, string>
  params?: Record<string, any>
  body?: any
  method?: 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH'
}

// API 客户端类
class ApiClient {
  private baseURL: string

  constructor(baseURL: string) {
    this.baseURL = baseURL
  }

  // 构建完整 URL
  private buildURL(endpoint: string, params?: Record<string, any>): string {
    const url = new URL(endpoint, this.baseURL)
    
    if (params) {
      Object.entries(params).forEach(([key, value]) => {
        if (value !== undefined && value !== null) {
          url.searchParams.append(key, String(value))
        }
      })
    }
    
    return url.toString()
  }

  // 获取请求头
  private getHeaders(customHeaders?: Record<string, string>): HeadersInit {
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
      ...customHeaders,
    }

    // 自动添加 Token
    const token = useAuthStore.getState().token
    if (token) {
      headers['Authorization'] = `Bearer ${token}`
    }

    return headers
  }

  // 处理响应
  private async handleResponse<T>(response: Response): Promise<T> {
    // 401 未授权 - 自动登出
    if (response.status === 401) {
      useAuthStore.getState().logout()
      
      // 如果在浏览器环境，跳转到登录页
      if (typeof window !== 'undefined') {
        window.location.href = '/login'
      }
      
      throw new ApiError(401, 'Unauthorized', { message: '登录已过期，请重新登录' })
    }

    // 403 禁止访问
    if (response.status === 403) {
      throw new ApiError(403, 'Forbidden', { message: '没有权限访问此资源' })
    }

    // 其他错误状态
    if (!response.ok) {
      let errorData
      try {
        errorData = await response.json()
      } catch {
        errorData = { message: response.statusText }
      }
      
      throw new ApiError(response.status, response.statusText, errorData)
    }

    // 204 No Content
    if (response.status === 204) {
      return null as T
    }

    // 解析 JSON
    try {
      return await response.json()
    } catch {
      return null as T
    }
  }

  // 通用请求方法
  async request<T>(
    endpoint: string,
    config: ApiClientConfig = {}
  ): Promise<T> {
    const { headers, params, body, method = 'GET' } = config

    const url = this.buildURL(endpoint, params)
    const requestHeaders = this.getHeaders(headers)

    const requestConfig: RequestInit = {
      method,
      headers: requestHeaders,
    }

    // 添加请求体（GET 和 DELETE 不需要）
    if (body && method !== 'GET' && method !== 'DELETE') {
      requestConfig.body = JSON.stringify(body)
    }

    try {
      const response = await fetch(url, requestConfig)
      return await this.handleResponse<T>(response)
    } catch (error) {
      if (error instanceof ApiError) {
        throw error
      }
      
      // 网络错误
      throw new ApiError(0, 'Network Error', {
        message: '网络连接失败，请检查网络设置',
        originalError: error,
      })
    }
  }

  // GET 请求
  async get<T>(endpoint: string, params?: Record<string, any>): Promise<T> {
    return this.request<T>(endpoint, { method: 'GET', params })
  }

  // POST 请求
  async post<T>(endpoint: string, body?: any): Promise<T> {
    return this.request<T>(endpoint, { method: 'POST', body })
  }

  // PUT 请求
  async put<T>(endpoint: string, body?: any): Promise<T> {
    return this.request<T>(endpoint, { method: 'PUT', body })
  }

  // PATCH 请求
  async patch<T>(endpoint: string, body?: any): Promise<T> {
    return this.request<T>(endpoint, { method: 'PATCH', body })
  }

  // DELETE 请求
  async delete<T>(endpoint: string): Promise<T> {
    return this.request<T>(endpoint, { method: 'DELETE' })
  }
}

// 导出单例
export const apiClient = new ApiClient(API_BASE_URL)

// 导出便捷方法
export const api = {
  get: <T>(endpoint: string, params?: Record<string, any>) => 
    apiClient.get<T>(endpoint, params),
  
  post: <T>(endpoint: string, body?: any) => 
    apiClient.post<T>(endpoint, body),
  
  put: <T>(endpoint: string, body?: any) => 
    apiClient.put<T>(endpoint, body),
  
  patch: <T>(endpoint: string, body?: any) => 
    apiClient.patch<T>(endpoint, body),
  
  delete: <T>(endpoint: string) => 
    apiClient.delete<T>(endpoint),
}
