use error_stack::{IntoReport, Result, ResultExt};

use crate::models::{DBState, Epic, Status, Story};

#[derive(Debug)]
pub enum JiraDatabaseError {
    Read,
    Write,
    NoEpicWithID,
    NoStoryWithID,
}

impl std::fmt::Display for JiraDatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JiraDatabaseError::Read => {
                write!(f, "Failed to read Jira database.")
            }
            JiraDatabaseError::Write => {
                write!(f, "Failed to write Jira database.")
            }
            JiraDatabaseError::NoEpicWithID => {
                write!(f, "No Epic with ID found.")
            }
            JiraDatabaseError::NoStoryWithID => {
                write!(f, "No Story with ID found.")
            }
        }
    }
}

impl std::error::Error for JiraDatabaseError {}

pub struct JiraDatabase {
    pub database: Box<dyn Database>,
}

impl JiraDatabase {
    pub fn new(file_path: String) -> Self {
        Self {
            database: Box::new(JSONFileDatabase { file_path }),
        }
    }

    pub fn read_db(&self) -> Result<DBState, JiraDatabaseError> {
        self.database
            .read_db()
            .change_context(JiraDatabaseError::Read)
    }

    pub fn create_epic(&self, epic: Epic) -> Result<u32, JiraDatabaseError> {
        let mut db_state = self
            .database
            .read_db()
            .change_context(JiraDatabaseError::Read)?;

        let id = db_state.last_item_id + 1;
        db_state.epics.insert(id, epic);
        db_state.last_item_id = id;

        self.database
            .write_db(&db_state)
            .change_context(JiraDatabaseError::Write)?;

        Ok(id)
    }

    pub fn create_story(&self, story: Story, epic_id: u32) -> Result<u32, JiraDatabaseError> {
        let mut db_state = self
            .database
            .read_db()
            .change_context(JiraDatabaseError::Read)?;

        let epic = db_state
            .epics
            .get_mut(&epic_id)
            .ok_or(JiraDatabaseError::NoEpicWithID)?;

        let id = db_state.last_item_id + 1;
        db_state.stories.insert(id, story);
        epic.stories.push(id);
        db_state.last_item_id = id;

        self.database
            .write_db(&db_state)
            .change_context(JiraDatabaseError::Write)?;

        Ok(id)
    }

    pub fn delete_epic(&self, epic_id: u32) -> Result<(), JiraDatabaseError> {
        let mut db_state = self
            .database
            .read_db()
            .change_context(JiraDatabaseError::Read)?;

        let epic = db_state
            .epics
            .get(&epic_id)
            .ok_or(JiraDatabaseError::NoEpicWithID)?;

        for story in &epic.stories {
            db_state.stories.remove(story);
        }

        db_state
            .epics
            .remove(&epic_id)
            .ok_or(JiraDatabaseError::NoEpicWithID)?;

        self.database
            .write_db(&db_state)
            .change_context(JiraDatabaseError::Write)?;

        Ok(())
    }

    pub fn delete_story(&self, epic_id: u32, story_id: u32) -> Result<(), JiraDatabaseError> {
        let mut db_state = self
            .database
            .read_db()
            .change_context(JiraDatabaseError::Read)?;

        let epic = db_state
            .epics
            .get_mut(&epic_id)
            .ok_or(JiraDatabaseError::NoEpicWithID)?;
        if epic.stories.contains(&story_id) {
            db_state
                .stories
                .remove(&story_id)
                .ok_or(JiraDatabaseError::NoStoryWithID)?;
            epic.stories.remove(
                epic.stories
                    .binary_search(&story_id)
                    .expect("Story ID not in epic."),
            );

            self.database
                .write_db(&db_state)
                .change_context(JiraDatabaseError::Write)?;
        } else {
            return Err(JiraDatabaseError::NoStoryWithID).into_report();
        }

        Ok(())
    }

    pub fn update_epic_status(
        &self,
        epic_id: u32,
        status: Status,
    ) -> Result<(), JiraDatabaseError> {
        let mut db_state = self
            .database
            .read_db()
            .change_context(JiraDatabaseError::Read)?;

        let epic = db_state
            .epics
            .get_mut(&epic_id)
            .ok_or(JiraDatabaseError::NoEpicWithID)?;

        epic.status = status;

        self.database
            .write_db(&db_state)
            .change_context(JiraDatabaseError::Write)?;

        Ok(())
    }

    pub fn update_story_status(
        &self,
        story_id: u32,
        status: Status,
    ) -> Result<(), JiraDatabaseError> {
        let mut db_state = self
            .database
            .read_db()
            .change_context(JiraDatabaseError::Read)?;

        let story = db_state
            .stories
            .get_mut(&story_id)
            .ok_or(JiraDatabaseError::NoEpicWithID)?;

        story.status = status;

        self.database
            .write_db(&db_state)
            .change_context(JiraDatabaseError::Write)?;

        Ok(())
    }
}

#[derive(Debug)]
pub enum DatabaseError {
    ReadError,
    WriteError,
}

impl std::fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DatabaseError::ReadError => {
                write!(f, "Failed to read database.")
            }
            DatabaseError::WriteError => {
                write!(f, "Failed to write database.")
            }
        }
    }
}

impl std::error::Error for DatabaseError {}

pub trait Database {
    fn read_db(&self) -> Result<DBState, DatabaseError>;
    fn write_db(&self, db_state: &DBState) -> Result<(), DatabaseError>;
}

struct JSONFileDatabase {
    pub file_path: String,
}

impl Database for JSONFileDatabase {
    fn read_db(&self) -> Result<DBState, DatabaseError> {
        let raw_content = std::fs::read_to_string(&self.file_path)
            .into_report()
            .change_context(DatabaseError::ReadError)?;

        serde_json::from_str::<DBState>(&raw_content)
            .into_report()
            .change_context(DatabaseError::ReadError)
    }

    fn write_db(&self, db_state: &DBState) -> Result<(), DatabaseError> {
        let file = std::fs::File::create(&self.file_path)
            .into_report()
            .change_context(DatabaseError::WriteError)?;

        serde_json::to_writer(file, db_state)
            .into_report()
            .change_context(DatabaseError::WriteError)
    }
}

#[cfg(test)]
pub mod test_utils {
    use std::{cell::RefCell, collections::HashMap};

    use super::*;

    pub struct MockDB {
        last_written_state: RefCell<DBState>,
    }

    impl MockDB {
        pub fn new() -> Self {
            Self {
                last_written_state: RefCell::new(DBState {
                    last_item_id: 0,
                    epics: HashMap::new(),
                    stories: HashMap::new(),
                }),
            }
        }
    }

    impl Database for MockDB {
        fn read_db(&self) -> Result<DBState, DatabaseError> {
            // TODO: fix this error by deriving the appropriate traits for Story
            let state = self.last_written_state.borrow().clone();
            Ok(state)
        }

        fn write_db(&self, db_state: &DBState) -> Result<(), DatabaseError> {
            let latest_state = &self.last_written_state;
            // TODO: fix this error by deriving the appropriate traits for DBState
            *latest_state.borrow_mut() = db_state.clone();
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::test_utils::MockDB;
    use super::*;

    #[test]
    fn create_epic_should_work() {
        let db = JiraDatabase {
            database: Box::new(MockDB::new()),
        };
        let epic = Epic::new("".to_owned(), "".to_owned());

        // TODO: fix this error by deriving the appropriate traits for Epic
        let result = db.create_epic(epic.clone());

        assert_eq!(result.is_ok(), true);

        let id = result.unwrap();
        let db_state = db.read_db().unwrap();

        let expected_id = 1;

        assert_eq!(id, expected_id);
        assert_eq!(db_state.last_item_id, expected_id);
        assert_eq!(db_state.epics.get(&id), Some(&epic));
    }

    #[test]
    fn create_story_should_error_if_invalid_epic_id() {
        let db = JiraDatabase {
            database: Box::new(MockDB::new()),
        };
        let story = Story::new("".to_owned(), "".to_owned());

        let non_existent_epic_id = 999;

        let result = db.create_story(story, non_existent_epic_id);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn create_story_should_work() {
        let db = JiraDatabase {
            database: Box::new(MockDB::new()),
        };
        let epic = Epic::new("".to_owned(), "".to_owned());
        let story = Story::new("".to_owned(), "".to_owned());

        let result = db.create_epic(epic);
        assert_eq!(result.is_ok(), true);

        let epic_id = result.unwrap();

        // TODO: fix this error by deriving the appropriate traits for Story
        let result = db.create_story(story.clone(), epic_id);
        assert_eq!(result.is_ok(), true);

        let id = result.unwrap();
        let db_state = db.read_db().unwrap();

        let expected_id = 2;

        assert_eq!(id, expected_id);
        assert_eq!(db_state.last_item_id, expected_id);
        assert_eq!(
            db_state.epics.get(&epic_id).unwrap().stories.contains(&id),
            true
        );
        assert_eq!(db_state.stories.get(&id), Some(&story));
    }

    #[test]
    fn delete_epic_should_error_if_invalid_epic_id() {
        let db = JiraDatabase {
            database: Box::new(MockDB::new()),
        };

        let non_existent_epic_id = 999;

        let result = db.delete_epic(non_existent_epic_id);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn delete_epic_should_work() {
        let db = JiraDatabase {
            database: Box::new(MockDB::new()),
        };
        let epic = Epic::new("".to_owned(), "".to_owned());
        let story = Story::new("".to_owned(), "".to_owned());

        let result = db.create_epic(epic);
        assert_eq!(result.is_ok(), true);

        let epic_id = result.unwrap();

        let result = db.create_story(story, epic_id);
        assert_eq!(result.is_ok(), true);

        let story_id = result.unwrap();

        let result = db.delete_epic(epic_id);
        assert_eq!(result.is_ok(), true);

        let db_state = db.read_db().unwrap();

        let expected_last_id = 2;

        assert_eq!(db_state.last_item_id, expected_last_id);
        assert_eq!(db_state.epics.get(&epic_id), None);
        assert_eq!(db_state.stories.get(&story_id), None);
    }

    #[test]
    fn delete_story_should_error_if_invalid_epic_id() {
        let db = JiraDatabase {
            database: Box::new(MockDB::new()),
        };
        let epic = Epic::new("".to_owned(), "".to_owned());
        let story = Story::new("".to_owned(), "".to_owned());

        let result = db.create_epic(epic);
        assert_eq!(result.is_ok(), true);

        let epic_id = result.unwrap();

        let result = db.create_story(story, epic_id);
        assert_eq!(result.is_ok(), true);

        let story_id = result.unwrap();

        let non_existent_epic_id = 999;

        let result = db.delete_story(non_existent_epic_id, story_id);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn delete_story_should_error_if_story_not_found_in_epic() {
        let db = JiraDatabase {
            database: Box::new(MockDB::new()),
        };
        let epic = Epic::new("".to_owned(), "".to_owned());
        let story = Story::new("".to_owned(), "".to_owned());

        let result = db.create_epic(epic);
        assert_eq!(result.is_ok(), true);

        let epic_id = result.unwrap();

        let result = db.create_story(story, epic_id);
        assert_eq!(result.is_ok(), true);

        let non_existent_story_id = 999;

        let result = db.delete_story(epic_id, non_existent_story_id);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn delete_story_should_work() {
        let db = JiraDatabase {
            database: Box::new(MockDB::new()),
        };
        let epic = Epic::new("".to_owned(), "".to_owned());
        let story = Story::new("".to_owned(), "".to_owned());

        let result = db.create_epic(epic);
        assert_eq!(result.is_ok(), true);

        let epic_id = result.unwrap();

        let result = db.create_story(story, epic_id);
        assert_eq!(result.is_ok(), true);

        let story_id = result.unwrap();

        let result = db.delete_story(epic_id, story_id);
        assert_eq!(result.is_ok(), true);

        let db_state = db.read_db().unwrap();

        let expected_last_id = 2;

        assert_eq!(db_state.last_item_id, expected_last_id);
        assert_eq!(
            db_state
                .epics
                .get(&epic_id)
                .unwrap()
                .stories
                .contains(&story_id),
            false
        );
        assert_eq!(db_state.stories.get(&story_id), None);
    }

    #[test]
    fn update_epic_status_should_error_if_invalid_epic_id() {
        let db = JiraDatabase {
            database: Box::new(MockDB::new()),
        };

        let non_existent_epic_id = 999;

        let result = db.update_epic_status(non_existent_epic_id, Status::Closed);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn update_epic_status_should_work() {
        let db = JiraDatabase {
            database: Box::new(MockDB::new()),
        };
        let epic = Epic::new("".to_owned(), "".to_owned());

        let result = db.create_epic(epic);

        assert_eq!(result.is_ok(), true);

        let epic_id = result.unwrap();

        let result = db.update_epic_status(epic_id, Status::Closed);

        assert_eq!(result.is_ok(), true);

        let db_state = db.read_db().unwrap();

        assert_eq!(db_state.epics.get(&epic_id).unwrap().status, Status::Closed);
    }

    #[test]
    fn update_story_status_should_error_if_invalid_story_id() {
        let db = JiraDatabase {
            database: Box::new(MockDB::new()),
        };

        let non_existent_story_id = 999;

        let result = db.update_story_status(non_existent_story_id, Status::Closed);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn update_story_status_should_work() {
        let db = JiraDatabase {
            database: Box::new(MockDB::new()),
        };
        let epic = Epic::new("".to_owned(), "".to_owned());
        let story = Story::new("".to_owned(), "".to_owned());

        let result = db.create_epic(epic);

        let epic_id = result.unwrap();

        let result = db.create_story(story, epic_id);

        let story_id = result.unwrap();

        let result = db.update_story_status(story_id, Status::Closed);

        assert_eq!(result.is_ok(), true);

        let db_state = db.read_db().unwrap();

        assert_eq!(
            db_state.stories.get(&story_id).unwrap().status,
            Status::Closed
        );
    }

    mod database {
        use std::collections::HashMap;
        use std::fs::remove_file;
        use std::io::Write;

        use super::*;

        #[test]
        fn read_db_should_fail_with_invalid_path() {
            let db = JSONFileDatabase {
                file_path: "INVALID_PATH".to_owned(),
            };
            assert_eq!(db.read_db().is_err(), true);
        }

        #[test]
        fn read_db_should_fail_with_invalid_json() {
            let mut tmpfile = tempfile::NamedTempFile::new().unwrap();

            let file_contents = r#"{ "last_item_id": 0 epics: {} stories {} }"#;
            write!(tmpfile, "{}", file_contents).unwrap();

            let file_path = "./data/read_db_should_fail_with_invalid_json.json".to_owned();

            let path = tmpfile.into_temp_path();
            path.persist(&file_path).unwrap();

            let db = JSONFileDatabase {
                file_path: file_path.clone(),
            };

            let result = db.read_db();

            remove_file(file_path).unwrap();

            assert_eq!(result.is_err(), true);
        }

        #[test]
        fn read_db_should_parse_json_file() {
            let mut tmpfile = tempfile::NamedTempFile::new().unwrap();

            let file_contents = r#"{ "last_item_id": 0, "epics": {}, "stories": {} }"#;
            write!(tmpfile, "{}", file_contents).unwrap();

            let file_path = "./data/read_db_should_parse_json_file.json".to_owned();

            let path = tmpfile.into_temp_path();
            path.persist(&file_path).unwrap();

            let db = JSONFileDatabase {
                file_path: file_path.clone(),
            };

            let result = db.read_db();

            remove_file(file_path).unwrap();

            assert_eq!(result.is_ok(), true);
        }

        #[test]
        fn write_db_should_work() {
            let mut tmpfile = tempfile::NamedTempFile::new().unwrap();

            let file_contents = r#"{ "last_item_id": 0, "epics": {}, "stories": {} }"#;
            write!(tmpfile, "{}", file_contents).unwrap();

            let file_path = "./data/write_db_should_work.json".to_owned();

            let path = tmpfile.into_temp_path();
            path.persist(&file_path).unwrap();

            let db = JSONFileDatabase {
                file_path: file_path.clone(),
            };

            let story = Story {
                name: "epic 1".to_owned(),
                description: "epic 1".to_owned(),
                status: Status::Open,
            };
            let epic = Epic {
                name: "epic 1".to_owned(),
                description: "epic 1".to_owned(),
                status: Status::Open,
                stories: vec![2],
            };

            let mut stories = HashMap::new();
            stories.insert(2, story);

            let mut epics = HashMap::new();
            epics.insert(1, epic);

            let state = DBState {
                last_item_id: 2,
                epics,
                stories,
            };

            let write_result = db.write_db(&state);
            let read_result = db.read_db().unwrap();

            remove_file(file_path).unwrap();

            assert_eq!(write_result.is_ok(), true);
            assert_eq!(read_result, state);
        }
    }
}
