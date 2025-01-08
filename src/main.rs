use iced::widget::image::Handle;
use iced::widget::{
    checkbox, column, container, pick_list, progress_bar, scrollable, slider, stack, text, Button,
};
use iced::widget::{row, Image};
use iced::window::{self, Id};
use iced::{Alignment, Element, Fill, Size, Subscription, Task, Theme};
use iced_aw::Wrap;
use image::ImageReader;
use mime_guess::{mime, MimeGuess};
use rfd::FileHandle;
use std::path::PathBuf;
use std::{fs, io};

pub fn main() -> iced::Result {
    iced::application("Image Viewer", ImageViewer::update, ImageViewer::view)
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

#[derive(Debug, Clone)]
struct ImageInfo {
    image_handle: Handle,
    image_path_buf: PathBuf,
}

struct ImageViewer {
    theme: Theme,
    zoom_factor: f32,
    image_infos: Option<Vec<ImageInfo>>,
    main_window_id: Option<Id>,
    image_count: usize,
    image_load_abort_handle: Option<iced::task::Handle>,
    image_loaded: Option<PathBuf>,
    append_checkbox_value: bool,
}

impl Default for ImageViewer {
    fn default() -> Self {
        Self {
            theme: Theme::Dark,
            zoom_factor: SLIDER_STEPS / 2.0, //half zoom
            image_infos: Some(Vec::default()),
            main_window_id: None,
            image_count: usize::default(),
            image_load_abort_handle: None,
            image_loaded: None,
            append_checkbox_value: false,
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    ThemeChanged(Theme),
    ZoomFactorChanged(f32),
    SelectFolders,
    FoldersOpened(Result<Vec<FileHandle>, Error>),
    ImageLoaded(Result<ImageInfo, Error>),
    WindowResized(Id, Size),
    SetMainWindowID(Id),
    ImageClicked(PathBuf),
    AppendCheckboxToggled(bool),
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

async fn recreate_image(image_path_buf: PathBuf) -> Result<ImageInfo, Error> {
    let image = ImageReader::open(&image_path_buf)
        .unwrap()
        .decode()
        .unwrap_or_default();

    let thumbnail_image = image
        .thumbnail(THUMBNAIL_WIDTH as u32, THUMBNAIL_HEIGHT as u32)
        .into_rgba8();

    let image_handle = Handle::from_rgba(
        thumbnail_image.width(),
        thumbnail_image.height(),
        thumbnail_image.into_raw(),
    );

    let image_info = ImageInfo {
        image_handle,
        image_path_buf,
    };

    Ok(image_info)
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
                    if !self.append_checkbox_value && !&folder_paths.is_empty() {
                        self.image_infos = Some(Vec::new());
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
                if let Ok(image_info) = result {
                    self.image_loaded = Some(image_info.image_path_buf.clone());
                    self.image_infos.as_mut().unwrap().push(image_info);
                }
                Task::none()
            }
            Message::WindowResized(_id, _size) => Task::none(),
            Message::SetMainWindowID(id) => {
                self.main_window_id = Some(id);
                Task::none()
            }
            Message::ImageClicked(image_path_buf) => {
                println!("{:?}", image_path_buf);

                /*
                let file = std::fs::File::open(image_path_buf.as_path()).unwrap();
                let mut bufreader = std::io::BufReader::new(&file);

                println!("{:?}", bufreader);
                let exifreader = exif::Reader::new();
                let exif = exifreader.read_from_container(&mut bufreader).unwrap();
                for f in exif.fields() {
                    println!(
                        "{} {} {}",
                        f.tag,
                        f.ifd_num,
                        f.display_value().with_unit(&exif)
                    );
                }
                */

                Task::none()
            }
            Message::AppendCheckboxToggled(value) => {
                self.append_checkbox_value = value;
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

        let append_checkbox = checkbox("Append images", self.append_checkbox_value)
            .on_toggle(Message::AppendCheckboxToggled);

        // thumbnails
        let image_infos = self.image_infos.as_ref().unwrap();
        let mut thumbnail_image_buttons = Vec::new();

        for image_info in image_infos {
            let t_width = THUMBNAIL_WIDTH * self.zoom_factor / SLIDER_STEPS;
            let t_height = THUMBNAIL_HEIGHT * self.zoom_factor / SLIDER_STEPS;
            let image: Image = Image::new(&image_info.image_handle)
                .width(t_width)
                .height(t_height);

            let image_button = Button::new(image)
                .on_press(Message::ImageClicked(image_info.image_path_buf.clone()));

            thumbnail_image_buttons.push(image_button.into());
        }

        let thumbnails_wrap = Wrap::with_elements(thumbnail_image_buttons)
            .align_items(Alignment::Center)
            .spacing(MIN_SPACING)
            .line_spacing(MIN_SPACING);

        let scrollable_rows_for_thumbnails = scrollable(container(thumbnails_wrap).center_x(Fill))
            .width(Fill)
            .height(Fill);

        let toolbar = row![select_folder, choose_theme, append_checkbox, zoom_slider,]
            .spacing(MIN_SPACING)
            .padding(MIN_SPACING)
            .align_y(Alignment::Center)
            .wrap();

        let loaded_images = self.image_infos.as_ref().unwrap().len();
        let mut loading_message = String::default();
        if let Some(image_pathbuf) = &self.image_loaded {
            let loaded_image_path = image_pathbuf.to_str().unwrap_or_default().to_string();
            let counter_string = "(".to_owned()
                + &loaded_images.to_string()
                + "/"
                + &self.image_count.to_string()
                + ")";
            loading_message =
                "Loaded image: ".to_owned() + &loaded_image_path + " " + &counter_string;
        }

        let progress_bar = stack![
            progress_bar(0.0..=self.image_count as f32, loaded_images as f32),
            text(loading_message)
                .height(Fill)
                .width(Fill)
                .align_x(Alignment::Center)
                .align_y(Alignment::Center),
        ];

        // create content
        let content = column![toolbar, scrollable_rows_for_thumbnails, progress_bar];

        content.into()
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }
}
