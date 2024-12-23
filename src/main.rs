use iced::widget::image::Handle;
use iced::widget::{column, pick_list, scrollable, text, Button, Column};
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
    thumbnail_handles: Option<Vec<Handle>>,
}

impl Default for ImageViewer {
    fn default() -> Self {
        Self {
            theme: Theme::default(),
            image_paths: Some(Vec::new()),
            thumbnail_size: 200,
            thumbnail_handles: Some(Vec::new()),
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
                //self.recreate_images();
            }
            Message::SelectFolder => {
                let folder_paths = FileDialog::new()
                    //.add_filter("text", &["txt", "rs"])
                    //.add_filter("rust", &["rs", "toml"])
                    //.set_directory("/")
                    .pick_folders();

                self.get_image_paths(folder_paths);
                self.recreate_images();
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
            let paths_inside_folder = fs::read_dir(folder_path).unwrap();
            for path in paths_inside_folder.into_iter() {
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

    fn recreate_images(&mut self) {
        // create images
        self.thumbnail_handles = Some(Vec::new());
        if self.image_paths.is_some() {
            for image_path in self.image_paths.as_ref().unwrap().iter() {
                let image = ImageReader::open(&image_path).unwrap().decode().unwrap();
                let thumbnail = image.thumbnail(200, 200).into_rgba8();
                let thumbnail_handle =
                    Handle::from_rgba(thumbnail.width(), thumbnail.height(), thumbnail.into_raw());

                self.thumbnail_handles
                    .as_mut()
                    .unwrap()
                    .push(thumbnail_handle);
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let choose_theme = row![
            text("Theme:"),
            pick_list(Theme::ALL, Some(&self.theme), Message::ThemeChanged),
        ];

        let select_folder = Button::new("Select Folder").on_press(Message::SelectFolder);
        let zoom_in = Button::new("+").on_press(Message::ThumbnailSizeChanged(50));
        let zoom_out = Button::new("-").on_press(Message::ThumbnailSizeChanged(-50));

        let mut image_elements = Vec::new();
        for thumbnail_handle in self.thumbnail_handles.as_ref().unwrap() {
            let image_element = Image::new(thumbnail_handle).width(self.thumbnail_size as f32);
            image_elements.push(image_element.into());
        }
        let processed_images_element = scrollable(Column::from_vec(image_elements)).width(Fill);

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
