use iced::widget::image::Handle;
use iced::widget::{column, pick_list, scrollable, slider, text, Button, Column};
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

const DEFAULT_THUMBNAIL_WIDTH: f32 = 500.0;
const DEFAULT_THUMBNAIL_HEIGHT: f32 = 500.0;

struct ImageViewer {
    theme: Theme,
    zoom_factor: f32,
    thumbnail_handles: Option<Vec<Handle>>,
}

impl Default for ImageViewer {
    fn default() -> Self {
        Self {
            theme: Theme::default(),
            zoom_factor: 5.0, //gets divided by 10 to have smaller steps in the slider
            thumbnail_handles: Some(Vec::new()),
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    ThemeChanged(Theme),
    ZoomFactorChanged(f32),
    SelectFolder,
}

impl ImageViewer {
    fn update(&mut self, message: Message) {
        match message {
            Message::ThemeChanged(theme) => {
                self.theme = theme;
            }
            Message::ZoomFactorChanged(value) => {
                self.zoom_factor = value;
                //self.recreate_images();
            }
            Message::SelectFolder => {
                let folder_paths = FileDialog::new()
                    //.add_filter("text", &["txt", "rs"])
                    //.add_filter("rust", &["rs", "toml"])
                    //.set_directory("/")
                    .pick_folders();

                let image_paths = self.get_image_paths(folder_paths);
                self.recreate_images(image_paths);
            }
        }
    }

    fn get_image_paths(&mut self, folder_paths: Option<Vec<PathBuf>>) -> Option<Vec<PathBuf>> {
        let mut image_paths = Some(Vec::new());
        // return if empty folder_paths
        if folder_paths.is_none() {
            return image_paths;
        }
        //remove old image_paths (to avoid duplicate images)
        image_paths = Some(Vec::new());
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
                    image_paths.as_mut().unwrap().push(path.unwrap().path());
                }
            }
        }
        return image_paths;
    }

    fn recreate_images(&mut self, image_paths: Option<Vec<PathBuf>>) {
        // create images
        self.thumbnail_handles = Some(Vec::new());
        if image_paths.is_some() {
            for image_path in image_paths.as_ref().unwrap().iter() {
                let image = ImageReader::open(&image_path).unwrap().decode().unwrap();
                let thumbnail = image
                    .thumbnail(
                        DEFAULT_THUMBNAIL_WIDTH as u32,
                        DEFAULT_THUMBNAIL_HEIGHT as u32,
                    )
                    .into_rgba8();
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
        let zoom_slider = slider(1.0..=10.0, self.zoom_factor, Message::ZoomFactorChanged);

        let mut image_elements = Vec::new();
        for thumbnail_handle in self.thumbnail_handles.as_ref().unwrap() {
            let image_element = Image::new(thumbnail_handle)
                .width(DEFAULT_THUMBNAIL_WIDTH * self.zoom_factor / 10.0);
            image_elements.push(image_element.into());
        }
        let processed_images_element = scrollable(Column::from_vec(image_elements)).width(Fill);

        // create content
        let content = column![
            row![choose_theme, zoom_slider, select_folder],
            processed_images_element,
        ];

        content.into()
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }
}
