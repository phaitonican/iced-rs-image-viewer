use iced::widget::image::Handle;
use iced::widget::{column, container, pick_list, progress_bar, scrollable, slider, Button, Row};
use iced::widget::{row, Image};
use iced::window::{self, Id};
use iced::{Alignment, Element, Fill, Size, Subscription, Task, Theme};
use image::ImageReader;
use mime_guess::{mime, MimeGuess};
use rfd::FileHandle;
use std::path::PathBuf;
use std::{fs, io};

pub fn main() -> iced::Result {
    iced::application(
        "Image Viewer - Iced",
        ImageViewer::update,
        ImageViewer::view,
    )
    .subscription(ImageViewer::subscription)
    .theme(ImageViewer::theme)
    .run_with(ImageViewer::new)
}

const THUMBNAIL_WIDTH: f32 = 500.0;
const THUMBNAIL_HEIGHT: f32 = 500.0;
const MIN_SPACING: f32 = 10.0;
const SLIDER_STEPS: f32 = 100.0;

#[derive(Debug, Clone)]
pub enum Error {
    DialogClosed,
    IoError(io::ErrorKind),
}

struct ImageViewer {
    theme: Theme,
    zoom_factor: f32,
    thumbnail_handles: Option<Vec<Handle>>,
    main_window_id: Option<Id>,
    image_count: usize,
    image_load_abort_handle: Option<iced::task::Handle>,
}

impl Default for ImageViewer {
    fn default() -> Self {
        Self {
            theme: Theme::default(),
            zoom_factor: SLIDER_STEPS / 2.0, //half zoom
            thumbnail_handles: Some(Vec::new()),
            main_window_id: None,
            image_count: 0,
            image_load_abort_handle: None,
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    ThemeChanged(Theme),
    ZoomFactorChanged(f32),
    SelectFolders,
    FoldersOpened(Result<Vec<FileHandle>, Error>),
    ImageLoaded(Result<Handle, Error>),
    WindowResized(Id, Size),
    SetMainWindowID(Id),
}

async fn open_folders() -> Result<Vec<FileHandle>, Error> {
    let picked_folders = rfd::AsyncFileDialog::new()
        .set_title("Open (multiple) folders...")
        .pick_folders()
        .await;
    Ok(picked_folders.unwrap_or_default())
}

fn get_image_paths(folder_paths: &Vec<FileHandle>) -> Vec<PathBuf> {
    let mut image_paths = Vec::new();

    //update image paths
    for folder_path in folder_paths.iter() {
        let paths_inside_folder_result = fs::read_dir(folder_path.path()).unwrap();
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
    let dynamic_image = ImageReader::open(image_path)
        .unwrap()
        .decode()
        .unwrap_or_default();
    let thumbnail = dynamic_image
        .thumbnail(THUMBNAIL_WIDTH as u32, THUMBNAIL_HEIGHT as u32)
        .into_rgba8();
    let thumbnail_handle =
        Handle::from_rgba(thumbnail.width(), thumbnail.height(), thumbnail.into_raw());

    Ok(thumbnail_handle)
}

impl ImageViewer {
    fn subscription(&self) -> Subscription<Message> {
        window::resize_events().map(|(id, size)| Message::WindowResized(id, size))
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ThemeChanged(theme) => {
                self.theme = theme;
                Task::none()
            }
            Message::ZoomFactorChanged(value) => {
                self.zoom_factor = value;
                Task::none()
            }
            Message::SelectFolders => {
                return Task::perform(open_folders(), Message::FoldersOpened);
            }
            Message::FoldersOpened(result) => {
                if let Ok(folder_paths) = result {
                    //abort last tasks
                    if let Some(last_abort_handle) = self.image_load_abort_handle.as_ref() {
                        last_abort_handle.abort();
                    }

                    //continue
                    let image_paths = get_image_paths(&folder_paths);
                    self.image_count = image_paths.len();

                    //remove old thumbnail handles
                    if !&folder_paths.is_empty() {
                        self.thumbnail_handles = Some(Vec::new());
                    }

                    let mut tasks = vec![];
                    for image_path in image_paths.iter() {
                        let task =
                            Task::perform(recreate_image(image_path.into()), Message::ImageLoaded);
                        tasks.push(task);
                    }

                    //let task = Task::perform(recreate_images(image_paths), Message::ImagesLoaded);
                    let (summarized_task, abort_handle) = Task::abortable(Task::batch(tasks)); //returns this
                    self.image_load_abort_handle = Some(abort_handle);
                    summarized_task
                } else {
                    return Task::none();
                }
            }
            Message::ImageLoaded(result) => {
                if let Ok(handle) = result {
                    self.thumbnail_handles.as_mut().unwrap().push(handle);
                }
                Task::none()
            }
            Message::WindowResized(_id, _size) => Task::none(),
            Message::SetMainWindowID(id) => {
                self.main_window_id = Some(id);
                Task::none()
            }
        }
    }

    fn new() -> (Self, Task<Message>) {
        (
            Self::default(),
            window::get_oldest().and_then(|id| Task::done(Message::SetMainWindowID(id))),
        )
    }

    fn view(&self) -> Element<Message> {
        let choose_theme = pick_list(Theme::ALL, Some(&self.theme), Message::ThemeChanged);

        let select_folder = Button::new("Select Folder").on_press(Message::SelectFolders);
        let zoom_slider = slider(
            1.0..=SLIDER_STEPS,
            self.zoom_factor,
            Message::ZoomFactorChanged,
        )
        .width(300);

        // thumbnails
        let thumbnail_handles = self.thumbnail_handles.as_ref().unwrap();

        let mut thumbnail_images = Vec::new();
        for thumbnail_handle in thumbnail_handles {
            let t_width = THUMBNAIL_WIDTH * self.zoom_factor / SLIDER_STEPS;
            let t_height = THUMBNAIL_HEIGHT * self.zoom_factor / SLIDER_STEPS;
            let image: Image = Image::new(thumbnail_handle).width(t_width).height(t_height);
            thumbnail_images.push(image.into());
        }

        let rows_for_thumbnails = Row::from_vec(thumbnail_images)
            .spacing(MIN_SPACING)
            .align_y(Alignment::Center)
            .wrap();

        let scrollable_rows_for_thumbnails =
            scrollable(container(rows_for_thumbnails).center_x(Fill))
                .width(Fill)
                .height(Fill);

        let toolbar = row![select_folder, choose_theme, zoom_slider,]
            .spacing(MIN_SPACING)
            .padding(MIN_SPACING)
            .align_y(Alignment::Center)
            .wrap();

        let progress_bar = row![progress_bar(
            0.0..=self.image_count as f32,
            self.thumbnail_handles.as_ref().unwrap().len() as f32
        )];

        // create content
        let content = column![toolbar, scrollable_rows_for_thumbnails, progress_bar];

        content.into()
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }
}
