//! ACP 代理进程组清理和退出摘要。

use super::*;

/// 清理代理进程组。
///
/// 会先发送温和终止信号并短暂等待，再发送强制结束信号；非 Unix 平台下信号
/// 函数为空实现，因此该函数主要依赖子进程自身退出。
pub(super) async fn cleanup_process_group(process_group_id: Option<u32>) {
    send_terminate_signal_to_process_group(process_group_id);
    tokio::time::sleep(Duration::from_millis(150)).await;
    send_kill_signal_to_process_group(process_group_id);
}

/// 向 Unix 进程组发送 `SIGTERM`。
///
/// `process_group_id` 为 `None` 时不会执行任何操作；发送失败会被忽略，因为
/// 进程可能已经退出。
#[cfg(unix)]
pub(super) fn send_terminate_signal_to_process_group(process_group_id: Option<u32>) {
    let _ = send_signal_to_process_group(process_group_id, libc::SIGTERM);
}

/// 非 Unix 平台的 `SIGTERM` 占位实现。
#[cfg(not(unix))]
pub(super) fn send_terminate_signal_to_process_group(_process_group_id: Option<u32>) {}

/// 向 Unix 进程组发送 `SIGKILL`。
///
/// 用于温和终止超时后的兜底清理，确保代理子进程不会继续持有工作区资源。
#[cfg(unix)]
pub(super) fn send_kill_signal_to_process_group(process_group_id: Option<u32>) {
    let _ = send_signal_to_process_group(process_group_id, libc::SIGKILL);
}

/// 非 Unix 平台的 `SIGKILL` 占位实现。
#[cfg(not(unix))]
pub(super) fn send_kill_signal_to_process_group(_process_group_id: Option<u32>) {}

#[cfg(unix)]
fn send_signal_to_process_group(process_group_id: Option<u32>, signal: i32) -> bool {
    let Some(process_group_id) = process_group_id else {
        return false;
    };
    let result = unsafe { libc::kill(-(process_group_id as i32), signal) };
    if result == 0 {
        return true;
    }
    matches!(std::io::Error::last_os_error().raw_os_error(), Some(libc::ESRCH))
}

/// 从子进程退出状态构造生命周期摘要。
///
/// 返回值包含退出码和 Unix 信号名；没有退出状态时字段为空。
pub(super) fn child_exit_summary(status: Option<&ExitStatus>) -> ChildExitSummary {
    ChildExitSummary {
        exit_code: status.and_then(ExitStatus::code),
        signal: exit_signal_name(status),
    }
}

#[cfg(unix)]
fn exit_signal_name(status: Option<&ExitStatus>) -> Option<String> {
    use std::os::unix::process::ExitStatusExt;

    status.and_then(|value| value.signal()).map(|signal| format!("SIG{signal}"))
}

#[cfg(not(unix))]
fn exit_signal_name(_status: Option<&ExitStatus>) -> Option<String> {
    None
}
