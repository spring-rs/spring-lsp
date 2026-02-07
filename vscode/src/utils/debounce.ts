/**
 * 防抖函数
 * 
 * 延迟执行函数，如果在延迟期间再次调用，则重新计时
 * 
 * @param func 要防抖的函数
 * @param wait 延迟时间（毫秒）
 * @returns 防抖后的函数
 */
export function debounce<T extends (...args: any[]) => any>(
  func: T,
  wait: number
): (...args: Parameters<T>) => void {
  let timeout: NodeJS.Timeout | undefined;

  return function (this: any, ...args: Parameters<T>) {
    const context = this;

    const later = () => {
      timeout = undefined;
      func.apply(context, args);
    };

    if (timeout) {
      clearTimeout(timeout);
    }

    timeout = setTimeout(later, wait);
  };
}

/**
 * 创建一个可取消的防抖函数
 * 
 * @param func 要防抖的函数
 * @param wait 延迟时间（毫秒）
 * @returns 包含防抖函数和取消函数的对象
 */
export function debounceCancelable<T extends (...args: any[]) => any>(
  func: T,
  wait: number
): {
  debounced: (...args: Parameters<T>) => void;
  cancel: () => void;
} {
  let timeout: NodeJS.Timeout | undefined;

  const debounced = function (this: any, ...args: Parameters<T>) {
    const context = this;

    const later = () => {
      timeout = undefined;
      func.apply(context, args);
    };

    if (timeout) {
      clearTimeout(timeout);
    }

    timeout = setTimeout(later, wait);
  };

  const cancel = () => {
    if (timeout) {
      clearTimeout(timeout);
      timeout = undefined;
    }
  };

  return { debounced, cancel };
}
