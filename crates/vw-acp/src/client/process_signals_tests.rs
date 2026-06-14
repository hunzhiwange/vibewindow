use std::process::Command;

use super::{
    child_exit_summary, cleanup_process_group, send_kill_signal_to_process_group,
    send_terminate_signal_to_process_group,
};

#[test]
fn child_exit_summary_reports_exit_code_without_signal() {
    let status =
        Command::new("sh").arg("-c").arg("exit 7").status().expect("shell exits with code");

    let summary = child_exit_summary(Some(&status));

    assert_eq!(summary.exit_code, Some(7));
    assert_eq!(summary.signal, None);
}

#[cfg(unix)]
#[test]
fn child_exit_summary_reports_unix_signal_name() {
    let status = Command::new("sh")
        .arg("-c")
        .arg("kill -TERM $$")
        .status()
        .expect("shell exits from signal");

    let summary = child_exit_summary(Some(&status));

    assert_eq!(summary.exit_code, None);
    assert_eq!(summary.signal.as_deref(), Some("SIG15"));
}

#[cfg(unix)]
#[test]
fn process_group_signal_helpers_ignore_empty_and_missing_targets() {
    send_terminate_signal_to_process_group(None);
    send_kill_signal_to_process_group(None);

    let missing_process_group = i32::MAX as u32;
    send_terminate_signal_to_process_group(Some(missing_process_group));
    send_kill_signal_to_process_group(Some(missing_process_group));
}

#[cfg(unix)]
#[tokio::test]
async fn cleanup_process_group_kills_term_ignoring_process_group() {
    use std::os::unix::process::{CommandExt, ExitStatusExt};
    use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
    let ready_path = std::env::temp_dir()
        .join(format!("vw-acp-term-ignore-ready-{}-{nanos}", std::process::id()));
    let mut command = Command::new("perl");
    command
        .arg("-e")
        .arg(
            "$SIG{TERM} = 'IGNORE'; open my $fh, '>', $ARGV[0] or die $!; close $fh; select undef, undef, undef, 10 while 1;",
        )
        .arg(&ready_path);
    unsafe {
        command.pre_exec(|| {
            if libc::setpgid(0, 0) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
    let mut child = command.spawn().expect("spawn term-ignoring process group");
    let process_group_id = child.id();

    let ready_deadline = Instant::now() + Duration::from_secs(2);
    while !ready_path.exists() {
        if Instant::now() >= ready_deadline {
            let _ = child.kill();
            panic!("process did not install TERM handler");
        }
        std::thread::sleep(Duration::from_millis(10));
    }

    cleanup_process_group(Some(process_group_id)).await;
    let _ = std::fs::remove_file(&ready_path);

    let deadline = Instant::now() + Duration::from_secs(2);
    let status = loop {
        if let Some(status) = child.try_wait().expect("poll child status") {
            break status;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            panic!("process group was not cleaned up");
        }
        std::thread::sleep(Duration::from_millis(10));
    };

    assert_eq!(status.signal(), Some(libc::SIGKILL));
}
