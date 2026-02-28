use crate::gui::{NewFile, OpenFile, Quit};
use crate::modal::app_state::AppState;
use gpui::{App, Menu, MenuItem, SystemMenuType};
use rfd::{AsyncFileDialog, MessageDialog};
use crate::modal::project::LedMakerProject;

pub fn set_app_menus(cx: &mut App) {
    cx.set_menus(vec![
        Menu {
            name: "set_menus".into(),
            items: vec![
                MenuItem::os_submenu("Services", SystemMenuType::Services),
                MenuItem::separator(),
                MenuItem::action("Quit", Quit),
            ],
        },
        Menu {
            name: "Project".into(),
            items: vec![
                MenuItem::action("New File", NewFile),
                MenuItem::action("Open File", OpenFile),
            ],
        },
    ]);
}

pub fn quit(_: &Quit, cx: &mut App) {
    println!("Quitting...");
    cx.quit();
}

pub fn new_file(_: &NewFile, cx: &mut App) {
    println!("Creating new file...");
    let app_state = cx.global_mut::<AppState>();
    app_state.file_path = None;
    app_state.current_project = LedMakerProject::default();
}

pub fn open_file(_: &OpenFile, cx: &mut App) {
    cx.spawn(async move |cx| {
        let file = AsyncFileDialog::new()
            .add_filter("project file", &["ledm", "toml"])
            .add_filter("all files", &["*"])
            .set_directory(".")
            .pick_file()
            .await;
        let Some(file) = file else { return; };
        let path = file.path().to_path_buf();

        match LedMakerProject::load(&path) {
            Ok(proj) => {
                let _ = cx.update(|cx| {
                    let app_state = cx.global_mut::<AppState>();
                    app_state.file_path = Some(path);
                    app_state.current_project = proj;
                });
            }
            Err(err) => {
                println!("Error loading project: {}", err);
                MessageDialog::new()
                    .set_title("Error")
                    .set_description(format!("Failed to load project:\n{}", err))
                    .set_buttons(rfd::MessageButtons::Ok)
                    .set_level(rfd::MessageLevel::Error)
                    .show();
            }
        }

    })
    .detach();
}
