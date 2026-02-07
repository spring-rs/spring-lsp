import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs';
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from 'vscode-languageclient/node';

/**
 * 语言客户端管理器
 * 
 * 负责启动、管理和与 spring-rs 语言服务器通信
 */
export class LanguageClientManager implements vscode.Disposable {
  /**
   * 语言客户端实例
   */
  private client: LanguageClient | undefined;

  /**
   * 输出通道（用于扩展日志）
   */
  private outputChannel: vscode.OutputChannel;

  /**
   * 扩展上下文
   */
  private readonly context: vscode.ExtensionContext;

  /**
   * 创建 LanguageClientManager 实例
   * 
   * @param context 扩展上下文
   * @param outputChannel 输出通道（用于扩展日志）
   */
  constructor(context: vscode.ExtensionContext, outputChannel: vscode.OutputChannel) {
    this.context = context;
    this.outputChannel = outputChannel;
  }

  /**
   * 启动语言服务器
   */
  public async start(): Promise<void> {
    try {
      // 查找语言服务器可执行文件
      const serverPath = await this.findServerExecutable();

      if (!serverPath) {
        this.showServerNotFoundError();
        return;
      }

      this.outputChannel.appendLine(`Found Spring LSP server at: ${serverPath}`);

      // 配置服务器选项
      const serverOptions: ServerOptions = {
        command: serverPath,
        args: ['--stdio'],
        transport: TransportKind.stdio,
      };

      // 配置客户端选项
      const clientOptions: LanguageClientOptions = {
        documentSelector: [
          { scheme: 'file', language: 'toml', pattern: '**/.spring-lsp.toml' },
          { scheme: 'file', language: 'toml', pattern: '**/config/app*.toml' },
          { scheme: 'file', language: 'rust' },
        ],
        synchronize: {
          fileEvents: vscode.workspace.createFileSystemWatcher(
            '**/{*.toml,*.rs,Cargo.toml}'
          ),
        },
        // 不指定 outputChannel，让 LSP 客户端自动创建
        // 这样会创建一个名为 "Spring RS" 的输出通道用于语言服务器日志
      };

      // 创建语言客户端
      this.client = new LanguageClient(
        'spring-rs',
        'Spring RS',
        serverOptions,
        clientOptions
      );

      // 启动客户端
      await this.client.start();

      this.outputChannel.appendLine('Spring LSP server started successfully');
    } catch (error) {
      this.outputChannel.appendLine(`Failed to start Spring LSP server: ${error}`);
      vscode.window.showErrorMessage(
        `Failed to start Spring LSP server: ${error}`,
        'Open Settings',
        'View Documentation'
      ).then(selection => {
        if (selection === 'Open Settings') {
          vscode.commands.executeCommand(
            'workbench.action.openSettings',
            'spring-rs.serverPath'
          );
        } else if (selection === 'View Documentation') {
          vscode.env.openExternal(
            vscode.Uri.parse('https://spring-rs.github.io/')
          );
        }
      });
    }
  }

  /**
   * 停止语言服务器
   */
  public async stop(): Promise<void> {
    if (this.client) {
      this.outputChannel.appendLine('Stopping Spring LSP server...');
      await this.client.stop();
      this.client = undefined;
      this.outputChannel.appendLine('Spring LSP server stopped');
    }
  }

  /**
   * 发送自定义请求到语言服务器
   * 
   * @param method 请求方法名
   * @param params 请求参数
   * @param timeout 超时时间（毫秒），默认 30 秒
   * @returns 响应结果
   */
  public async sendRequest<T>(
    method: string,
    params: any,
    timeout: number = 30000
  ): Promise<T | null> {
    if (!this.client) {
      this.outputChannel.appendLine('Language client not initialized');
      return null;
    }

    try {
      const result = await Promise.race([
        this.client.sendRequest<T>(method, params),
        new Promise<null>((_, reject) =>
          setTimeout(() => reject(new Error('Request timeout')), timeout)
        ),
      ]);

      return result;
    } catch (error) {
      this.outputChannel.appendLine(`LSP request failed: ${method} - ${error}`);
      
      vscode.window.showWarningMessage(
        `Request to language server timed out: ${method}`,
        'Retry'
      ).then(selection => {
        if (selection === 'Retry') {
          this.sendRequest<T>(method, params, timeout);
        }
      });

      return null;
    }
  }

  /**
   * 获取语言客户端实例
   * 
   * @returns 语言客户端实例，如果未初始化返回 undefined
   */
  public getClient(): LanguageClient | undefined {
    return this.client;
  }

  /**
   * 检查语言服务器是否正在运行
   * 
   * @returns 如果正在运行返回 true
   */
  public isRunning(): boolean {
    return this.client !== undefined;
  }

  /**
   * 查找语言服务器可执行文件
   * 
   * 按以下顺序查找：
   * 1. 配置中指定的路径
   * 2. 扩展目录的 bin/ 子目录
   * 3. 系统 PATH
   * 
   * @returns 服务器可执行文件路径，如果未找到返回 undefined
   */
  private async findServerExecutable(): Promise<string | undefined> {
    // 1. 检查配置中指定的路径
    const config = vscode.workspace.getConfiguration('spring-rs');
    const configPath = config.get<string>('serverPath');

    if (configPath) {
      if (fs.existsSync(configPath)) {
        return configPath;
      } else {
        this.outputChannel.appendLine(
          `Configured server path does not exist: ${configPath}`
        );
      }
    }

    // 2. 检查扩展目录中的二进制文件
    const extensionPath = this.context.extensionPath;
    const platform = process.platform;
    const binaryName = platform === 'win32' ? 'spring-lsp.exe' : 'spring-lsp';
    const binaryPath = path.join(extensionPath, 'bin', binaryName);

    if (fs.existsSync(binaryPath)) {
      return binaryPath;
    }

    // 3. 检查系统 PATH
    const pathResult = await this.findInPath(binaryName);
    if (pathResult) {
      return pathResult;
    }

    return undefined;
  }

  /**
   * 在系统 PATH 中查找可执行文件
   * 
   * @param binaryName 可执行文件名
   * @returns 完整路径，如果未找到返回 undefined
   */
  private async findInPath(binaryName: string): Promise<string | undefined> {
    try {
      const { exec } = require('child_process');
      const command = process.platform === 'win32' ? 'where' : 'which';

      return new Promise<string | undefined>((resolve) => {
        exec(`${command} ${binaryName}`, (error: any, stdout: string) => {
          if (error) {
            resolve(undefined);
          } else {
            const path = stdout.trim().split('\n')[0];
            resolve(path || undefined);
          }
        });
      });
    } catch {
      return undefined;
    }
  }

  /**
   * 显示服务器未找到错误
   */
  private showServerNotFoundError(): void {
    const message = 'Spring LSP server not found. Please install it or configure the path.';
    
    vscode.window.showErrorMessage(
      message,
      'Open Settings',
      'View Documentation',
      'Install Guide'
    ).then(selection => {
      if (selection === 'Open Settings') {
        vscode.commands.executeCommand(
          'workbench.action.openSettings',
          'spring-rs.serverPath'
        );
      } else if (selection === 'View Documentation') {
        vscode.env.openExternal(
          vscode.Uri.parse('https://spring-rs.github.io/')
        );
      } else if (selection === 'Install Guide') {
        vscode.env.openExternal(
          vscode.Uri.parse('https://spring-rs.github.io/')
        );
      }
    });

    this.outputChannel.appendLine(message);
    this.outputChannel.appendLine('Search paths:');
    this.outputChannel.appendLine(`  1. Configuration: spring-rs.serverPath`);
    this.outputChannel.appendLine(`  2. Extension directory: ${this.context.extensionPath}/bin/`);
    this.outputChannel.appendLine(`  3. System PATH`);
  }

  /**
   * 清理资源
   */
  public dispose(): void {
    this.stop();
    this.outputChannel.dispose();
  }
}
