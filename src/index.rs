//! 索引管理模块
//!
//! 提供项目级别的索引管理，包括符号索引、路由索引和组件索引。
//! 使用并发安全的数据结构支持多线程访问。

use dashmap::DashMap;
use lsp_types::{Location, Url};
use std::sync::{Arc, RwLock};

/// 符号信息
#[derive(Debug, Clone)]
pub struct SymbolInfo {
    /// 符号名称
    pub name: String,
    /// 符号类型
    pub symbol_type: SymbolType,
    /// 位置
    pub location: Location,
}

/// 符号类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolType {
    /// 结构体
    Struct,
    /// 函数
    Function,
    /// 常量
    Const,
    /// 静态变量
    Static,
    /// 模块
    Module,
}

/// 符号索引
#[derive(Debug, Clone)]
pub struct SymbolIndex {
    /// 符号映射（内部使用 DashMap 提供并发安全）
    pub symbols: DashMap<String, Vec<SymbolInfo>>,
}

impl SymbolIndex {
    /// 创建新的符号索引
    pub fn new() -> Self {
        Self {
            symbols: DashMap::new(),
        }
    }

    /// 添加符号
    pub fn add(&self, name: String, info: SymbolInfo) {
        self.symbols.entry(name).or_default().push(info);
    }

    /// 查找符号
    pub fn find(&self, name: &str) -> Vec<SymbolInfo> {
        self.symbols
            .get(name)
            .map(|v| v.clone())
            .unwrap_or_default()
    }

    /// 清空索引
    pub fn clear(&self) {
        self.symbols.clear();
    }
}

impl Default for SymbolIndex {
    fn default() -> Self {
        Self::new()
    }
}

/// 组件信息
#[derive(Debug, Clone)]
pub struct ComponentInfo {
    /// 组件名称
    pub name: String,
    /// 类型名称
    pub type_name: String,
    /// 位置
    pub location: Location,
    /// 所属插件
    pub plugin: Option<String>,
}

/// 组件索引
#[derive(Debug, Clone)]
pub struct ComponentIndex {
    /// 组件映射（内部使用 DashMap 提供并发安全）
    pub components: DashMap<String, ComponentInfo>,
}

impl ComponentIndex {
    /// 创建新的组件索引
    pub fn new() -> Self {
        Self {
            components: DashMap::new(),
        }
    }

    /// 添加组件
    pub fn add(&self, name: String, info: ComponentInfo) {
        self.components.insert(name, info);
    }

    /// 查找组件
    pub fn find(&self, name: &str) -> Option<ComponentInfo> {
        self.components.get(name).map(|v| v.clone())
    }

    /// 清空索引
    pub fn clear(&self) {
        self.components.clear();
    }
}

impl Default for ComponentIndex {
    fn default() -> Self {
        Self::new()
    }
}

/// 工作空间信息
pub struct Workspace {
    /// 根目录 URI
    pub root_uri: Url,
    /// 文档列表
    pub documents: Vec<(Url, String)>,
}

/// 索引管理器
///
/// 管理项目级别的索引，包括符号索引、路由索引和组件索引。
/// 使用 RwLock 保护索引结构，因为重建索引时需要整体替换。
pub struct IndexManager {
    /// 符号索引（使用 RwLock 因为重建时需要整体替换）
    symbol_index: Arc<RwLock<SymbolIndex>>,
    /// 路由索引
    route_index: Arc<RwLock<crate::route::RouteIndex>>,
    /// 组件索引
    component_index: Arc<RwLock<ComponentIndex>>,
}

impl IndexManager {
    /// 创建新的索引管理器
    pub fn new() -> Self {
        Self {
            symbol_index: Arc::new(RwLock::new(SymbolIndex::new())),
            route_index: Arc::new(RwLock::new(crate::route::RouteIndex::new())),
            component_index: Arc::new(RwLock::new(ComponentIndex::new())),
        }
    }

    /// 构建索引（异步，可能耗时）
    ///
    /// 在后台线程构建索引，完成后原子性地替换整个索引。
    /// 这确保读取者看到的是完整的旧索引或完整的新索引，而不会看到中间状态。
    pub async fn build(&self, workspace: &Workspace) {
        let symbol_index = self.symbol_index.clone();
        let route_index = self.route_index.clone();
        let component_index = self.component_index.clone();

        // 克隆工作空间数据以便在异步任务中使用
        let root_uri = workspace.root_uri.clone();
        let documents = workspace.documents.clone();

        tokio::spawn(async move {
            // 构建新索引（不持有锁）
            let new_symbols = Self::build_symbol_index(&root_uri, &documents).await;
            let new_routes = Self::build_route_index(&root_uri, &documents).await;
            let new_components = Self::build_component_index(&root_uri, &documents).await;

            // 原子性地替换整个索引（短暂持有写锁）
            *symbol_index
                .write()
                .expect("Failed to acquire write lock on symbol index") = new_symbols;
            *route_index
                .write()
                .expect("Failed to acquire write lock on route index") = new_routes;
            *component_index
                .write()
                .expect("Failed to acquire write lock on component index") = new_components;

            tracing::info!("Index rebuild completed");
        });
    }

    /// 增量更新索引
    ///
    /// 当单个文档发生变化时，只更新该文档相关的索引条目。
    pub fn update(&self, uri: &Url, _content: &str) {
        // 解析文档并更新索引
        // 这里简化实现，实际应该解析文档并提取符号、路由和组件信息

        tracing::debug!("Updating index for {}", uri);

        // TODO: 实现增量更新逻辑
        // 1. 解析文档
        // 2. 提取符号、路由和组件
        // 3. 更新对应的索引
    }

    /// 查找符号
    pub fn find_symbol(&self, name: &str) -> Vec<SymbolInfo> {
        let index = self
            .symbol_index
            .read()
            .expect("Failed to acquire read lock on symbol index");
        index.find(name)
    }

    /// 查找组件
    pub fn find_component(&self, name: &str) -> Option<ComponentInfo> {
        let index = self
            .component_index
            .read()
            .expect("Failed to acquire read lock on component index");
        index.find(name)
    }

    /// 获取所有路由
    pub fn get_all_routes(&self) -> Vec<crate::route::RouteInfo> {
        let index = self
            .route_index
            .read()
            .expect("Failed to acquire read lock on route index");
        index.routes.clone()
    }

    /// 构建符号索引（内部方法）
    async fn build_symbol_index(_root_uri: &Url, _documents: &[(Url, String)]) -> SymbolIndex {
        // TODO: 实现符号索引构建
        // 1. 遍历所有 Rust 文件
        // 2. 解析语法树
        // 3. 提取符号信息
        SymbolIndex::new()
    }

    /// 构建路由索引（内部方法）
    async fn build_route_index(
        _root_uri: &Url,
        _documents: &[(Url, String)],
    ) -> crate::route::RouteIndex {
        // TODO: 实现路由索引构建
        // 1. 遍历所有 Rust 文件
        // 2. 识别路由宏
        // 3. 构建路由索引
        crate::route::RouteIndex::new()
    }

    /// 构建组件索引（内部方法）
    async fn build_component_index(
        _root_uri: &Url,
        _documents: &[(Url, String)],
    ) -> ComponentIndex {
        // TODO: 实现组件索引构建
        // 1. 遍历所有 Rust 文件
        // 2. 识别 #[derive(Service)] 和组件注册
        // 3. 构建组件索引
        ComponentIndex::new()
    }
}

impl Default for IndexManager {
    fn default() -> Self {
        Self::new()
    }
}
