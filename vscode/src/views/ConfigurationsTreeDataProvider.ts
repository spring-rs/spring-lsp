import * as vscode from 'vscode';
import { LanguageClientManager } from '../languageClient/LanguageClientManager';

/**
 * 配置结构信息
 */
export interface ConfigurationStruct {
  name: string;           // 结构体名称
  prefix: string;         // 配置前缀（从 #[config_prefix = "..."] 提取）
  fields: ConfigField[];  // 字段列表
  location?: {            // 定义位置
    uri: string;
    range: {
      start: { line: number; character: number };
      end: { line: number; character: number };
    };
  };
}

/**
 * 配置字段信息
 */
export interface ConfigField {
  name: string;           // 字段名称
  type: string;           // 字段类型
  optional: boolean;      // 是否可选
  description?: string;   // 描述（从文档注释提取）
}

/**
 * 树节点类型
 */
type ConfigTreeItem = ConfigStructItem | ConfigFieldItem | PlaceholderTreeItem;

/**
 * 配置结构树节点
 */
class ConfigStructItem extends vscode.TreeItem {
  constructor(
    public config: ConfigurationStruct,
    public readonly collapsibleState: vscode.TreeItemCollapsibleState,
    private context: vscode.ExtensionContext
  ) {
    super(config.name, collapsibleState);
    
    this.tooltip = this.buildTooltip();
    this.description = `[${config.prefix}]`;
    this.contextValue = 'spring:configStruct';
    // 使用专用的 config 图标
    this.iconPath = {
      light: vscode.Uri.joinPath(context.extensionUri, 'resources', 'icons', 'config.svg'),
      dark: vscode.Uri.joinPath(context.extensionUri, 'resources', 'icons', 'config.svg')
    };
    
    // 点击时跳转到定义
    if (config.location) {
      this.command = {
        command: 'vscode.open',
        title: 'Open Definition',
        arguments: [
          vscode.Uri.parse(config.location.uri),
          {
            selection: new vscode.Range(
              config.location.range.start.line,
              config.location.range.start.character,
              config.location.range.end.line,
              config.location.range.end.character
            ),
          },
        ],
      };
    }
  }

  private buildTooltip(): vscode.MarkdownString {
    const tooltip = new vscode.MarkdownString();
    tooltip.appendMarkdown(`**Configuration Struct: ${this.config.name}**\n\n`);
    tooltip.appendMarkdown(`Config Prefix: \`[${this.config.prefix}]\`\n\n`);
    tooltip.appendMarkdown(`Fields: ${this.config.fields.length}\n\n`);
    tooltip.appendMarkdown('Click to view definition');
    return tooltip;
  }
}

/**
 * 配置字段树节点
 */
class ConfigFieldItem extends vscode.TreeItem {
  constructor(
    public field: ConfigField,
    public prefix: string
  ) {
    super(field.name, vscode.TreeItemCollapsibleState.None);
    
    this.tooltip = this.buildTooltip();
    this.description = field.type;
    this.contextValue = 'spring:configField';
    this.iconPath = new vscode.ThemeIcon(
      field.optional ? 'symbol-field' : 'symbol-property'
    );
  }

  private buildTooltip(): vscode.MarkdownString {
    const tooltip = new vscode.MarkdownString();
    tooltip.appendMarkdown(`**Field: ${this.field.name}**\n\n`);
    tooltip.appendMarkdown(`Type: \`${this.field.type}\`\n\n`);
    tooltip.appendMarkdown(`Required: ${this.field.optional ? 'No' : 'Yes'}\n\n`);
    
    if (this.field.description) {
      tooltip.appendMarkdown(`Description: ${this.field.description}\n\n`);
    }
    
    tooltip.appendMarkdown(`Usage in config file:\n\`\`\`toml\n[${this.prefix}]\n${this.field.name} = ...\n\`\`\``);
    return tooltip;
  }
}

/**
 * 占位符树节点
 */
class PlaceholderTreeItem extends vscode.TreeItem {
  constructor(message: string) {
    super(message, vscode.TreeItemCollapsibleState.None);
    this.contextValue = 'spring:placeholder';
    this.iconPath = new vscode.ThemeIcon('info');
  }
}

/**
 * 配置树数据提供者
 */
export class ConfigurationsTreeDataProvider implements vscode.TreeDataProvider<ConfigTreeItem> {
  private _onDidChangeTreeData = new vscode.EventEmitter<ConfigTreeItem | undefined>();
  readonly onDidChangeTreeData = this._onDidChangeTreeData.event;

  private configurations: ConfigurationStruct[] = [];
  private loading = false;

  constructor(
    private languageClient: LanguageClientManager,
    private context: vscode.ExtensionContext
  ) {}

  /**
   * 刷新配置列表
   */
  public async refresh(): Promise<void> {
    this.loading = true;
    this._onDidChangeTreeData.fire(undefined);

    try {
      // 从语言服务器获取配置信息
      const client = this.languageClient.getClient();
      if (!client) {
        this.configurations = [];
        return;
      }

      // 获取工作空间路径
      const workspaceFolders = vscode.workspace.workspaceFolders;
      if (!workspaceFolders || workspaceFolders.length === 0) {
        this.configurations = [];
        return;
      }

      // 使用第一个工作空间文件夹
      const appPath = workspaceFolders[0].uri.fsPath;

      // 发送自定义 LSP 请求获取配置结构
      const response = await client.sendRequest<{ configurations: ConfigurationStruct[] }>(
        'spring/configurations',
        { appPath }
      );

      this.configurations = response?.configurations || [];
    } catch (error) {
      console.error('Failed to fetch configurations:', error);
      this.configurations = [];
    } finally {
      this.loading = false;
      this._onDidChangeTreeData.fire(undefined);
    }
  }

  /**
   * 获取树节点
   */
  getTreeItem(element: ConfigTreeItem): vscode.TreeItem {
    return element;
  }

  /**
   * 获取子节点
   */
  async getChildren(element?: ConfigTreeItem): Promise<ConfigTreeItem[]> {
    if (!element) {
      // 根节点
      if (this.loading) {
        return [new PlaceholderTreeItem('Loading configurations...')];
      }

      if (this.configurations.length === 0) {
        return [
          new PlaceholderTreeItem('No configuration structs found'),
          new PlaceholderTreeItem('Add #[derive(Configurable)] to your structs'),
        ];
      }

      // 返回所有配置结构
      return this.configurations.map(
        (config) => new ConfigStructItem(config, vscode.TreeItemCollapsibleState.Collapsed, this.context)
      );
    }

    if (element instanceof ConfigStructItem) {
      // 返回配置结构的字段
      if (element.config.fields.length === 0) {
        return [new PlaceholderTreeItem('No fields defined')];
      }

      return element.config.fields.map(
        (field) => new ConfigFieldItem(field, element.config.prefix)
      );
    }

    return [];
  }

  /**
   * 获取父节点
   */
  getParent(element: ConfigTreeItem): ConfigTreeItem | undefined {
    if (element instanceof ConfigFieldItem) {
      // 查找对应的配置结构
      const config = this.configurations.find((c) => c.prefix === element.prefix);
      if (config) {
        return new ConfigStructItem(config, vscode.TreeItemCollapsibleState.Collapsed, this.context);
      }
    }
    return undefined;
  }
}
