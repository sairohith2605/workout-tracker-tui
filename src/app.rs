use crate::db::queries;
use crate::error::Result;
use crate::models::{Category, Exercise, HistoryEntry, HistorySummary, SessionExercise, SetInput};
use chrono::Local;
use rusqlite::Connection;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    CatalogList,
    AddExercise,
    MainMenu,
    WorkoutDate,
    ExercisePicker,
    SetLogger,
    WorkoutSummary,
    HistoryList,
    HistoryDetail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddField {
    Name,
    Category,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetField {
    Reps,
    Weight,
}

pub struct App {
    pub screen: Screen,
    pub db: Connection,
    pub should_quit: bool,

    // --- Catalog ---
    pub catalog: Vec<Exercise>,
    pub categories: Vec<Category>,
    pub catalog_filter_idx: Option<usize>,
    pub catalog_selected: usize,
    pub delete_confirm: bool,

    // --- AddExercise form ---
    pub add_name: String,
    pub add_category_idx: usize,
    pub add_focus: AddField,

    // --- Workout session ---
    pub session_date: String,
    pub session_exercises: Vec<SessionExercise>,

    // --- ExercisePicker ---
    pub picker_selected: usize,
    pub picker_filter_idx: Option<usize>,

    // --- SetLogger ---
    pub current_exercise: Option<Exercise>,
    pub current_sets: Vec<SetInput>,
    pub set_reps_input: String,
    pub set_weight_input: String,
    pub set_focus: SetField,

    // --- History ---
    pub history_sessions: Vec<HistorySummary>,
    pub history_selected: usize,
    pub history_detail: Vec<HistoryEntry>,
    pub history_detail_scroll: usize,
    pub history_filter_input: String,
    pub history_filtering: bool,

    // --- Status bar ---
    pub status_msg: Option<String>,
}

impl App {
    pub fn new(db: Connection) -> Result<Self> {
        let catalog = queries::list_exercises(&db)?;
        let categories = queries::list_categories(&db)?;

        let initial_screen = if catalog.is_empty() {
            Screen::CatalogList
        } else {
            Screen::MainMenu
        };

        Ok(Self {
            screen: initial_screen,
            db,
            should_quit: false,
            catalog,
            categories,
            catalog_filter_idx: None,
            catalog_selected: 0,
            delete_confirm: false,
            add_name: String::new(),
            add_category_idx: 0,
            add_focus: AddField::Name,
            session_date: Local::now().format("%Y-%m-%d").to_string(),
            session_exercises: Vec::new(),
            picker_selected: 0,
            picker_filter_idx: None,
            current_exercise: None,
            current_sets: Vec::new(),
            set_reps_input: String::new(),
            set_weight_input: String::new(),
            set_focus: SetField::Reps,
            history_sessions: Vec::new(),
            history_selected: 0,
            history_detail: Vec::new(),
            history_detail_scroll: 0,
            history_filter_input: String::new(),
            history_filtering: false,
            status_msg: None,
        })
    }

    pub fn reload_catalog(&mut self) -> Result<()> {
        self.catalog = queries::list_exercises(&self.db)?;
        self.categories = queries::list_categories(&self.db)?;
        // Clamp cursor after deletion / reload
        let filtered_len = self.filtered_catalog().len();
        if filtered_len == 0 {
            self.catalog_selected = 0;
        } else if self.catalog_selected >= filtered_len {
            self.catalog_selected = filtered_len - 1;
        }
        Ok(())
    }

    /// Returns exercises matching the active catalog filter (owned clones).
    pub fn filtered_catalog(&self) -> Vec<Exercise> {
        match self.catalog_filter_idx {
            None => self.catalog.clone(),
            Some(idx) => match self.categories.get(idx) {
                Some(cat) => self
                    .catalog
                    .iter()
                    .filter(|e| e.category_id == cat.id)
                    .cloned()
                    .collect(),
                None => self.catalog.clone(),
            },
        }
    }

    /// Returns exercises matching the active picker filter (owned clones).
    pub fn filtered_picker_catalog(&self) -> Vec<Exercise> {
        match self.picker_filter_idx {
            None => self.catalog.clone(),
            Some(idx) => match self.categories.get(idx) {
                Some(cat) => self
                    .catalog
                    .iter()
                    .filter(|e| e.category_id == cat.id)
                    .cloned()
                    .collect(),
                None => self.catalog.clone(),
            },
        }
    }

    pub fn is_exercise_in_session(&self, exercise: &Exercise) -> bool {
        self.session_exercises
            .iter()
            .any(|se| se.exercise.id == exercise.id)
    }
}
