//! 内存协调总线的可变状态容器。
//!
//! 该模块集中保存消息去重、agent inbox、死信、共享上下文和统计信息。状态本身不
//! 暴露锁；外层总线负责同步，本模块只提供带限制值的初始化入口。

use std::collections::{HashMap, HashSet, VecDeque};

use crate::app::agent::coordination::types::{
    DeadLetter, InMemoryMessageBusLimits, InMemoryMessageBusStats, SequencedEnvelope,
    SharedContextEntry,
};

/// 内存消息总线的完整状态快照。
///
/// 字段保持 `pub(crate)`，让同一子系统内的队列、上下文和死信模块可以直接维护各自
/// 的索引，同时避免跨 crate 暴露内部存储布局。
#[derive(Debug, Default)]
pub(crate) struct BusState {
    pub(crate) next_sequence: u64,
    pub(crate) seen_message_ids: HashSet<String>,
    pub(crate) seen_message_order: VecDeque<String>,
    pub(crate) inboxes: HashMap<String, VecDeque<SequencedEnvelope>>,
    pub(crate) inbox_correlation_counts: HashMap<String, HashMap<String, usize>>,
    pub(crate) dead_letters: Vec<DeadLetter>,
    pub(crate) dead_letters_by_correlation: HashMap<String, VecDeque<DeadLetter>>,
    pub(crate) context: HashMap<String, SharedContextEntry>,
    pub(crate) context_order: VecDeque<String>,
    pub(crate) delegate_context_order: VecDeque<String>,
    pub(crate) context_order_by_correlation: HashMap<String, VecDeque<String>>,
    pub(crate) delegate_context_order_by_correlation: HashMap<String, VecDeque<String>>,
    pub(crate) context_correlation_by_key: HashMap<String, String>,
    pub(crate) limits: InMemoryMessageBusLimits,
    pub(crate) stats: InMemoryMessageBusStats,
}

impl BusState {
    /// 使用指定限制创建总线状态。
    ///
    /// 参数 `limits` 控制 inbox、死信、上下文和去重集合的容量。返回初始化后的
    /// `BusState`；为避免后续容量检查出现不可用状态，传入的 0 会被提升为 1。
    pub(crate) fn with_limits(mut limits: InMemoryMessageBusLimits) -> Self {
        if limits.max_inbox_messages_per_agent == 0 {
            limits.max_inbox_messages_per_agent = 1;
        }
        if limits.max_dead_letters == 0 {
            limits.max_dead_letters = 1;
        }
        if limits.max_context_entries == 0 {
            limits.max_context_entries = 1;
        }
        if limits.max_seen_message_ids == 0 {
            limits.max_seen_message_ids = 1;
        }

        Self {
            next_sequence: 1,
            seen_message_ids: HashSet::new(),
            seen_message_order: VecDeque::new(),
            inboxes: HashMap::new(),
            inbox_correlation_counts: HashMap::new(),
            dead_letters: Vec::new(),
            dead_letters_by_correlation: HashMap::new(),
            context: HashMap::new(),
            context_order: VecDeque::new(),
            delegate_context_order: VecDeque::new(),
            context_order_by_correlation: HashMap::new(),
            delegate_context_order_by_correlation: HashMap::new(),
            context_correlation_by_key: HashMap::new(),
            limits,
            stats: InMemoryMessageBusStats::default(),
        }
    }
}
