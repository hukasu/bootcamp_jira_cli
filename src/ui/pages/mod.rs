use std::rc::Rc;

use itertools::Itertools;
use error_stack::{Context, IntoReport, Report, Result, ResultExt};

use crate::db::JiraDatabase;
use crate::models::Action;

mod page_helpers;
use page_helpers::*;

#[derive(Debug)]
pub enum PageError {
    DrawError
}

impl std::fmt::Display for PageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PageError::DrawError => {
                write!(f, "Failed to draw page.")
            },
        }
    }
}

impl std::error::Error for PageError {}

pub trait Page {
    fn draw_page(&self) -> Result<(), PageError>;
    fn handle_input(&self, input: &str) -> Result<Option<Action>, PageError>;
}

pub struct HomePage {
    pub db: Rc<JiraDatabase>
}
impl Page for HomePage {
    fn draw_page(&self) -> Result<(), PageError> {
        println!("----------------------------- EPICS -----------------------------");
        println!("     id     |               name               |      status      ");

        let db_state = self.db.read_db(
        ).change_context(PageError::DrawError)?;
        let keys = itertools::sorted(db_state.epics.keys());
        for id in keys {
            if let Some(epic) = db_state.epics.get(&id) {
                println!("{}|{}|{}", id, epic.name, epic.status);
            }
        }

        println!();
        println!();

        println!("[q] quit | [c] create epic | [:id:] navigate to epic");

        Ok(())
    }

    fn handle_input(&self, input: &str) -> Result<Option<Action>, PageError> {
        if input.is_empty() {
            Ok(None)
        } else if let Some(id) = input.parse::<u32>().ok() {
            let db_state = self.db.read_db().change_context(PageError::DrawError)?;
            if db_state.epics.contains_key(&id) {
                Ok(Some(Action::NavigateToEpicDetail { epic_id: id }))
            } else {
                Ok(None)
            }
        } else {
            match input {
                "q" => Ok(Some(Action::Exit)),
                "c" => Ok(Some(Action::CreateEpic)),
                _ => Ok(None)
            }
        }
    }
}

pub struct EpicDetail {
    pub epic_id: u32,
    pub db: Rc<JiraDatabase>
}

impl Page for EpicDetail {
    fn draw_page(&self) -> Result<(), PageError> {
        let db_state = self.db.read_db().change_context(PageError::DrawError)?;
        let epic = db_state.epics.get(&self.epic_id).ok_or(PageError::DrawError)?;

        println!("------------------------------ EPIC ------------------------------");
        println!("  id  |     name     |         description         |    status    ");

        println!("{}|{}|{}|{}", self.epic_id, epic.name, epic.description, epic.status);
  
        println!();

        println!("---------------------------- STORIES ----------------------------");
        println!("     id     |               name               |      status      ");

        let stories = &db_state.stories;
        let keys = itertools::sorted(stories.keys());
        for id in keys {
            if let Some(story) = db_state.stories.get(&id) {
                println!("{}|{}|{}", id, story.name, story.status);
            }
        }
        
        println!();
        println!();

        println!("[p] previous | [u] update epic | [d] delete epic | [c] create story | [:id:] navigate to story");

        Ok(())
    }

    fn handle_input(&self, input: &str) -> Result<Option<Action>, PageError> {
        if input.is_empty() {
            Ok(None)
        } else if let Some(id) = input.parse::<u32>().ok() {
            let db_state = self.db.read_db().change_context(PageError::DrawError)?;
            if db_state.stories.contains_key(&id) {
                Ok(Some(Action::NavigateToStoryDetail { epic_id: self.epic_id, story_id: id }))
            } else {
                Ok(None)
            }
        } else {
            match input {
                "p" => Ok(Some(Action::NavigateToPreviousPage)),
                "u" => Ok(Some(Action::UpdateEpicStatus { epic_id: self.epic_id })),
                "d" => Ok(Some(Action::DeleteEpic { epic_id: self.epic_id })),
                "c" => Ok(Some(Action::CreateStory { epic_id: self.epic_id })),
                _ => Ok(None)
            }
        }
    }
}

pub struct StoryDetail {
    pub epic_id: u32,
    pub story_id: u32,
    pub db: Rc<JiraDatabase>
}

impl Page for StoryDetail {
    fn draw_page(&self) -> Result<(), PageError> {
        let db_state = self.db.read_db().change_context(PageError::DrawError)?;
        let story = db_state.stories.get(
            &self.story_id
        ).ok_or(PageError::DrawError)?;

        println!("------------------------------ STORY ------------------------------");
        println!("  id  |     name     |         description         |    status    ");
        
        println!("{}|{}|{}|{}", self.epic_id, story.name, story.description, story.status);
        
        println!();
        println!();

        println!("[p] previous | [u] update story | [d] delete story");

        Ok(())
    }

    fn handle_input(&self, input: &str) -> Result<Option<Action>, PageError> {
        if input.is_empty() {
            Ok(None)
        } else if let Some(_id) = input.parse::<u32>().ok() {
            Ok(None)
        } else {
            match input {
                "p" => Ok(Some(Action::NavigateToPreviousPage)),
                "q" => Ok(Some(Action::Exit)),
                "u" => Ok(Some(Action::UpdateStoryStatus { story_id: self.story_id })),
                "d" => Ok(Some(Action::DeleteStory { epic_id: self.epic_id, story_id: self.story_id })),
                _ => Ok(None)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{db::test_utils::MockDB};
    use crate::models::{Epic, Story};

    mod home_page {
        use super::*;

        #[test]
        fn draw_page_should_not_throw_error() {
            let db = Rc::new(JiraDatabase { database: Box::new(MockDB::new()) });

            let page = HomePage { db };
            assert_eq!(page.draw_page().is_ok(), true);
        }
        
        #[test]
        fn handle_input_should_not_throw_error() {
            let db = Rc::new(JiraDatabase { database: Box::new(MockDB::new()) });

            let page = HomePage { db };
            assert_eq!(page.handle_input("").is_ok(), true);
        } 

        #[test]
        fn handle_input_should_return_the_correct_actions() {
            let db = Rc::new(JiraDatabase { database: Box::new(MockDB::new()) });

            let epic = Epic::new("".to_owned(), "".to_owned());

            let epic_id = db.create_epic(epic).unwrap();

            let page = HomePage { db };

            let q = "q";
            let c = "c";
            let valid_epic_id = epic_id.to_string();
            let invalid_epic_id = "999";
            let junk_input = "j983f2j";

            assert_eq!(page.handle_input(q).unwrap(), Some(Action::Exit));
            assert_eq!(page.handle_input(c).unwrap(), Some(Action::CreateEpic));
            assert_eq!(page.handle_input(&valid_epic_id).unwrap(), Some(Action::NavigateToEpicDetail { epic_id: 1 }));
            assert_eq!(page.handle_input(invalid_epic_id).unwrap(), None);
            assert_eq!(page.handle_input(junk_input).unwrap(), None);
        } 
    }

    mod epic_detail_page {
        use super::*;

        #[test]
        fn draw_page_should_not_throw_error() {
            let db = Rc::new(JiraDatabase { database: Box::new(MockDB::new()) });
            let epic_id = db.create_epic(Epic::new("".to_owned(), "".to_owned())).unwrap();

            let page = EpicDetail { epic_id, db };
            assert_eq!(page.draw_page().is_ok(), true);
        }

        #[test]
        fn handle_input_should_not_throw_error() {
            let db = Rc::new(JiraDatabase { database: Box::new(MockDB::new()) });
            let epic_id = db.create_epic(Epic::new("".to_owned(), "".to_owned())).unwrap();

            let page = EpicDetail { epic_id, db };
            assert_eq!(page.handle_input("").is_ok(), true);
        }

        #[test]
        fn draw_page_should_throw_error_for_invalid_epic_id() {
            let db = Rc::new(JiraDatabase { database: Box::new(MockDB::new()) });

            let page = EpicDetail { epic_id: 999, db };
            assert_eq!(page.draw_page().is_err(), true);
        }

        #[test]
        fn handle_input_should_return_the_correct_actions() {
            let db = Rc::new(JiraDatabase { database: Box::new(MockDB::new()) });

            let epic_id = db.create_epic(Epic::new("".to_owned(), "".to_owned())).unwrap();
            let story_id = db.create_story(Story::new("".to_owned(), "".to_owned()), epic_id).unwrap();

            let page = EpicDetail { epic_id, db };

            let p = "p";
            let u = "u";
            let d = "d";
            let c = "c";
            let invalid_story_id = "999";
            let junk_input = "j983f2j";

            assert_eq!(page.handle_input(p).unwrap(), Some(Action::NavigateToPreviousPage));
            assert_eq!(page.handle_input(u).unwrap(), Some(Action::UpdateEpicStatus { epic_id: 1 }));
            assert_eq!(page.handle_input(d).unwrap(), Some(Action::DeleteEpic { epic_id: 1 }));
            assert_eq!(page.handle_input(c).unwrap(), Some(Action::CreateStory { epic_id: 1 }));
            assert_eq!(page.handle_input(&story_id.to_string()).unwrap(), Some(Action::NavigateToStoryDetail { epic_id: 1, story_id: 2 }));
            assert_eq!(page.handle_input(invalid_story_id).unwrap(), None);
            assert_eq!(page.handle_input(junk_input).unwrap(), None);
        } 
    }

    mod story_detail_page {
        use super::*;

        #[test]
        fn draw_page_should_not_throw_error() {
            let db = Rc::new(JiraDatabase { database: Box::new(MockDB::new()) });

            let epic_id = db.create_epic(Epic::new("".to_owned(), "".to_owned())).unwrap();
            let story_id = db.create_story(Story::new("".to_owned(), "".to_owned()), epic_id).unwrap();

            let page = StoryDetail { epic_id, story_id, db };
            assert_eq!(page.draw_page().is_ok(), true);
        }

        #[test]
        fn handle_input_should_not_throw_error() {
            let db = Rc::new(JiraDatabase { database: Box::new(MockDB::new()) });

            let epic_id = db.create_epic(Epic::new("".to_owned(), "".to_owned())).unwrap();
            let story_id = db.create_story(Story::new("".to_owned(), "".to_owned()), epic_id).unwrap();

            let page = StoryDetail { epic_id, story_id, db };
            assert_eq!(page.handle_input("").is_ok(), true);
        }

        #[test]
        fn draw_page_should_throw_error_for_invalid_story_id() {
            let db = Rc::new(JiraDatabase { database: Box::new(MockDB::new()) });

            let epic_id = db.create_epic(Epic::new("".to_owned(), "".to_owned())).unwrap();
            let _ = db.create_story(Story::new("".to_owned(), "".to_owned()), epic_id).unwrap();

            let page = StoryDetail { epic_id, story_id: 999, db };
            assert_eq!(page.draw_page().is_err(), true);
        }

        #[test]
        fn handle_input_should_return_the_correct_actions() {
            let db = Rc::new(JiraDatabase { database: Box::new(MockDB::new()) });

            let epic_id = db.create_epic(Epic::new("".to_owned(), "".to_owned())).unwrap();
            let story_id = db.create_story(Story::new("".to_owned(), "".to_owned()), epic_id).unwrap();

            let page = StoryDetail { epic_id, story_id, db };

            let p = "p";
            let u = "u";
            let d = "d";
            let some_number = "1";
            let junk_input = "j983f2j";

            assert_eq!(page.handle_input(p).unwrap(), Some(Action::NavigateToPreviousPage));
            assert_eq!(page.handle_input(u).unwrap(), Some(Action::UpdateStoryStatus { story_id }));
            assert_eq!(page.handle_input(d).unwrap(), Some(Action::DeleteStory { epic_id, story_id }));
            assert_eq!(page.handle_input(some_number).unwrap(), None);
            assert_eq!(page.handle_input(junk_input).unwrap(), None);
        } 
    }
}