use super::*;

pub(crate) fn classify_health_result(
    result: &std::result::Result<bool, tokio::time::error::Elapsed>,
) -> ChannelHealthState {
    match result {
        Ok(true) => ChannelHealthState::Healthy,
        Ok(false) => ChannelHealthState::Unhealthy,
        Err(_) => ChannelHealthState::Timeout,
    }
}

/// Run health checks for configured channels.
pub async fn doctor_channels(config: Config) -> Result<()> {
    let mut channels = collect_configured_channels(&config, "health check");
    let mut init_failures = Vec::new();

    if let Some(reason) =
        append_nostr_channel_if_available(&config, &mut channels, "health check").await
    {
        init_failures.push(reason);
    }

    if channels.is_empty() && init_failures.is_empty() {
        println!("No real-time channels configured.");
        return Ok(());
    }

    println!("🩺 VibeWindow Channel Doctor");
    println!();

    let mut healthy = 0_u32;
    let mut unhealthy = u32::try_from(init_failures.len()).unwrap_or(u32::MAX);
    let mut timeout = 0_u32;
    let has_runtime_channels = !channels.is_empty();

    for failure in &init_failures {
        println!("  ❌ {:<9} {failure}", "Nostr");
    }

    for configured in channels {
        let result =
            tokio::time::timeout(Duration::from_secs(10), configured.channel.health_check()).await;
        let state = classify_health_result(&result);

        match state {
            ChannelHealthState::Healthy => {
                healthy += 1;
                println!("  ✅ {:<9} healthy", configured.display_name);
            }
            ChannelHealthState::Unhealthy => {
                unhealthy += 1;
                println!("  ❌ {:<9} unhealthy (auth/config/network)", configured.display_name);
            }
            ChannelHealthState::Timeout => {
                timeout += 1;
                println!("  ⏱️  {:<9} timed out (>10s)", configured.display_name);
            }
        }
    }

    if config.channels_config.webhook.is_some() {
        println!("  ℹ️  Webhook   check via `vibewindow gateway` then GET /health");
    }

    if !has_runtime_channels && !init_failures.is_empty() {
        println!();
        anyhow::bail!("All configured channels failed during initialization.");
    }

    println!();
    println!("Summary: {healthy} healthy, {unhealthy} unhealthy, {timeout} timed out");
    Ok(())
}

#[cfg(test)]
#[path = "health_tests.rs"]
mod health_tests;
