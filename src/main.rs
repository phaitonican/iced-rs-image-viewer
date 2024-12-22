use iced::widget::{column, pick_list, text};
use iced::widget::{row, Image};
use iced::{Element, Fill, Theme};

pub fn main() -> iced::Result {
    iced::application("Styling - Iced", Styling::update, Styling::view)
        .theme(Styling::theme)
        .run()
}

#[derive(Default)]
struct Styling {
    theme: Theme,
}

#[derive(Debug, Clone)]
enum Message {
    ThemeChanged(Theme),
}

impl Styling {
    fn update(&mut self, message: Message) {
        match message {
            Message::ThemeChanged(theme) => {
                self.theme = theme;
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let test: Image = Image::new("assets/ferris.png").height(50).width(50).into();
        let test2: Image = Image::new("assets/ferris.png").height(50).width(50).into();

        let choose_theme = column![
            text("Theme:"),
            pick_list(Theme::ALL, Some(&self.theme), Message::ThemeChanged).width(Fill),
        ]
        .spacing(10);

        let content = column![choose_theme, row![test, test2]];

        content.into()
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }
}
