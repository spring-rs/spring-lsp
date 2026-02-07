import * as vscode from 'vscode';
import { SpringApp, AppState } from '../models';
import { LocalAppManager } from '../controllers';

/**
 * Apps 树视图数据提供者
 * 
 * 负责显示工作空间中的 Spring RS 应用列表，并支持选择当前要查看的应用
 */
export class AppsTreeDataProvider implements vscode.TreeDataProvider<TreeItem> {
  /**
   * 树数据变化事件发射器
   */
  private _onDidChangeTreeData = new vscode.EventEmitter<TreeItem | undefined>();

  /**
   * 树数据变化事件
   */
  readonly onDidChangeTreeData: vscode.Event<TreeItem | undefined> =
    this._onDidChangeTreeData.event;

  /**
   * 应用选择事件发射器
   */
  private _onDidSelectApp = new vscode.EventEmitter<SpringApp>();

  /**
   * 应用选择事件
   */
  readonly onDidSelectApp: vscode.Event<SpringApp> = this._onDidSelectApp.event;

  /**
   * 应用管理器
   */
  private readonly appManager: LocalAppManager;

  /**
   * 当前选中的应用
   */
  private selectedApp: SpringApp | undefined;

  /**
   * 创建 AppsTreeDataProvider 实例
   * 
   * @param appManager 应用管理器
   */
  constructor(appManager: LocalAppManager) {
    this.appManager = appManager;

    // 监听应用列表变化
    this.appManager.onDidChangeApps(() => {
      // 如果当前选中的应用不在列表中，清除选择
      const apps = appManager.getAppList();
      if (this.selectedApp && !apps.find(a => a.path === this.selectedApp!.path)) {
        this.selectedApp = undefined;
      }
      
      // 如果没有选中应用且有可用应用，自动选择第一个
      if (!this.selectedApp && apps.length > 0) {
        this.selectApp(apps[0]);
      }
      
      this.refresh();
    });

    // 初始化：自动选择第一个应用
    const apps = appManager.getAppList();
    if (apps.length > 0) {
      this.selectApp(apps[0]);
    }
  }

  /**
   * 选择应用
   * 
   * @param app 要选择的应用
   */
  public selectApp(app: SpringApp): void {
    if (this.selectedApp === app) {
      return;
    }

    this.selectedApp = app;
    this.refresh();
    this._onDidSelectApp.fire(app);
    
    console.log(`Selected app: ${app.name} (${app.path})`);
  }

  /**
   * 获取当前选中的应用
   * 
   * @returns 当前选中的应用，如果没有选中则返回 undefined
   */
  public getSelectedApp(): SpringApp | undefined {
    return this.selectedApp;
  }

  /**
   * 刷新树视图
   */
  public refresh(): void {
    this._onDidChangeTreeData.fire(undefined);
  }

  /**
   * 获取树节点
   * 
   * @param element 树节点元素
   * @returns 树节点
   */
  public getTreeItem(element: TreeItem): vscode.TreeItem {
    return element;
  }

  /**
   * 获取子节点
   * 
   * @param element 父节点，如果为 undefined 表示根节点
   * @returns 子节点列表
   */
  public async getChildren(element?: TreeItem): Promise<TreeItem[]> {
    if (!element) {
      // 根节点：显示所有应用
      const apps = this.appManager.getAppList();
      return apps.map((app) => new AppTreeItem(app, app === this.selectedApp));
    }

    if (element instanceof AppTreeItem) {
      // 应用节点的子节点：显示详细信息
      const app = element.app;
      const children: TreeItem[] = [];

      // 路径
      children.push(new InfoTreeItem('Path', app.path, '📁'));

      // 版本
      children.push(new InfoTreeItem('Version', app.version, '🏷️'));

      // 状态
      children.push(
        new InfoTreeItem('State', app.state, this.getStateIcon(app.state))
      );

      // 端口（仅在运行时显示）
      if (app.port) {
        children.push(new InfoTreeItem('Port', app.port.toString(), '🌐'));
      }

      // Profile（如果有）
      if (app.profile) {
        children.push(new InfoTreeItem('Profile', app.profile, '⚙️'));
      }

      // PID（如果有）
      if (app.pid) {
        children.push(new InfoTreeItem('PID', app.pid.toString(), '🔢'));
      }

      return children;
    }

    return [];
  }

  /**
   * 获取状态图标
   * 
   * @param state 应用状态
   * @returns 图标字符
   */
  private getStateIcon(state: AppState): string {
    switch (state) {
      case AppState.INACTIVE:
        return '⚪';
      case AppState.LAUNCHING:
        return '🟡';
      case AppState.RUNNING:
        return '🟢';
      case AppState.STOPPING:
        return '🟠';
      default:
        return '⚪';
    }
  }
}

/**
 * 树节点基类
 */
export type TreeItem = AppTreeItem | InfoTreeItem;

/**
 * 应用树节点
 */
export class AppTreeItem extends vscode.TreeItem {
  /**
   * 应用实例
   */
  public readonly app: SpringApp;

  /**
   * 是否为当前选中的应用
   */
  public readonly isSelected: boolean;

  /**
   * 创建应用树节点
   * 
   * @param app 应用实例
   * @param isSelected 是否为当前选中的应用
   */
  constructor(app: SpringApp, isSelected: boolean = false) {
    super(app.getDisplayName(), vscode.TreeItemCollapsibleState.Collapsed);

    this.app = app;
    this.isSelected = isSelected;

    // 设置上下文值（用于命令菜单）
    this.contextValue = isSelected 
      ? `SpringApp_${app.state}_selected` 
      : `SpringApp_${app.state}`;

    // 设置工具提示
    this.tooltip = this.buildTooltip();

    // 设置描述
    this.description = this.getDescription();

    // 设置图标
    this.iconPath = this.getIcon();

    // 设置复选框状态
    this.checkboxState = isSelected 
      ? vscode.TreeItemCheckboxState.Checked 
      : vscode.TreeItemCheckboxState.Unchecked;

    // 设置点击命令（打开配置文件）
    this.command = {
      command: 'vscode.open',
      title: 'Open Config',
      arguments: [vscode.Uri.file(`${app.path}/config/app.toml`)],
    };
  }

  /**
   * 构建工具提示
   */
  private buildTooltip(): vscode.MarkdownString {
    const tooltip = new vscode.MarkdownString();
    tooltip.isTrusted = true;

    tooltip.appendMarkdown(`### ${this.app.name}\n\n`);
    
    if (this.isSelected) {
      tooltip.appendMarkdown(`✅ **Currently Selected**\n\n`);
      tooltip.appendMarkdown(`_All views are showing information from this application_\n\n`);
    }
    
    tooltip.appendMarkdown(`**Path:** ${this.app.path}\n\n`);
    tooltip.appendMarkdown(`**Version:** ${this.app.version}\n\n`);
    tooltip.appendMarkdown(`**State:** ${this.app.state}\n\n`);

    if (this.app.port) {
      tooltip.appendMarkdown(`**Port:** ${this.app.port}\n\n`);
    }

    if (this.app.profile) {
      tooltip.appendMarkdown(`**Profile:** ${this.app.profile}\n\n`);
    }

    // 添加依赖信息
    if (this.app.dependencies.length > 0) {
      tooltip.appendMarkdown(`**Dependencies:**\n`);
      const springDeps = this.app.dependencies.filter((dep) =>
        dep.startsWith('spring')
      );
      springDeps.forEach((dep) => {
        tooltip.appendMarkdown(`- ${dep}\n`);
      });
    }

    if (!this.isSelected) {
      tooltip.appendMarkdown(`\n*Check the checkbox to select this application*`);
    }

    return tooltip;
  }

  /**
   * 获取描述
   */
  private getDescription(): string {
    const parts: string[] = [];
    
    // 状态
    parts.push(this.app.state);
    
    // 端口
    if (this.app.port) {
      parts.push(`:${this.app.port}`);
    }
    
    // Profile
    if (this.app.profile) {
      parts.push(`[${this.app.profile}]`);
    }
    
    // 选中标识
    if (this.isSelected) {
      parts.push('(current)');
    }
    
    return parts.join(' ');
  }

  /**
   * 获取图标
   */
  private getIcon(): vscode.ThemeIcon {
    let iconId: string;
    let color: vscode.ThemeColor | undefined;

    switch (this.app.state) {
      case AppState.RUNNING:
        iconId = 'debug-start';
        color = new vscode.ThemeColor('testing.iconPassed');
        break;
      case AppState.LAUNCHING:
        iconId = 'loading~spin';
        color = new vscode.ThemeColor('testing.iconQueued');
        break;
      case AppState.STOPPING:
        iconId = 'loading~spin';
        color = new vscode.ThemeColor('testing.iconErrored');
        break;
      case AppState.INACTIVE:
      default:
        iconId = 'circle-outline';
        break;
    }

    return new vscode.ThemeIcon(iconId, color);
  }
}

/**
 * 信息树节点
 */
export class InfoTreeItem extends vscode.TreeItem {
  /**
   * 创建信息树节点
   * 
   * @param label 标签
   * @param value 值
   * @param icon 图标（可选）
   */
  constructor(label: string, value: string, icon?: string) {
    super(`${label}: ${value}`, vscode.TreeItemCollapsibleState.None);

    // 设置图标
    if (icon) {
      this.iconPath = new vscode.ThemeIcon('symbol-string');
      this.description = icon;
    }

    // 不可选择
    this.contextValue = 'info';
  }
}
