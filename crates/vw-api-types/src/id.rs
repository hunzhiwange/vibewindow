//! 领域实体 ID 的轻量包装类型。
//!
//! 本模块用字符串新类型包装不同业务域的标识符，目标是：
//! - 在 Rust 类型系统中区分会话、项目、消息、任务等 ID
//! - 保持序列化形式仍然是字符串，兼容现有协议
//! - 降低把一种 ID 误传到另一种字段中的风险
//!
//! 所有 ID 均基于同一宏生成，保持实现一致且轻量。

use serde::{Deserialize, Serialize};

macro_rules! string_id {
    ($name:ident) => {
        /// 基于字符串的稳定标识符新类型。
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub String);

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self(value)
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self(value.to_owned())
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }
    };
}

string_id!(MessageId);
string_id!(ModelId);
string_id!(ProjectId);
string_id!(ProviderId);
string_id!(QuestionId);
string_id!(RequestId);
string_id!(SessionId);
string_id!(TaskId);
string_id!(TodoId);
string_id!(ToolId);
string_id!(WorktreeId);
