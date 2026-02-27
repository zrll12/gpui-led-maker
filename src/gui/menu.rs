use crate::gui::{NewFile, OpenFile, Quit};
use crate::modal::app_state::AppState;
use gpui::{App, Menu, MenuItem, SystemMenuType};

pub fn set_app_menus(cx: &mut App) {
    let app_state = cx.global::<AppState>();
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
