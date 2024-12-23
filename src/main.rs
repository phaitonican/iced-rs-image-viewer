use std::fs;
use std::path::PathBuf;

use iced::widget::{column, pick_list, scrollable, slider, text, Button, Column};
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

struct ImageViewer {
    theme: Theme,
    image_paths: Option<Vec<PathBuf>>,
    image_size: f32,
}

impl Default for ImageViewer {
    fn default() -> Self {
        Self {
            theme: Theme::default(),
            image_paths: Some(Vec::new()),
            image_size: 25.0,
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    ThemeChanged(Theme),
    ImageSizeChanged(f32),
    SelectFolder,
}

impl ImageViewer {
    fn update(&mut self, message: Message) {
        match message {
            Message::ThemeChanged(theme) => {
                self.theme = theme;
            }
            Message::ImageSizeChanged(value) => {
                self.image_size = value;
            }
            Message::SelectFolder => {
                let folder_paths = FileDialog::new()
                    //.add_filter("text", &["txt", "rs"])
                    //.add_filter("rust", &["rs", "toml"])
                    //.set_directory("/")
                    .pick_folders();

                // return if nothing selected
                if folder_paths.is_none() {
                    return;
                }

                //remove old image_paths (to avoid duplicate images)
                self.image_paths = Some(Vec::new());
                //update image paths
                for folder_path in folder_paths.as_ref().unwrap().iter() {
                    let paths = fs::read_dir(folder_path).unwrap();
                    for path in paths.into_iter() {
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
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let choose_theme = row![
            text("Theme:"),
            pick_list(Theme::ALL, Some(&self.theme), Message::ThemeChanged),
        ];

        let select_folder = Button::new("Select Folder").on_press(Message::SelectFolder);

        let image_size = row![
            text("Zoom:"),
            slider(0.0..=100.0, self.image_size, Message::ImageSizeChanged)
        ];

        // create images
        let mut images = Vec::new();
        if self.image_paths.is_some() {
            for image_path in self.image_paths.as_ref().unwrap().iter() {
                let image = Image::new(image_path)
                    //.height(self.image_size.powf(2.0))
                    .width(self.image_size.powf(2.0))
                    .into();
                images.push(image);
            }
        }
        let image_element = scrollable(Column::from_vec(images)).width(Fill);

        // create content
        let content = column![row![choose_theme, image_size, select_folder], image_element,];

        content.into()
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }
}
