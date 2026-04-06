//! ArgusFlow 核心库
//!
//! 提供文献搜索、存储、引用分析等核心功能，无 CLI 依赖。

mod argusflow;

pub use argusflow::{ArgusFlow, ArgusFlowBuilder, SortBy, SearchScope, GraphFormat};