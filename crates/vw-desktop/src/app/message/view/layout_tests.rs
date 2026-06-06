#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("layout_tests"));
}

#[test]
fn main_window_closed_exits_even_when_task_pet_is_open() {
    let main_window_id = iced::window::Id::unique();
    let task_pet_window_id = iced::window::Id::unique();
    let mut main_window = Some(main_window_id);
    let mut task_pet_window = Some(task_pet_window_id);

    let outcome = super::mark_window_closed(&mut main_window, &mut task_pet_window, main_window_id);

    assert_eq!(outcome, super::WindowClosedOutcome::Exit);
    assert_eq!(main_window, None);
    assert_eq!(task_pet_window, None);
}

#[test]
fn task_pet_window_closed_keeps_main_window_running() {
    let main_window_id = iced::window::Id::unique();
    let task_pet_window_id = iced::window::Id::unique();
    let mut main_window = Some(main_window_id);
    let mut task_pet_window = Some(task_pet_window_id);

    let outcome =
        super::mark_window_closed(&mut main_window, &mut task_pet_window, task_pet_window_id);

    assert_eq!(outcome, super::WindowClosedOutcome::Continue);
    assert_eq!(main_window, Some(main_window_id));
    assert_eq!(task_pet_window, None);
}
