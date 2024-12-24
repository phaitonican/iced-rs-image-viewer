use iced::widget::image::Handle;
use iced::widget::{column, container, pick_list, scrollable, slider, text, Button, Row};
use iced::widget::{row, Image};
use iced::{Element, Fill, Task, Theme};
use image::ImageReader;
use mime_guess::{mime, MimeGuess};
use rfd::FileDialog;
use std::path::PathBuf;
use std::{fs, io};

pub fn main() -> iced::Result {
    iced::application(
        "Image Viewer - Iced",
        ImageViewer::update,
        ImageViewer::view,
    )
    .theme(ImageViewer::theme)
    .run()
}

const THUMBNAIL_WIDTH: f32 = 500.0;
const THUMBNAIL_HEIGHT: f32 = 500.0;
const SPACING: u16 = 10;

#[derive(Debug, Clone)]
pub enum Error {
    DialogClosed,
    IoError(io::ErrorKind),
}

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
    ImageLoaded(Result<Handle, Error>),
}

fn get_image_paths(folder_paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut image_paths = Vec::new();

    //update image paths
    for folder_path in folder_paths.iter() {
        let paths_inside_folder_result = fs::read_dir(folder_path).unwrap();
        for path_result in paths_inside_folder_result.into_iter() {
            let path = path_result.unwrap().path();
            //println!("{:?}", &top_level_mime_type);
            if MimeGuess::from_path(&path)
                .first()
                .as_ref()
                .unwrap()
                .type_()
                == mime::IMAGE
            {
                image_paths.push(path);
            }
        }
    }
    return image_paths;
}

async fn recreate_image(image_path: PathBuf) -> Result<Handle, Error> {
    let dynamic_image = ImageReader::open(image_path).unwrap().decode().unwrap();
    let thumbnail = dynamic_image
        .thumbnail(THUMBNAIL_WIDTH as u32, THUMBNAIL_HEIGHT as u32)
        .into_rgba8();
    let thumbnail_handle =
        Handle::from_rgba(thumbnail.width(), thumbnail.height(), thumbnail.into_raw());

    Ok(thumbnail_handle)
}
impl ImageViewer {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ThemeChanged(theme) => {
                self.theme = theme;
                Task::none()
            }
            Message::ZoomFactorChanged(value) => {
                self.zoom_factor = value;
                //self.recreate_images();
                Task::none()
            }
            Message::SelectFolder => {
                let folder_paths = FileDialog::new()
                    //.add_filter("text", &["txt", "rs"])
                    //.add_filter("rust", &["rs", "toml"])
                    //.set_directory("/")
                    .pick_folders();

                let image_paths = get_image_paths(folder_paths.unwrap_or_default());

                //remove old thumbnail handles
                self.thumbnail_handles = Some(Vec::new());

                let mut tasks = Vec::new();
                for image_path in image_paths.iter() {
                    let task =
                        Task::perform(recreate_image(image_path.into()), Message::ImageLoaded);
                    tasks.push(task);
                }

                //let task = Task::perform(recreate_images(image_paths), Message::ImagesLoaded);
                Task::batch(tasks) //returns this
            }
            Message::ImageLoaded(result) => {
                if let Ok(handle) = result {
                    self.thumbnail_handles.as_mut().unwrap().push(handle);
                }
                Task::none()
            } //Message::SetMainWindowID(id) => {
              //self.main_window_id = Some(id);
              //Task::none()
              //}
        }
    }

    fn view(&self) -> Element<Message> {
        let choose_theme = row![
            text("Theme:"),
            pick_list(Theme::ALL, Some(&self.theme), Message::ThemeChanged),
        ];

        let select_folder = Button::new("Select Folder").on_press(Message::SelectFolder);
        let zoom_slider = slider(1.0..=10.0, self.zoom_factor, Message::ZoomFactorChanged);

        let thumbnail_handles = self.thumbnail_handles.as_ref().unwrap();
        let chunked_thumbnail_handles: Vec<&[Handle]> = thumbnail_handles.chunks(3).collect();

        let mut rows = column![].spacing(SPACING);
        for thumbnail_handle_chunk in chunked_thumbnail_handles {
            let mut row_images = Vec::new();
            for thumbnail_handle in thumbnail_handle_chunk {
                let image_element: Image =
                    Image::new(thumbnail_handle).width(THUMBNAIL_WIDTH * self.zoom_factor / 10.0);
                row_images.push(image_element.into());
            }
            let row = Row::from_vec(row_images).spacing(SPACING);
            rows = rows.push(row);
        }

        let scrollable_rows = scrollable(container(rows).center_x(Fill)).width(Fill);

        // create content
        let content = column![
            row![choose_theme, zoom_slider, select_folder],
            scrollable_rows,
        ];

        content.into()
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }
}
