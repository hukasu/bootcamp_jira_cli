use std::rc::Rc;

mod models;

mod db;
use db::*;

mod ui;

mod io_utils;
use io_utils::*;

mod navigator;
use navigator::*;

fn main() {
    let db = Rc::new(JiraDatabase::new("data/db.json".to_owned()));
    let mut navigator = Navigator::new(db);

    loop {
        clearscreen::clear().unwrap();
        let cur_page = match navigator.get_current_page() {
            Some(page) => page,
            None => break,
        };

        if let Err(e) = cur_page.draw_page() {
            println!("{:?}", e);
            let _ = get_user_input();
            continue;
        };

        let buf = get_user_input();
        let action = match cur_page.handle_input(&buf) {
            Ok(Some(a)) => a,
            Err(e) => {
                println!("{:?}", e);
                let _ = get_user_input();
                continue;
            }
            _ => continue,
        };
        if let Err(e) = navigator.handle_action(action) {
            println!("{:?}", e);
            let _ = get_user_input();
        }
    }
}
