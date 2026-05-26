/**
 * 通用工具函数
 */

/**
 * 对用户输入进行消毒处理，防止 XSS
 * - 移除 < > 字符
 * - 去除首尾空白
 * - 限制最大长度
 */
export const sanitize = (v: string): string =>
  v.replace(/[<>]/g, "").trim().slice(0, 256);