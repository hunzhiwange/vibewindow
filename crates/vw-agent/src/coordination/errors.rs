use thiserror::Error;

use crate::app::agent::coordination::envelope::DeliveryScope;

/// Errors emitted by the coordination protocol and message bus.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CoordinationError {
    #[error("field `{field}` must not be empty")]
    EmptyField { field: &'static str },
    #[error("message `{message_id}` requires a direct target agent")]
    MissingTarget { message_id: String },
    #[error("broadcast message `{message_id}` cannot set explicit target")]
    BroadcastHasTarget { message_id: String },
    #[error("task result message `{message_id}` requires `correlation_id`")]
    MissingCorrelationId { message_id: String },
    #[error(
        "invalid delivery scope for payload `{payload}` on message `{message_id}`: expected {expected:?}, got {actual:?}"
    )]
    InvalidDeliveryScope {
        message_id: String,
        expected: DeliveryScope,
        actual: DeliveryScope,
        payload: String,
    },
    #[error("duplicate message id `{message_id}`")]
    DuplicateMessageId { message_id: String },
    #[error("unknown target agent `{agent}` for message `{message_id}`")]
    UnknownTarget { agent: String, message_id: String },
    #[error("agent `{agent}` is not registered")]
    UnknownAgent { agent: String },
    #[error("invalid delegate context key `{key}` on message `{message_id}`")]
    InvalidDelegateContextKey { key: String, message_id: String },
    #[error("delegate context key `{key}` requires `correlation_id` on message `{message_id}`")]
    MissingDelegateContextCorrelation { key: String, message_id: String },
    #[error(
        "delegate context key `{key}` correlation mismatch on message `{message_id}`: key has `{key_correlation_id}`, envelope has `{envelope_correlation_id}`"
    )]
    DelegateContextCorrelationMismatch {
        key: String,
        message_id: String,
        key_correlation_id: String,
        envelope_correlation_id: String,
    },
    #[error("context version mismatch for key `{key}`: expected {expected}, actual {actual}")]
    ContextVersionMismatch { key: String, expected: u64, actual: u64 },
}
