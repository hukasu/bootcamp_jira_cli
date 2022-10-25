use crate::{models::{Epic, Story, Status}, io_utils::get_user_input};

pub struct Prompts {
    pub create_epic: Box<dyn Fn() -> Epic>,
    pub create_story: Box<dyn Fn() -> Story>,
    pub delete_epic: Box<dyn Fn() -> bool>,
    pub delete_story: Box<dyn Fn() -> bool>,
    pub update_status: Box<dyn Fn() -> Option<Status>>
}

impl Prompts {
    pub fn new() -> Self {
        Self { 
            create_epic: Box::new(create_epic_prompt),
            create_story: Box::new(create_story_prompt),
            delete_epic: Box::new(delete_epic_prompt),
            delete_story: Box::new(delete_story_prompt),
            update_status: Box::new(update_status_prompt)
        }
    }
}

fn create_epic_prompt() -> Epic {
    println!("----------------------------");
    println!("Epic Name:");
    let mut name = String::with_capacity(64);
    let _r = std::io::stdin().read_line(&mut name);
    let name = name.trim();
    println!("Epic Description:");
    let mut description = String::with_capacity(1024);
    let _r = std::io::stdin().read_line(&mut description);
    let description = description.trim();
    Epic::new(name.to_owned(), description.to_owned())
}

fn create_story_prompt() -> Story {
    println!("----------------------------");
    println!("Story Name:");
    let mut name = String::with_capacity(64);
    let _r = std::io::stdin().read_line(&mut name);
    let name = name.trim();
    println!("Story Description:");
    let mut description = String::with_capacity(1024);
    let _r = std::io::stdin().read_line(&mut description);
    let description = description.trim();
    Story::new(name.to_owned(), description.to_owned())
}

fn delete_epic_prompt() -> bool {
    println!("----------------------------");
    println!("Are you sure you want to delete this epic? All stories in this epic will also be deleted [Y/n]:");
    let mut name = String::with_capacity(64);
    let _r = std::io::stdin().read_line(&mut name);
    let name = name.trim();
    if name.is_empty() {
        true
    } else {
        match name {
            "Y" => true,
            "y" => true,
            "N" => false,
            "n" => false,
            _ => false
        }
    }
}

fn delete_story_prompt() -> bool {
    println!("----------------------------");
    println!("Are you sure you want to delete this story? [Y/n]:");
    let mut prompt = String::with_capacity(64);
    let _r = std::io::stdin().read_line(&mut prompt);
    let prompt = prompt.trim();
    if prompt.is_empty() {
        true
    } else {
        match prompt {
            "Y" => true,
            "y" => true,
            "N" => false,
            "n" => false,
            _ => false
        }
    }
}

fn update_status_prompt() -> Option<Status> {
    println!("----------------------------");
    println!("New Status (1 - OPEN, 2 - IN-PROGRESS, 3 - RESOLVED, 4 - CLOSED):");
    let mut prompt = String::with_capacity(64);
    let _r = std::io::stdin().read_line(&mut prompt);
    let prompt = prompt.trim();
    let pars = prompt.parse::<u32>().ok();
    match pars {
        Some(1) => Some(Status::Open),
        Some(2) => Some(Status::InProgress),
        Some(3) => Some(Status::Resolved),
        Some(4) => Some(Status::Closed),
        _ => None
    }
}