//! 桌面应用入口模块，负责启动 iced 应用并注册窗口、字体、主题和渲染器配置。

#![cfg_attr(all(target_os = "windows", not(debug_assertions)), windows_subsystem = "windows")]

use vw_desktop::{app, fonts};

/// 启动桌面应用。
///
/// 返回 iced 运行结果，窗口或渲染器初始化失败时将错误交给进程入口处理。
pub fn main() -> iced::Result {
    #[cfg(target_arch = "wasm32")]
    {
        console_log::init().expect("Initialize logger");
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let filter = tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            tracing_subscriber::EnvFilter::new(
                "info,iced_winit=error,iced_futures::subscription::tracker=error",
            )
        });
        tracing_subscriber::fmt().with_env_filter(filter).init();
    }

    let default_font = {
        #[cfg(target_os = "macos")]
        {
            iced::Font::with_name("PingFang SC")
        }

        #[cfg(not(target_os = "macos"))]
        {
            iced::Font::with_name("JetBrains Mono")
        }
    };

    let settings = iced::Settings { default_font, fonts: fonts::load_all(), ..Default::default() };

    let platform_specific = {
        #[cfg(target_os = "macos")]
        {
            iced::window::settings::PlatformSpecific {
                title_hidden: true,
                titlebar_transparent: true,
                fullsize_content_view: true,
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            iced::window::settings::PlatformSpecific::default()
        }
    };

    let window = iced::window::Settings {
        size: iced::Size::new(1720.0, 1000.0),
        maximized: cfg!(not(target_arch = "wasm32")),
        platform_specific,
        ..Default::default()
    };

    #[cfg(target_arch = "wasm32")]
    let window = {
        let mut window = window;
        window.platform_specific.target = Some("vibe-window-app-container".to_string());
        window
    };

    iced::application(app::App::new, app::App::update, app::App::view)
        .title(app::App::title)
        .theme(app::App::theme)
        .subscription(app::App::subscription)
        .window(window)
        .settings(settings)
        .centered()
        .run()
}

#[cfg(test)]
#[path = "main_tests.rs"]
mod main_tests;
