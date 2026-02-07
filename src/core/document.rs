//! 文档管理模块

use dashmap::DashMap;
use lsp_types::{TextDocumentContentChangeEvent, Url};

/// 文档管理器
pub struct DocumentManager {
    /// 文档缓存（DashMap 本身就是并发安全的）
    documents: DashMap<Url, Document>,
}

/// 文档
#[derive(Debug, Clone)]
pub struct Document {
    /// 文档 URI
    pub uri: Url,
    /// 文档版本
    pub version: i32,
    /// 文档内容
    pub content: String,
    /// 语言 ID
    pub language_id: String,
}

impl DocumentManager {
    /// 创建新的文档管理器
    pub fn new() -> Self {
        Self {
            documents: DashMap::new(),
        }
    }

    /// 打开文档
    pub fn open(&self, uri: Url, version: i32, content: String, language_id: String) {
        let doc = Document {
            uri: uri.clone(),
            version,
            content,
            language_id,
        };
        self.documents.insert(uri, doc);
    }

    /// 修改文档
    pub fn change(&self, uri: &Url, version: i32, changes: Vec<TextDocumentContentChangeEvent>) {
        if let Some(mut doc) = self.documents.get_mut(uri) {
            doc.version = version;

            // 应用修改
            for change in changes {
                if let Some(range) = change.range {
                    // 增量修改
                    if let Err(e) =
                        Self::apply_incremental_change(&mut doc.content, range, &change.text)
                    {
                        tracing::error!("Failed to apply incremental change: {}", e);
                        // 降级到全量更新
                        doc.content = change.text;
                    }
                } else {
                    // 全量修改
                    doc.content = change.text;
                }
            }
        }
    }

    /// 应用增量修改
    fn apply_incremental_change(
        content: &mut String,
        range: lsp_types::Range,
        text: &str,
    ) -> Result<(), String> {
        // 将内容按行分割
        let lines: Vec<&str> = content.lines().collect();

        // 验证范围的有效性
        let start_line = range.start.line as usize;
        let end_line = range.end.line as usize;

        if start_line > lines.len() || end_line > lines.len() {
            return Err(format!(
                "Invalid range: start_line={}, end_line={}, total_lines={}",
                start_line,
                end_line,
                lines.len()
            ));
        }

        // 计算起始和结束位置的字节偏移
        let start_offset = Self::position_to_offset(content, range.start)?;
        let end_offset = Self::position_to_offset(content, range.end)?;

        if start_offset > end_offset || end_offset > content.len() {
            return Err(format!(
                "Invalid offsets: start={}, end={}, content_len={}",
                start_offset,
                end_offset,
                content.len()
            ));
        }

        // 构建新内容
        let mut new_content =
            String::with_capacity(content.len() - (end_offset - start_offset) + text.len());
        new_content.push_str(&content[..start_offset]);
        new_content.push_str(text);
        new_content.push_str(&content[end_offset..]);

        *content = new_content;
        Ok(())
    }

    /// 将 LSP Position 转换为字节偏移
    fn position_to_offset(content: &str, position: lsp_types::Position) -> Result<usize, String> {
        let mut offset = 0;
        let mut current_line = 0;
        let target_line = position.line as usize;
        let target_char = position.character as usize;

        for line in content.lines() {
            if current_line == target_line {
                // 找到目标行，计算字符偏移
                let char_offset = Self::char_offset_to_byte_offset(line, target_char)?;
                return Ok(offset + char_offset);
            }

            // 移动到下一行（包括换行符）
            offset += line.len() + 1; // +1 for '\n'
            current_line += 1;
        }

        // 如果到达文件末尾
        if current_line == target_line && target_char == 0 {
            return Ok(offset);
        }

        Err(format!(
            "Position out of bounds: line={}, char={}, total_lines={}",
            target_line, target_char, current_line
        ))
    }

    /// 将字符偏移转换为字节偏移
    fn char_offset_to_byte_offset(line: &str, char_offset: usize) -> Result<usize, String> {
        let mut byte_offset = 0;
        let mut current_char = 0;

        for ch in line.chars() {
            if current_char == char_offset {
                return Ok(byte_offset);
            }
            byte_offset += ch.len_utf8();
            current_char += 1;
        }

        // 允许在行尾
        if current_char == char_offset {
            return Ok(byte_offset);
        }

        Err(format!(
            "Character offset out of bounds: char_offset={}, line_length={}",
            char_offset, current_char
        ))
    }

    /// 关闭文档
    pub fn close(&self, uri: &Url) {
        self.documents.remove(uri);
    }

    /// 获取文档（返回克隆以避免锁竞争）
    pub fn get(&self, uri: &Url) -> Option<Document> {
        self.documents.get(uri).map(|doc| doc.clone())
    }

    /// 获取文档的只读引用（用于快速访问）
    pub fn with_document<F, R>(&self, uri: &Url, f: F) -> Option<R>
    where
        F: FnOnce(&Document) -> R,
    {
        self.documents.get(uri).map(|doc| f(&doc))
    }
}

impl Default for DocumentManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lsp_types::{Position, Range};

    #[test]
    fn test_open_document() {
        let manager = DocumentManager::new();
        let uri: Url = "file:///test.toml".parse().unwrap();

        manager.open(uri.clone(), 1, "content".to_string(), "toml".to_string());

        let doc = manager.get(&uri).unwrap();
        assert_eq!(doc.version, 1);
        assert_eq!(doc.content, "content");
        assert_eq!(doc.language_id, "toml");
    }

    #[test]
    fn test_close_document() {
        let manager = DocumentManager::new();
        let uri: Url = "file:///test.toml".parse().unwrap();

        manager.open(uri.clone(), 1, "content".to_string(), "toml".to_string());
        assert!(manager.get(&uri).is_some());

        manager.close(&uri);
        assert!(manager.get(&uri).is_none());
    }

    #[test]
    fn test_full_content_change() {
        let manager = DocumentManager::new();
        let uri: Url = "file:///test.toml".parse().unwrap();

        manager.open(
            uri.clone(),
            1,
            "old content".to_string(),
            "toml".to_string(),
        );

        let changes = vec![TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: "new content".to_string(),
        }];

        manager.change(&uri, 2, changes);

        let doc = manager.get(&uri).unwrap();
        assert_eq!(doc.version, 2);
        assert_eq!(doc.content, "new content");
    }

    #[test]
    fn test_incremental_change_single_line() {
        let manager = DocumentManager::new();
        let uri: Url = "file:///test.toml".parse().unwrap();

        manager.open(
            uri.clone(),
            1,
            "hello world".to_string(),
            "toml".to_string(),
        );

        // 替换 "world" 为 "rust"
        let changes = vec![TextDocumentContentChangeEvent {
            range: Some(Range {
                start: Position {
                    line: 0,
                    character: 6,
                },
                end: Position {
                    line: 0,
                    character: 11,
                },
            }),
            range_length: None,
            text: "rust".to_string(),
        }];

        manager.change(&uri, 2, changes);

        let doc = manager.get(&uri).unwrap();
        assert_eq!(doc.content, "hello rust");
    }

    #[test]
    fn test_incremental_change_multiline() {
        let manager = DocumentManager::new();
        let uri: Url = "file:///test.toml".parse().unwrap();

        let initial_content = "line 1\nline 2\nline 3";
        manager.open(
            uri.clone(),
            1,
            initial_content.to_string(),
            "toml".to_string(),
        );

        // 替换第二行的 "line 2" 为 "modified"
        let changes = vec![TextDocumentContentChangeEvent {
            range: Some(Range {
                start: Position {
                    line: 1,
                    character: 0,
                },
                end: Position {
                    line: 1,
                    character: 6,
                },
            }),
            range_length: None,
            text: "modified".to_string(),
        }];

        manager.change(&uri, 2, changes);

        let doc = manager.get(&uri).unwrap();
        assert_eq!(doc.content, "line 1\nmodified\nline 3");
    }

    #[test]
    fn test_incremental_change_insert() {
        let manager = DocumentManager::new();
        let uri: Url = "file:///test.toml".parse().unwrap();

        manager.open(uri.clone(), 1, "hello".to_string(), "toml".to_string());

        // 在 "hello" 后插入 " world"
        let changes = vec![TextDocumentContentChangeEvent {
            range: Some(Range {
                start: Position {
                    line: 0,
                    character: 5,
                },
                end: Position {
                    line: 0,
                    character: 5,
                },
            }),
            range_length: None,
            text: " world".to_string(),
        }];

        manager.change(&uri, 2, changes);

        let doc = manager.get(&uri).unwrap();
        assert_eq!(doc.content, "hello world");
    }

    #[test]
    fn test_incremental_change_delete() {
        let manager = DocumentManager::new();
        let uri: Url = "file:///test.toml".parse().unwrap();

        manager.open(
            uri.clone(),
            1,
            "hello world".to_string(),
            "toml".to_string(),
        );

        // 删除 " world"
        let changes = vec![TextDocumentContentChangeEvent {
            range: Some(Range {
                start: Position {
                    line: 0,
                    character: 5,
                },
                end: Position {
                    line: 0,
                    character: 11,
                },
            }),
            range_length: None,
            text: "".to_string(),
        }];

        manager.change(&uri, 2, changes);

        let doc = manager.get(&uri).unwrap();
        assert_eq!(doc.content, "hello");
    }

    #[test]
    fn test_incremental_change_utf8() {
        let manager = DocumentManager::new();
        let uri: Url = "file:///test.toml".parse().unwrap();

        manager.open(uri.clone(), 1, "你好世界".to_string(), "toml".to_string());

        // 替换 "世界" 为 "Rust"
        let changes = vec![TextDocumentContentChangeEvent {
            range: Some(Range {
                start: Position {
                    line: 0,
                    character: 2,
                },
                end: Position {
                    line: 0,
                    character: 4,
                },
            }),
            range_length: None,
            text: "Rust".to_string(),
        }];

        manager.change(&uri, 2, changes);

        let doc = manager.get(&uri).unwrap();
        assert_eq!(doc.content, "你好Rust");
    }

    #[test]
    fn test_with_document() {
        let manager = DocumentManager::new();
        let uri: Url = "file:///test.toml".parse().unwrap();

        manager.open(uri.clone(), 1, "content".to_string(), "toml".to_string());

        let length = manager.with_document(&uri, |doc| doc.content.len());
        assert_eq!(length, Some(7));

        let not_found = manager.with_document(&"file:///notfound.toml".parse().unwrap(), |doc| {
            doc.content.len()
        });
        assert_eq!(not_found, None);
    }

    #[test]
    fn test_position_to_offset() {
        let content = "line 1\nline 2\nline 3";

        // 第一行开始
        let offset = DocumentManager::position_to_offset(
            content,
            Position {
                line: 0,
                character: 0,
            },
        )
        .unwrap();
        assert_eq!(offset, 0);

        // 第一行 "line" 后
        let offset = DocumentManager::position_to_offset(
            content,
            Position {
                line: 0,
                character: 4,
            },
        )
        .unwrap();
        assert_eq!(offset, 4);

        // 第二行开始
        let offset = DocumentManager::position_to_offset(
            content,
            Position {
                line: 1,
                character: 0,
            },
        )
        .unwrap();
        assert_eq!(offset, 7); // "line 1\n"

        // 第三行开始
        let offset = DocumentManager::position_to_offset(
            content,
            Position {
                line: 2,
                character: 0,
            },
        )
        .unwrap();
        assert_eq!(offset, 14); // "line 1\nline 2\n"
    }

    #[test]
    fn test_char_offset_to_byte_offset() {
        // ASCII
        let line = "hello";
        let offset = DocumentManager::char_offset_to_byte_offset(line, 2).unwrap();
        assert_eq!(offset, 2);

        // UTF-8
        let line = "你好世界";
        let offset = DocumentManager::char_offset_to_byte_offset(line, 2).unwrap();
        assert_eq!(offset, 6); // 每个中文字符 3 字节

        // 行尾
        let offset = DocumentManager::char_offset_to_byte_offset(line, 4).unwrap();
        assert_eq!(offset, 12);
    }

    #[test]
    fn test_multiple_changes() {
        let manager = DocumentManager::new();
        let uri: Url = "file:///test.toml".parse().unwrap();

        manager.open(uri.clone(), 1, "a b c".to_string(), "toml".to_string());

        // 应用多个修改
        let changes = vec![
            TextDocumentContentChangeEvent {
                range: Some(Range {
                    start: Position {
                        line: 0,
                        character: 0,
                    },
                    end: Position {
                        line: 0,
                        character: 1,
                    },
                }),
                range_length: None,
                text: "x".to_string(),
            },
            TextDocumentContentChangeEvent {
                range: Some(Range {
                    start: Position {
                        line: 0,
                        character: 2,
                    },
                    end: Position {
                        line: 0,
                        character: 3,
                    },
                }),
                range_length: None,
                text: "y".to_string(),
            },
        ];

        manager.change(&uri, 2, changes);

        let doc = manager.get(&uri).unwrap();
        // 注意：LSP 的修改是按顺序应用的，每次修改后文档内容会改变
        // 第一次修改: "a b c" -> "x b c"
        // 第二次修改: "x b c" -> "x y c"
        assert_eq!(doc.content, "x y c");
    }
}
