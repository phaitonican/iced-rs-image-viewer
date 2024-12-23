use iced::widget::{self, column, pick_list, scrollable, text, Button, Column, Space};
use iced::widget::{row, Image};
use iced::{Element, Fill, Theme};
use image::ImageReader;
use rfd::FileDialog;
use std::fs;
use std::path::PathBuf;

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
    thumbnail_size: i32,
}

impl Default for ImageViewer {
    fn default() -> Self {
        Self {
            theme: Theme::default(),
            image_paths: Some(Vec::new()),
            thumbnail_size: 200,
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    ThemeChanged(Theme),
    ThumbnailSizeChanged(i32),
    SelectFolder,
}

impl ImageViewer {
    fn update(&mut self, message: Message) {
        match message {
            Message::ThemeChanged(theme) => {
                self.theme = theme;
            }
            Message::ThumbnailSizeChanged(value) => {
                self.thumbnail_size += value;
            }
            Message::SelectFolder => {
                let folder_paths = FileDialog::new()
                    //.add_filter("text", &["txt", "rs"])
                    //.add_filter("rust", &["rs", "toml"])
                    //.set_directory("/")
                    .pick_folders();

                self.get_image_paths(folder_paths);
            }
        }
    }

    fn get_image_paths(&mut self, folder_paths: Option<Vec<PathBuf>>) {
        // return if empty folder_paths
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

    fn create_image_widget(&self) -> Element<Message> {
        // create images
        let mut processed_images = Vec::new();
        if self.image_paths.is_some() {
            for image_path in self.image_paths.as_ref().unwrap().iter() {
                let image = ImageReader::open(&image_path).unwrap().decode().unwrap();
                let thumbnail = image
                    .thumbnail(self.thumbnail_size as u32, self.thumbnail_size as u32)
                    .into_rgba8();
                let thumbnail_widget = widget::image::Handle::from_rgba(
                    thumbnail.width(),
                    thumbnail.height(),
                    thumbnail.into_raw(),
                );

                let processed_image = Image::new(thumbnail_widget).into();
                processed_images.push(processed_image);
            }
        }
        let processed_images_element = scrollable(Column::from_vec(processed_images)).width(Fill);
        processed_images_element.into()
    }

    fn view(&self) -> Element<Message> {
        let choose_theme = row![
            text("Theme:"),
            pick_list(Theme::ALL, Some(&self.theme), Message::ThemeChanged),
        ];

        let select_folder = Button::new("Select Folder").on_press(Message::SelectFolder);
        let zoom_in = Button::new("+").on_press(Message::ThumbnailSizeChanged(50));
        let zoom_out = Button::new("-").on_press(Message::ThumbnailSizeChanged(-50));

        // create images
        let processed_images_element = self.create_image_widget();

        // create content
        let content = column![
            row![choose_theme, select_folder, zoom_in, zoom_out],
            processed_images_element,
        ];

        content.into()
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }
}
