/**
 * 时间工具类
 * 提供各种时间格式化和处理功能
 */

/**
 * 格式化时间，包含毫秒
 * @param date 要格式化的日期对象
 * @returns 格式化后的时间字符串，格式为 HH:MM:SS.mmm
 */
export function formatTime(date: Date): string {
  const hours = date.getHours().toString().padStart(2, '0');
  const minutes = date.getMinutes().toString().padStart(2, '0');
  const seconds = date.getSeconds().toString().padStart(2, '0');
  const milliseconds = date.getMilliseconds().toString().padStart(3, '0');
  return `${hours}:${minutes}:${seconds}.${milliseconds}`;
}

/**
 * 获取当前时间的格式化字符串（包含毫秒）
 * @returns 格式化后的当前时间字符串
 */
export function getCurrentFormattedTime(): string {
  return formatTime(new Date());
}
