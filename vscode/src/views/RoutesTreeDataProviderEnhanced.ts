import * as vscode from 'vscode';
import { SpringApp } from '../models';
import { LanguageClientManager } from '../languageClient';
import { Route, RoutesResponse, DataSource } from '../types';
import { ViewMode, VIEW_MODE_KEYS } from '../types/viewMode';
import { FileTreeNode, createFileTreeNodes } from './BaseTreeDataProvider';

/**
 * 增强版 Routes 树视图数据提供者
 * 
 * 支持两种视图模式：
 * - List 模式：按 HTTP 方法分组显示
 * - Tree 模式：按文件组织显示
 */
export class RoutesTreeDataProviderEnhanced
  implements vscode.TreeDataProvider<TreeNode>
{
  private _onDidChangeTreeData = new vscode.EventEmitter<TreeNode | undefined>();
  readonly onDidChangeTreeData = this._onDidChangeTreeData.event;

  private staticRoutes: Route[] = [];
  private runtimeRoutes: Route[] = [];
  private currentApp: SpringApp | undefined;
  private viewMode: ViewMode = ViewMode.List;

  constructor(
    private readonly clientManager: LanguageClientManager,
    private readonly context: vscode.ExtensionContext
  ) {
    this.loadViewMode();

    vscode.workspace.onDidChangeConfiguration(e => {
      if (e.affectsConfiguration(VIEW_MODE_KEYS.routes)) {
        this.loadViewMode();
        this._onDidChangeTreeData.fire(undefined);
      }
    });

    vscode.workspace.onDidSaveTextDocument(doc => {
      if (doc.languageId === 'rust') {
        this.refreshStatic();
      }
    });

    vscode.workspace.onDidChangeWorkspaceFolders(() => {
      this.refreshStatic();
    });

    this.refreshStatic();
  }

  private loadViewMode(): void {
    const config = vscode.workspace.getConfiguration();
    const mode = config.get<string>(VIEW_MODE_KEYS.routes, ViewMode.List);
    this.viewMode = mode as ViewMode;
    console.log(`[RoutesTreeDataProvider] View mode: ${this.viewMode}`);
  }

  /**
   * 选择视图模式（通过快速选择）
   */
  public async selectViewMode(): Promise<void> {
    const items: vscode.QuickPickItem[] = [
      {
        label: '$(list-flat) List',
        description: 'Group by HTTP method',
        detail: 'Show routes grouped by HTTP method (GET, POST, etc.)',
        picked: this.viewMode === ViewMode.List
      },
      {
        label: '$(list-tree) Tree',
        description: 'Group by file',
        detail: 'Organize routes by file structure',
        picked: this.viewMode === ViewMode.Tree
      }
    ];

    const selected = await vscode.window.showQuickPick(items, {
      placeHolder: 'Select view mode for Routes',
      title: 'Routes View Mode'
    });

    if (!selected) {
      return;
    }

    const newMode = selected.label.includes('List') ? ViewMode.List : ViewMode.Tree;
    
    if (newMode !== this.viewMode) {
      await vscode.workspace.getConfiguration().update(
        VIEW_MODE_KEYS.routes,
        newMode,
        vscode.ConfigurationTarget.Workspace
      );
      this.viewMode = newMode;
      this._onDidChangeTreeData.fire(undefined);
      
      vscode.window.showInformationMessage(
        `Routes view: ${newMode === ViewMode.List ? 'List' : 'Tree'} mode`
      );
    }
  }

  /**
   * 切换视图模式（快速切换）
   */
  public async toggleViewMode(): Promise<void> {
    const newMode = this.viewMode === ViewMode.List ? ViewMode.Tree : ViewMode.List;
    await vscode.workspace.getConfiguration().update(
      VIEW_MODE_KEYS.routes,
      newMode,
      vscode.ConfigurationTarget.Workspace
    );
    this.viewMode = newMode;
    this._onDidChangeTreeData.fire(undefined);
    
    vscode.window.showInformationMessage(
      `Routes view: ${newMode === ViewMode.List ? 'List' : 'Tree'} mode`
    );
  }

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

  private async refreshStaticByPath(appPath: string): Promise<void> {
    try {
      const response = await this.clientManager.sendRequest<RoutesResponse>(
        'spring/routes',
        { appPath }
      );

      this.staticRoutes = response?.routes || [];
      console.log(`[RoutesTreeDataProvider] Loaded ${this.staticRoutes.length} routes`);
      this._onDidChangeTreeData.fire(undefined);
    } catch (error) {
      console.error('Failed to load routes:', error);
      this.staticRoutes = [];
      this._onDidChangeTreeData.fire(undefined);
    }
  }

  public async refresh(app?: SpringApp): Promise<void> {
    if (!app) {
      this.clearRuntime();
      return;
    }

    this.currentApp = app;
    await this.refreshStaticByPath(app.path);

    if (app.state === 'running') {
      await this.refreshRuntime(app);
    }
  }

  private async refreshRuntime(app: SpringApp): Promise<void> {
    if (!app.port) {
      return;
    }

    try {
      const response = await fetch(`http://localhost:${app.port}/_debug/routes`);
      if (response.ok) {
        const data = await response.json() as { routes?: Route[] };
        this.runtimeRoutes = data.routes || [];
        this._onDidChangeTreeData.fire(undefined);
      }
    } catch (error) {
      console.warn('Failed to load runtime routes:', error);
    }
  }

  private clearRuntime(): void {
    this.runtimeRoutes = [];
    this.currentApp = undefined;
    this._onDidChangeTreeData.fire(undefined);
  }

  public getTreeItem(element: TreeNode): vscode.TreeItem {
    return element;
  }

  public async getChildren(element?: TreeNode): Promise<TreeNode[]> {
    const routes = this.runtimeRoutes.length > 0
      ? this.runtimeRoutes
      : this.staticRoutes;

    const source = this.runtimeRoutes.length > 0
      ? DataSource.Runtime
      : DataSource.Static;

    if (routes.length === 0) {
      return [];
    }

    // 根节点
    if (!element) {
      if (this.viewMode === ViewMode.Tree) {
        return createFileTreeNodes(routes, 'Routes');
      } else {
        return this.getMethodGroupNodes(routes, source);
      }
    }

    // 文件节点的子节点
    if (element instanceof FileTreeNode) {
      return element.items.map(
        route => new RouteTreeNode(route as Route, source, this.context)
      );
    }

    // 方法分组节点的子节点
    if (element instanceof MethodGroupNode) {
      return element.routes.map(
        route => new RouteTreeNode(route, source, this.context)
      );
    }

    return [];
  }

  /**
   * 获取按方法分组的节点（List 模式）
   */
  private getMethodGroupNodes(routes: Route[], source: DataSource): TreeNode[] {
    const methodMap = new Map<string, Route[]>();
    
    for (const route of routes) {
      const method = route.method || 'UNKNOWN';
      if (!methodMap.has(method)) {
        methodMap.set(method, []);
      }
      methodMap.get(method)!.push(route);
    }

    const methodNodes: MethodGroupNode[] = [];
    const methodOrder = ['GET', 'POST', 'PUT', 'PATCH', 'DELETE', 'HEAD', 'OPTIONS'];
    
    for (const method of methodOrder) {
      if (methodMap.has(method)) {
        methodNodes.push(new MethodGroupNode(method, methodMap.get(method)!));
      }
    }

    // 添加其他方法
    for (const [method, methodRoutes] of methodMap.entries()) {
      if (!methodOrder.includes(method)) {
        methodNodes.push(new MethodGroupNode(method, methodRoutes));
      }
    }

    return methodNodes;
  }
}

type TreeNode = FileTreeNode | MethodGroupNode | RouteTreeNode;

/**
 * HTTP 方法分组节点
 */
class MethodGroupNode extends vscode.TreeItem {
  constructor(
    public readonly method: string,
    public readonly routes: Route[]
  ) {
    super(method, vscode.TreeItemCollapsibleState.Expanded);

    this.description = `${routes.length} route${routes.length > 1 ? 's' : ''}`;
    this.contextValue = 'spring:method-group';
    
    // 设置图标和颜色
    const iconMap: Record<string, { icon: string; color: string }> = {
      'GET': { icon: 'arrow-down', color: 'charts.blue' },
      'POST': { icon: 'add', color: 'charts.green' },
      'PUT': { icon: 'edit', color: 'charts.yellow' },
      'PATCH': { icon: 'diff-modified', color: 'charts.orange' },
      'DELETE': { icon: 'trash', color: 'charts.red' },
      'HEAD': { icon: 'info', color: 'charts.purple' },
      'OPTIONS': { icon: 'settings-gear', color: 'charts.foreground' },
    };

    const iconInfo = iconMap[method] || { icon: 'symbol-method', color: 'charts.foreground' };
    this.iconPath = new vscode.ThemeIcon(iconInfo.icon, new vscode.ThemeColor(iconInfo.color));
  }
}

/**
 * 路由树节点
 */
class RouteTreeNode extends vscode.TreeItem {
  constructor(
    public readonly route: Route,
    private readonly source: DataSource,
    private readonly context: vscode.ExtensionContext
  ) {
    super(route.path, vscode.TreeItemCollapsibleState.None);

    this.contextValue = `spring:route-${source}`;
    this.description = route.handler || '';
    this.tooltip = this.buildTooltip();
    this.iconPath = this.getIcon();

    if (route.location) {
      this.command = {
        command: 'spring.route.navigate',
        title: 'Go to Handler',
        arguments: [route.location],
      };
    }
  }

  private buildTooltip(): vscode.MarkdownString {
    const tooltip = new vscode.MarkdownString();
    tooltip.isTrusted = true;

    tooltip.appendMarkdown(`### ${this.route.method} ${this.route.path}\n\n`);
    
    if (this.route.handler) {
      tooltip.appendMarkdown(`**Handler:** \`${this.route.handler}\`\n\n`);
    }

    if (this.route.isOpenapi) {
      tooltip.appendMarkdown(`**Type:** OpenAPI Route 📖\n\n`);
    }

    if (this.source === DataSource.Runtime) {
      tooltip.appendMarkdown('✅ **Runtime Information**\n\n');
    } else {
      tooltip.appendMarkdown('📝 **Static Analysis**\n\n');
    }

    tooltip.appendMarkdown(`\n*Click to go to handler*`);

    return tooltip;
  }

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
}
