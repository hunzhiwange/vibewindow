#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("settings_tests"));
}

#[test]
fn main_window_close_request_closes_task_pet_too() {
    let main_window_id = iced::window::Id::unique();
    let task_pet_window_id = iced::window::Id::unique();

    let windows = super::close_requested_windows(
        Some(main_window_id),
        Some(task_pet_window_id),
        main_window_id,
    );

    assert_eq!(windows.main_window_id, Some(main_window_id));
    assert_eq!(windows.task_pet_window_id, Some(task_pet_window_id));
}

#[test]
fn task_pet_close_request_only_closes_task_pet() {
    let main_window_id = iced::window::Id::unique();
    let task_pet_window_id = iced::window::Id::unique();

    let windows = super::close_requested_windows(
        Some(main_window_id),
        Some(task_pet_window_id),
        task_pet_window_id,
    );

    assert_eq!(windows.main_window_id, None);
    assert_eq!(windows.task_pet_window_id, Some(task_pet_window_id));
}
