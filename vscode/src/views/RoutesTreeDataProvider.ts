import * as vscode from 'vscode';
import { SpringApp } from '../models';
import { LanguageClientManager } from '../languageClient';
import { Route, RoutesResponse, DataSource } from '../types';
import { navigateToLocation } from '../utils';

/**
 * Routes 树视图数据提供者
 * 
 * 负责显示运行中应用的路由列表，按 HTTP 方法分组
 */
export class RoutesTreeDataProvider
  implements vscode.TreeDataProvider<RouteTreeItem>
{
  /**
   * 树数据变化事件发射器
   */
  private _onDidChangeTreeData = new vscode.EventEmitter<
    RouteTreeItem | undefined
  >();

  /**
   * 树数据变化事件
   */
  readonly onDidChangeTreeData: vscode.Event<RouteTreeItem | undefined> =
    this._onDidChangeTreeData.event;

  /**
   * 静态分析的路由列表
   */
  private staticRoutes: Route[] = [];

  /**
   * 运行时的路由列表
   */
  private runtimeRoutes: Route[] = [];

  /**
   * 当前选中的应用
   */
  private currentApp: SpringApp | undefined;

  /**
   * 语言客户端管理器
   */
  private readonly clientManager: LanguageClientManager;

  /**
   * 扩展上下文（用于获取资源路径）
   */
  private readonly context: vscode.ExtensionContext;

  /**
   * 创建 RoutesTreeDataProvider 实例
   * 
   * @param clientManager 语言客户端管理器
   * @param context 扩展上下文
   */
  constructor(clientManager: LanguageClientManager, context: vscode.ExtensionContext) {
    this.clientManager = clientManager;
    this.context = context;

    // 监听文档保存，触发静态分析
    vscode.workspace.onDidSaveTextDocument(doc => {
      if (doc.languageId === 'rust') {
        this.refreshStatic();
      }
    });

    // 监听工作空间变化
    vscode.workspace.onDidChangeWorkspaceFolders(() => {
      this.refreshStatic();
    });
  }

  /**
   * 刷新静态分析结果（基于工作空间）
   */
  public async refreshStatic(): Promise<void> {
    const workspaceFolders = vscode.workspace.workspaceFolders;
    if (!workspaceFolders || workspaceFolders.length === 0) {
      this.staticRoutes = [];
      this._onDidChangeTreeData.fire(undefined);
      return;
    }

    const workspacePath = workspaceFolders[0].uri.fsPath;
    await this.refreshStaticByPath(workspacePath);
  }

  /**
   * 刷新静态分析结果（基于指定路径）
   */
  private async refreshStaticByPath(appPath: string): Promise<void> {
    try {
      const response = await this.clientManager.sendRequest<RoutesResponse>(
        'spring/routes',
        { appPath }
      );

      this.staticRoutes = response?.routes || [];
      console.log(`Loaded ${this.staticRoutes.length} routes from static analysis (${appPath})`);
      this._onDidChangeTreeData.fire(undefined);
    } catch (error) {
      console.error('Failed to load static routes:', error);
      this.staticRoutes = [];
      this._onDidChangeTreeData.fire(undefined);
    }
  }

  /**
   * 刷新路由列表（兼容旧接口）
   * 
   * @param app 要刷新的应用（可选）
   */
  public async refresh(app?: SpringApp): Promise<void> {
    if (!app) {
      this.clearRuntime();
      return;
    }

    this.currentApp = app;

    // 先刷新静态分析（基于应用路径）
    await this.refreshStaticByPath(app.path);

    // 如果应用在运行，再刷新运行时信息
    if (app.state === 'running') {
      await this.refreshRuntime(app);
    }
  }

  /**
   * 刷新运行时信息
   */
  private async refreshRuntime(app: SpringApp): Promise<void> {
    if (!app.port) {
      console.warn('App is running but port is not available');
      return;
    }

    try {
      const response = await fetch(`http://localhost:${app.port}/_debug/routes`);
      if (response.ok) {
        const data = await response.json() as { routes?: Route[] };
        this.runtimeRoutes = data.routes || [];
        console.log(`Loaded ${this.runtimeRoutes.length} routes from runtime`);
        this._onDidChangeTreeData.fire(undefined);
      }
    } catch (error) {
      console.warn('Failed to load runtime routes:', error);
    }
  }

  /**
   * 清除运行时信息
   */
  private clearRuntime(): void {
    this.runtimeRoutes = [];
    this.currentApp = undefined;
    this._onDidChangeTreeData.fire(undefined);
  }

  /**
   * 获取树节点
   * 
   * @param element 树节点元素
   * @returns 树节点
   */
  public getTreeItem(element: RouteTreeItem): vscode.TreeItem {
    return element;
  }

  /**
   * 获取子节点
   * 
   * @param element 父节点，如果为 undefined 表示根节点
   * @returns 子节点列表
   */
  public async getChildren(element?: RouteTreeItem): Promise<RouteTreeItem[]> {
    // 优先使用运行时信息，否则使用静态分析结果
    const routes = this.runtimeRoutes.length > 0 ? this.runtimeRoutes : this.staticRoutes;
    const app = this.currentApp; // 可能为 undefined（静态模式）

    if (routes.length === 0) {
      // 没有路由
      return [];
    }

    if (!element) {
      // 根节点：按 HTTP 方法分组
      const grouped = this.groupByMethod(routes);
      return Object.entries(grouped).map(
        ([method, routes]) => new MethodGroupItem(method, routes, app, this.context)
      );
    }

    if (element instanceof MethodGroupItem) {
      // 方法分组节点的子节点：显示该方法的所有路由
      return element.routes.map((route) => new RouteItem(route, app, this.context));
    }

    return [];
  }

  /**
   * 按 HTTP 方法分组路由
   * 
   * @param routes 路由列表
   * @returns 按方法分组的路由
   */
  private groupByMethod(routes: Route[]): Record<string, Route[]> {
    const grouped: Record<string, Route[]> = {};

    for (const route of routes) {
      const method = route.method.toUpperCase();
      if (!grouped[method]) {
        grouped[method] = [];
      }
      grouped[method].push(route);
    }

    // 按方法名称排序
    const sortedGrouped: Record<string, Route[]> = {};
    const methodOrder = ['GET', 'POST', 'PUT', 'PATCH', 'DELETE', 'HEAD', 'OPTIONS'];
    
    // 先添加标准方法
    for (const method of methodOrder) {
      if (grouped[method]) {
        sortedGrouped[method] = grouped[method];
      }
    }
    
    // 再添加其他方法
    for (const method of Object.keys(grouped).sort()) {
      if (!methodOrder.includes(method)) {
        sortedGrouped[method] = grouped[method];
      }
    }

    return sortedGrouped;
  }

  /**
   * 获取路由定义位置
   * 
   * @param route 路由实例
   * @returns 位置信息
   */
  public getRouteLocation(route: Route): vscode.Location | undefined {
    if (!route.location) {
      return undefined;
    }

    const uri = vscode.Uri.parse(route.location.uri);
    const range = new vscode.Range(
      route.location.range.start.line,
      route.location.range.start.character,
      route.location.range.end.line,
      route.location.range.end.character
    );

    return new vscode.Location(uri, range);
  }
}

/**
 * 路由树节点基类
 */
export type RouteTreeItem = MethodGroupItem | RouteItem;

/**
 * HTTP 方法分组节点
 */
export class MethodGroupItem extends vscode.TreeItem {
  /**
   * HTTP 方法
   */
  public readonly method: string;

  /**
   * 该方法的所有路由
   */
  public readonly routes: Route[];

  /**
   * 当前应用（可能为 undefined）
   */
  public readonly app: SpringApp | undefined;

  /**
   * 扩展上下文
   */
  private readonly context: vscode.ExtensionContext;

  /**
   * 创建方法分组节点
   * 
   * @param method HTTP 方法
   * @param routes 路由列表
   * @param app 当前应用（可能为 undefined）
   * @param context 扩展上下文
   */
  constructor(method: string, routes: Route[], app: SpringApp | undefined, context: vscode.ExtensionContext) {
    super(method, vscode.TreeItemCollapsibleState.Collapsed);

    this.method = method;
    this.routes = routes;
    this.app = app;
    this.context = context;

    // 设置上下文值
    this.contextValue = 'spring:methodGroup';

    // 设置描述（路由数量）
    this.description = `${routes.length} route${routes.length !== 1 ? 's' : ''}`;

    // 设置图标
    this.iconPath = this.getIcon();

    // 设置工具提示
    this.tooltip = this.buildTooltip();
  }

  /**
   * 构建工具提示
   */
  private buildTooltip(): vscode.MarkdownString {
    const tooltip = new vscode.MarkdownString();
    tooltip.isTrusted = true;

    tooltip.appendMarkdown(`### ${this.method}\n\n`);
    tooltip.appendMarkdown(`**Routes:** ${this.routes.length}\n\n`);

    if (this.routes.length > 0) {
      tooltip.appendMarkdown(`**Paths:**\n`);
      this.routes.slice(0, 5).forEach((route) => {
        tooltip.appendMarkdown(`- ${route.path}\n`);
      });

      if (this.routes.length > 5) {
        tooltip.appendMarkdown(`- ... and ${this.routes.length - 5} more\n`);
      }
    }

    return tooltip;
  }

  /**
   * 获取图标
   */
  private getIcon(): vscode.ThemeIcon {
    // 根据 HTTP 方法选择图标和颜色
    let iconId: string;
    let color: vscode.ThemeColor | undefined;

    switch (this.method) {
      case 'GET':
        iconId = 'arrow-down';
        color = new vscode.ThemeColor('charts.blue');
        break;
      case 'POST':
        iconId = 'add';
        color = new vscode.ThemeColor('charts.green');
        break;
      case 'PUT':
        iconId = 'edit';
        color = new vscode.ThemeColor('charts.yellow');
        break;
      case 'PATCH':
        iconId = 'diff-modified';
        color = new vscode.ThemeColor('charts.orange');
        break;
      case 'DELETE':
        iconId = 'trash';
        color = new vscode.ThemeColor('charts.red');
        break;
      default:
        iconId = 'symbol-method';
        break;
    }

    return new vscode.ThemeIcon(iconId, color);
  }
}

/**
 * 路由节点
 */
export class RouteItem extends vscode.TreeItem {
  /**
   * 路由实例
   */
  public readonly route: Route;

  /**
   * 当前应用（可能为 undefined）
   */
  public readonly app: SpringApp | undefined;

  /**
   * 扩展上下文
   */
  private readonly context: vscode.ExtensionContext;

  /**
   * 创建路由节点
   * 
   * @param route 路由实例
   * @param app 当前应用（可能为 undefined）
   * @param context 扩展上下文
   */
  constructor(route: Route, app: SpringApp | undefined, context: vscode.ExtensionContext) {
    super(route.path, vscode.TreeItemCollapsibleState.None);

    this.route = route;
    this.app = app;
    this.context = context;

    // 设置上下文值（包含方法信息，用于命令菜单）
    this.contextValue = `spring:route+${route.method}`;

    // 设置工具提示
    this.tooltip = this.buildTooltip();

    // 设置描述
    this.description = route.handler;

    // 设置图标（使用 SVG 文件）
    this.iconPath = this.getIcon();

    // 设置点击命令（跳转到处理器）
    if (route.location) {
      this.command = {
        command: 'spring.route.navigate',
        title: 'Go to Handler',
        arguments: [route.location],
      };
    }
  }

  /**
   * 获取图标（使用 SVG 文件，区分 OpenAPI 和普通路由）
   */
  private getIcon(): vscode.Uri {
    // OpenAPI 路由使用特殊图标
    if (this.route.isOpenapi) {
      return vscode.Uri.joinPath(
        this.context.extensionUri,
        'resources',
        'icons',
        'route-openapi.svg'
      );
    }
    // 普通路由使用标准图标
    return vscode.Uri.joinPath(
      this.context.extensionUri,
      'resources',
      'icons',
      'route.svg'
    );
  }

  /**
   * 构建工具提示
   */
  private buildTooltip(): vscode.MarkdownString {
    const tooltip = new vscode.MarkdownString();
    tooltip.isTrusted = true;

    tooltip.appendMarkdown(`### ${this.route.method} ${this.route.path}\n\n`);
    tooltip.appendMarkdown(`**Handler:** \`${this.route.handler}\`\n\n`);
    
    if (this.route.isOpenapi) {
      tooltip.appendMarkdown(`**Type:** OpenAPI Route 📖\n\n`);
    }

    if (this.route.location) {
      tooltip.appendMarkdown(`\n*Click to go to handler*`);
    }

    return tooltip;
  }
}
