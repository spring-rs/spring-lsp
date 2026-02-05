import * as path from 'path';
import * as vscode from 'vscode';
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from 'vscode-languageclient/node';

let client: LanguageClient | undefined;

export async function activate(context: vscode.ExtensionContext) {
  const config = vscode.workspace.getConfiguration('spring-rs-lsp');
  
  if (!config.get<boolean>('enable', true)) {
    return;
  }

  const serverPath = await getServerPath(config);
  
  if (!serverPath) {
    vscode.window.showErrorMessage(
      'Spring RS LSP: Could not find spring-lsp executable. Please install it or configure the path.'
    );
    return;
  }

  const serverOptions: ServerOptions = {
    command: serverPath,
    args: [],
    transport: TransportKind.stdio,
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [
      { scheme: 'file', language: 'toml', pattern: '**/.spring-lsp.toml' },
      { scheme: 'file', language: 'toml', pattern: '**/config/app*.toml' },
    ],
    synchronize: {
      fileEvents: vscode.workspace.createFileSystemWatcher('**/*.toml'),
    },
  };

  client = new LanguageClient(
    'spring-rs-lsp',
    'Spring RS Language Server',
    serverOptions,
    clientOptions
  );

  try {
    await client.start();
    vscode.window.showInformationMessage('Spring RS LSP started successfully');
  } catch (error) {
    vscode.window.showErrorMessage(
      `Failed to start Spring RS LSP: ${error}`
    );
  }
}

export async function deactivate(): Promise<void> {
  if (client) {
    await client.stop();
  }
}

async function getServerPath(
  config: vscode.WorkspaceConfiguration
): Promise<string | undefined> {
  // 1. Check user-configured path
  const configuredPath = config.get<string>('serverPath');
  if (configuredPath && configuredPath.trim() !== '') {
    return configuredPath;
  }

  // 2. Check if spring-lsp is in PATH
  const pathCommand = process.platform === 'win32' ? 'where' : 'which';
  try {
    const { execSync } = require('child_process');
    const result = execSync(`${pathCommand} spring-lsp`, {
      encoding: 'utf8',
    }).trim();
    if (result) {
      return result.split('\n')[0];
    }
  } catch {
    // Not in PATH, continue
  }

  // 3. Check common installation locations
  const possiblePaths = [
    path.join(process.env.HOME || '', '.cargo', 'bin', 'spring-lsp'),
    path.join(process.env.USERPROFILE || '', '.cargo', 'bin', 'spring-lsp.exe'),
    '/usr/local/bin/spring-lsp',
  ];

  for (const p of possiblePaths) {
    try {
      const fs = require('fs');
      if (fs.existsSync(p)) {
        return p;
      }
    } catch {
      continue;
    }
  }

  // 4. Prompt user to install
  const install = await vscode.window.showWarningMessage(
    'Spring RS LSP server not found. Would you like to install it?',
    'Install',
    'Configure Path'
  );

  if (install === 'Install') {
    vscode.env.openExternal(
      vscode.Uri.parse('https://github.com/spring-rs/spring-lsp#installation')
    );
  } else if (install === 'Configure Path') {
    vscode.commands.executeCommand(
      'workbench.action.openSettings',
      'spring-rs-lsp.serverPath'
    );
  }

  return undefined;
}
