# 添加组件

使用 [column!](https://docs.rs/iced/0.13.1/iced/widget/macro.column.html) 和 [row!](https://docs.rs/iced/0.13.1/iced/widget/macro.row.html) 将多个组件（如 [text](https://docs.rs/iced/0.13.1/iced/widget/fn.text.html) 和 [button](https://docs.rs/iced/0.13.1/iced/widget/fn.button.html)）组合在一起。

```rust
use iced::widget::{button, column, row, text};

fn main() -> iced::Result {
    iced::run("MyApp", MyApp::update, MyApp::view)
}

#[derive(Default)]
struct MyApp;

#[derive(Debug, Clone)]
enum Message {}

impl MyApp {
    fn update(&mut self, _message: Message) {}

    fn view(&self) -> iced::Element<Message> {
        column![text("Yes or no?"), row![button("Yes"), button("No"),],].into()
    }
}
```

![添加组件](./pic/adding_widgets.png)

:arrow_right: 下一步: [组件](./widgets.md)

:blue_book: 返回: [目录](./../README.md)
