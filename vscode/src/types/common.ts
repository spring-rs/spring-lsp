/**
 * 公共类型定义
 * 
 * 包含在 VSCode 扩展和语言服务器之间共享的数据结构
 */

/**
 * 位置信息
 * 
 * 表示源代码中的一个位置范围
 */
export interface Location {
  /**
   * 文件 URI
   */
  uri: string;

  /**
   * 位置范围
   */
  range: Range;
}

/**
 * 位置范围
 */
export interface Range {
  /**
   * 起始位置
   */
  start: Position;

  /**
   * 结束位置
   */
  end: Position;
}

/**
 * 位置坐标
 */
export interface Position {
  /**
   * 行号（从 0 开始）
   */
  line: number;

  /**
   * 列号（从 0 开始）
   */
  character: number;
}

/**
 * 组件作用域
 */
export enum ComponentScope {
  Singleton = 'Singleton',
  Prototype = 'Prototype'
}

/**
 * 任务类型
 */
export enum JobType {
  Cron = 'Cron',
  FixDelay = 'FixDelay',
  FixRate = 'FixRate'
}

/**
 * HTTP 方法
 */
export enum HttpMethod {
  GET = 'GET',
  POST = 'POST',
  PUT = 'PUT',
  PATCH = 'PATCH',
  DELETE = 'DELETE',
  HEAD = 'HEAD',
  OPTIONS = 'OPTIONS'
}

/**
 * 数据来源
 */
export enum DataSource {
  /**
   * 静态分析（通过解析代码获取）
   */
  Static = 'static',

  /**
   * 运行时（从运行中的应用获取）
   */
  Runtime = 'runtime'
}
