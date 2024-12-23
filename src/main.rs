use std::fs;
use std::path::PathBuf;

use iced::widget::{column, pick_list, slider, text, text_input, Button, Row};
use iced::widget::{row, Image};
use iced::{Element, Fill, Theme};
use rfd::FileDialog;

pub fn main() -> iced::Result {
    iced::application(
        "Image Viewer - Iced",
        ImageViewer::update,
        ImageViewer::view,
    )
    .theme(ImageViewer::theme)
    .run()
}

#[derive(Default)]
struct ImageViewer {
    theme: Theme,
    input_value: String,
    folder_paths: Option<Vec<PathBuf>>,
    image_paths: Option<Vec<PathBuf>>,
    image_size: f32,
}

#[derive(Debug, Clone)]
enum Message {
    ThemeChanged(Theme),
    InputChanged(String),
    SelectFolder,
    ImageSizeChanged(f32),
}

impl ImageViewer {
    fn update(&mut self, message: Message) {
        match message {
            Message::ThemeChanged(theme) => {
                self.theme = theme;
            }
            Message::InputChanged(value) => self.input_value = value,
            Message::SelectFolder => {
                self.folder_paths = FileDialog::new()
                    //.add_filter("text", &["txt", "rs"])
                    //.add_filter("rust", &["rs", "toml"])
                    //.set_directory("/")
                    .pick_folders();

                // return if nothing selected
                if self.folder_paths.is_none() {
                    return;
                }

                self.image_paths = Some(Vec::new());

                //update textbox
                let mut folder_paths_string = "".to_owned();
                for folder_path in self.folder_paths.as_ref().unwrap() {
                    folder_paths_string.push_str(", ");
                    folder_paths_string
                        .push_str(folder_path.as_os_str().to_str().unwrap_or_default());

                    //get image file paths
                    let paths = fs::read_dir(folder_path).unwrap();

                    for path in paths {
                        if path
                            .as_ref()
                            .unwrap()
                            .path()
                            .extension()
                            .unwrap_or_default()
                            == "png"
                        {
                            self.image_paths
                                .as_mut()
                                .unwrap()
                                .push(path.unwrap().path());
                        }
                    }
                }
                self.input_value = folder_paths_string;
            }
            Message::ImageSizeChanged(value) => {
                self.image_size = value;
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let text_input =
            text_input("path to images...", &self.input_value).on_input(Message::InputChanged);

        let choose_theme = row![
            text("Theme:"),
            pick_list(Theme::ALL, Some(&self.theme), Message::ThemeChanged).width(Fill),
        ];

        let button = Button::new("Select Folder").on_press(Message::SelectFolder);

        let slider = slider(0.0..=100.0, self.image_size, Message::ImageSizeChanged);

        let mut image_elements = Vec::new();

        if self.image_paths.is_some() {
            for image_path in self.image_paths.as_ref().unwrap() {
                let image_element = Element::new(
                    Image::new(image_path)
                        .height(self.image_size.powf(2.0))
                        .width(self.image_size.powf(2.0)),
                );
                image_elements.push(image_element);
            }
        }

        let content = column![
            row![choose_theme, text_input, button],
            row![slider],
            Row::from_vec(image_elements)
        ];

        content.into()
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }
}
