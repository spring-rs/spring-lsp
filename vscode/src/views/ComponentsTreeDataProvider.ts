import * as vscode from 'vscode';
import { SpringApp } from '../models';
import { LanguageClientManager } from '../languageClient';
import { Component, ComponentsResponse, DataSource } from '../types';
import { navigateToLocation } from '../utils';

/**
 * Components 树视图数据提供者
 * 
 * 支持静态分析和运行时信息两种模式：
 * - 静态分析：通过解析 Rust 代码获取组件信息（不需要运行应用）
 * - 运行时：从运行中的应用获取实时信息（实例数、内存使用等）
 */
export class ComponentsTreeDataProvider
  implements vscode.TreeDataProvider<ComponentTreeItem | PlaceholderTreeItem>
{
  /**
   * 树数据变化事件发射器
   */
  private _onDidChangeTreeData = new vscode.EventEmitter<
    ComponentTreeItem | PlaceholderTreeItem | undefined
  >();

  /**
   * 树数据变化事件
   */
  readonly onDidChangeTreeData: vscode.Event<ComponentTreeItem | PlaceholderTreeItem | undefined> =
    this._onDidChangeTreeData.event;

  /**
   * 静态分析的组件列表（按名称索引）
   */
  private staticComponents: Map<string, Component> = new Map();

  /**
   * 运行时的组件列表（按名称索引）
   */
  private runtimeComponents: Map<string, Component> = new Map();

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
   * 创建 ComponentsTreeDataProvider 实例
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

    // 初始加载静态分析结果
    this.refreshStatic();
  }

  /**
   * 刷新静态分析结果（基于工作空间）
   */
  public async refreshStatic(): Promise<void> {
    const workspaceFolders = vscode.workspace.workspaceFolders;
    if (!workspaceFolders || workspaceFolders.length === 0) {
      this.staticComponents.clear();
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
      const response = await this.clientManager.sendRequest<ComponentsResponse>(
        'spring/components',
        { appPath }
      );

      this.staticComponents.clear();
      if (response && response.components) {
        response.components.forEach((component) => {
          this.staticComponents.set(component.name, component);
        });
        console.log(`Loaded ${this.staticComponents.size} components from static analysis (${appPath})`);
      }
      this._onDidChangeTreeData.fire(undefined);
    } catch (error) {
      console.error('Failed to load static components:', error);
      this.staticComponents.clear();
      this._onDidChangeTreeData.fire(undefined);
    }
  }

  /**
   * 刷新信息（兼容旧接口）
   * 
   * @param app 要刷新的应用
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
   * 
   * @param app 运行中的应用
   */
  private async refreshRuntime(app: SpringApp): Promise<void> {
    if (!app.port) {
      console.warn('App is running but port is not available');
      return;
    }

    try {
      const response = await fetch(`http://localhost:${app.port}/_debug/components`);
      if (response.ok) {
        const data = await response.json() as { components?: Component[] };
        this.runtimeComponents.clear();
        if (data.components) {
          data.components.forEach((component: Component) => {
            this.runtimeComponents.set(component.name, component);
          });
          console.log(`Loaded ${this.runtimeComponents.size} components from runtime`);
        }
        this._onDidChangeTreeData.fire(undefined);
      }
    } catch (error) {
      console.warn('Failed to load runtime components:', error);
    }
  }

  /**
   * 清除运行时信息
   */
  private clearRuntime(): void {
    this.runtimeComponents.clear();
    this.currentApp = undefined;
    this._onDidChangeTreeData.fire(undefined);
  }

  /**
   * 获取树节点
   * 
   * @param element 树节点元素
   * @returns 树节点
   */
  public getTreeItem(element: ComponentTreeItem | PlaceholderTreeItem): vscode.TreeItem {
    console.log(`[ComponentsTreeDataProvider] getTreeItem called for: ${element.label}`);
    console.log(`[ComponentsTreeDataProvider] collapsibleState: ${element.collapsibleState}`);
    console.log(`[ComponentsTreeDataProvider] has command: ${!!element.command}`);
    return element;
  }

  /**
   * 获取子节点
   * 
   * @param element 父节点，如果为 undefined 表示根节点
   * @returns 子节点列表
   */
  public async getChildren(
    element?: ComponentTreeItem
  ): Promise<(ComponentTreeItem | PlaceholderTreeItem)[]> {
    console.log(`[ComponentsTreeDataProvider] getChildren called, element: ${element ? element.component.name : 'ROOT'}`);
    
    if (!element) {
      // 根节点：显示所有组件
      // 优先使用运行时信息，否则使用静态分析结果
      const components = this.runtimeComponents.size > 0
        ? this.runtimeComponents
        : this.staticComponents;

      const source = this.runtimeComponents.size > 0
        ? DataSource.Runtime
        : DataSource.Static;

      if (components.size === 0) {
        console.log(`[ComponentsTreeDataProvider] No components found`);
        return [];
      }

      console.log(`[ComponentsTreeDataProvider] Returning ${components.size} root components (${source})`);
      return Array.from(components.values()).map(
        (component) => new ComponentTreeItem(component, components, this.context, source)
      );
    }

    // 组件节点的子节点：显示依赖
    console.log(`[ComponentsTreeDataProvider] Element has ${element.component.dependencies.length} dependencies`);
    
    if (element.component.dependencies.length > 0) {
      console.log(`[ComponentsTreeDataProvider] Getting children for ${element.component.name}`);
      console.log(`[ComponentsTreeDataProvider] Dependencies:`, element.component.dependencies);
      
      const components = this.runtimeComponents.size > 0
        ? this.runtimeComponents
        : this.staticComponents;

      const source = this.runtimeComponents.size > 0
        ? DataSource.Runtime
        : DataSource.Static;

      console.log(`[ComponentsTreeDataProvider] Available components:`, Array.from(components.keys()));
      
      const dependencyItems: (ComponentTreeItem | PlaceholderTreeItem)[] = [];
      
      for (const depTypeName of element.component.dependencies) {
        console.log(`[ComponentsTreeDataProvider] Looking for dependency: ${depTypeName}`);
        
        // 尝试通过类型名查找组件
        let depComponent = components.get(depTypeName);
        
        // 如果找不到，遍历所有组件查找匹配的类型名
        if (!depComponent) {
          for (const component of components.values()) {
            if (component.typeName === depTypeName || component.name === depTypeName) {
              depComponent = component;
              console.log(`[ComponentsTreeDataProvider] Found by typeName match: ${component.name}`);
              break;
            }
          }
        } else {
          console.log(`[ComponentsTreeDataProvider] Found by direct match: ${depComponent.name}`);
        }
        
        if (depComponent) {
          dependencyItems.push(new ComponentTreeItem(depComponent, components, this.context, source));
        } else {
          console.log(`[ComponentsTreeDataProvider] Creating placeholder for: ${depTypeName}`);
          dependencyItems.push(new PlaceholderTreeItem(depTypeName));
        }
      }
      
      console.log(`[ComponentsTreeDataProvider] Returning ${dependencyItems.length} dependency items`);
      return dependencyItems;
    }

    console.log(`[ComponentsTreeDataProvider] No dependencies, returning empty array`);
    return [];
  }

  /**
   * 检查是否有运行时信息
   */
  public hasRuntimeInfo(): boolean {
    return this.runtimeComponents.size > 0;
  }

  /**
   * 获取组件定义位置
   * 
   * @param componentName 组件名称
   * @returns 位置信息
   */
  public getComponentLocation(componentName: string): vscode.Location | undefined {
    // 优先从运行时组件查找
    let component = this.runtimeComponents.get(componentName);
    if (!component) {
      component = this.staticComponents.get(componentName);
    }

    if (!component || !component.location) {
      return undefined;
    }

    const uri = vscode.Uri.parse(component.location.uri);
    const range = new vscode.Range(
      component.location.range.start.line,
      component.location.range.start.character,
      component.location.range.end.line,
      component.location.range.end.character
    );

    return new vscode.Location(uri, range);
  }
}

/**
 * 组件树节点
 */
export class ComponentTreeItem extends vscode.TreeItem {
  /**
   * 组件实例
   */
  public readonly component: Component;

  /**
   * 所有组件的映射（用于查找依赖）
   */
  private readonly allComponents: Map<string, Component>;

  /**
   * 扩展上下文
   */
  private readonly context: vscode.ExtensionContext;

  /**
   * 组件来源
   */
  private readonly source: DataSource;

  /**
   * 创建组件树节点
   * 
   * @param component 组件实例
   * @param allComponents 所有组件的映射
   * @param context 扩展上下文
   * @param source 组件来源
   */
  constructor(
    component: Component,
    allComponents: Map<string, Component>,
    context: vscode.ExtensionContext,
    source: DataSource = DataSource.Static
  ) {
    super(
      component.name,
      component.dependencies.length > 0
        ? vscode.TreeItemCollapsibleState.Collapsed
        : vscode.TreeItemCollapsibleState.None
    );

    this.component = component;
    this.allComponents = allComponents;
    this.context = context;
    this.source = source;

    // 设置上下文值（用于命令菜单）
    this.contextValue = `spring:component-${source}`;

    // 设置工具提示
    this.tooltip = this.buildTooltip();

    // 设置描述
    this.description = this.getDescription();

    // 设置图标
    this.iconPath = this.getIcon();

    // 设置点击命令（跳转到定义）
    // 注意：有依赖的组件也可以点击标题跳转，展开/折叠通过箭头控制
    if (component.location) {
      this.command = {
        command: 'spring.component.navigate',
        title: 'Go to Definition',
        arguments: [component.location],
      };
    }
  }

  /**
   * 构建工具提示
   */
  private buildTooltip(): vscode.MarkdownString {
    const tooltip = new vscode.MarkdownString();
    tooltip.isTrusted = true;

    tooltip.appendMarkdown(`### ${this.component.name}\n\n`);
    tooltip.appendMarkdown(`**Type:** \`${this.component.typeName}\`\n\n`);
    tooltip.appendMarkdown(`**Scope:** ${this.component.scope}\n\n`);

    // 显示来源信息
    if (this.source === DataSource.Runtime) {
      tooltip.appendMarkdown('✅ **Runtime Information**\n\n');
      tooltip.appendMarkdown('_Information from running application_\n\n');
    } else {
      tooltip.appendMarkdown('📝 **Static Analysis**\n\n');
      tooltip.appendMarkdown('_Start the application to see runtime information_\n\n');
    }

    if (this.component.dependencies.length > 0) {
      tooltip.appendMarkdown(`**Dependencies:**\n`);
      this.component.dependencies.forEach((dep) => {
        tooltip.appendMarkdown(`- ${dep}\n`);
      });
    } else {
      tooltip.appendMarkdown(`**Dependencies:** None\n\n`);
    }

    if (this.component.location) {
      tooltip.appendMarkdown(`\n*Click to go to definition*`);
    }

    return tooltip;
  }

  /**
   * 获取描述
   */
  private getDescription(): string {
    const parts: string[] = [];

    // 作用域
    parts.push(this.component.scope);

    // 依赖数量
    if (this.component.dependencies.length > 0) {
      parts.push(`${this.component.dependencies.length} deps`);
    }

    return parts.join(' • ');
  }

  /**
   * 获取图标
   */
  private getIcon(): vscode.ThemeIcon | vscode.Uri {
    // 根据来源使用不同颜色
    const color = this.source === DataSource.Runtime
      ? new vscode.ThemeColor('charts.green')
      : new vscode.ThemeColor('charts.blue');

    // 尝试使用 SVG 图标，如果不存在则使用主题图标
    try {
      return vscode.Uri.joinPath(
        this.context.extensionUri,
        'resources',
        'icons',
        'component.svg'
      );
    } catch {
      return new vscode.ThemeIcon('symbol-class', color);
    }
  }
}


/**
 * 占位符树节点
 * 
 * 用于显示找不到的依赖（外部类型、类型别名、配置等）
 */
export class PlaceholderTreeItem extends vscode.TreeItem {
  /**
   * 创建占位符树节点
   * 
   * @param typeName 类型名称
   */
  constructor(typeName: string) {
    super(typeName, vscode.TreeItemCollapsibleState.None);

    // 设置上下文值
    this.contextValue = 'spring:dependency:external';

    // 设置工具提示
    this.tooltip = new vscode.MarkdownString(
      `**External Dependency**\n\n` +
      `Type: \`${typeName}\`\n\n` +
      `This dependency is not a registered component. It might be:\n` +
      `- An external type from another crate\n` +
      `- A type alias\n` +
      `- A configuration struct\n` +
      `- A primitive type wrapper`
    );

    // 设置描述
    this.description = 'external';

    // 设置图标（使用不同的图标表示外部依赖）
    this.iconPath = new vscode.ThemeIcon(
      'symbol-interface',
      new vscode.ThemeColor('symbolIcon.interfaceForeground')
    );
  }
}
