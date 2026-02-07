/**
 * Location 工具函数
 * 
 * 提供 Location 类型转换和操作的工具函数
 */

import * as vscode from 'vscode';
import { Location } from '../types';

/**
 * 将自定义 Location 转换为 VSCode Location
 * 
 * @param location 自定义 Location 对象
 * @returns VSCode Location 对象
 */
export function toVSCodeLocation(location: Location): vscode.Location {
  const uri = vscode.Uri.parse(location.uri);
  const range = new vscode.Range(
    location.range.start.line,
    location.range.start.character,
    location.range.end.line,
    location.range.end.character
  );

  return new vscode.Location(uri, range);
}

/**
 * 将 VSCode Location 转换为自定义 Location
 * 
 * @param location VSCode Location 对象
 * @returns 自定义 Location 对象
 */
export function fromVSCodeLocation(location: vscode.Location): Location {
  return {
    uri: location.uri.toString(),
    range: {
      start: {
        line: location.range.start.line,
        character: location.range.start.character
      },
      end: {
        line: location.range.end.line,
        character: location.range.end.character
      }
    }
  };
}

/**
 * 导航到指定位置
 * 
 * @param location 位置信息（可以是自定义 Location 或 VSCode Location）
 * @param options 打开选项
 */
export async function navigateToLocation(
  location: Location | vscode.Location,
  options?: vscode.TextDocumentShowOptions
): Promise<void> {
  const vscodeLocation = location instanceof vscode.Location
    ? location
    : toVSCodeLocation(location);

  await vscode.window.showTextDocument(vscodeLocation.uri, {
    selection: vscodeLocation.range,
    ...options
  });
}

/**
 * 在编辑器中高亮显示位置
 * 
 * @param location 位置信息
 * @param decorationType 装饰类型
 */
export async function highlightLocation(
  location: Location | vscode.Location,
  decorationType?: vscode.TextEditorDecorationType
): Promise<void> {
  const vscodeLocation = location instanceof vscode.Location
    ? location
    : toVSCodeLocation(location);

  const editor = await vscode.window.showTextDocument(vscodeLocation.uri);

  if (decorationType) {
    editor.setDecorations(decorationType, [vscodeLocation.range]);
  }

  editor.revealRange(vscodeLocation.range, vscode.TextEditorRevealType.InCenter);
}
