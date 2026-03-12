use crate::app::{AddField, App, Screen, SetField};
use crate::db::queries;
use crate::error::Result;
use crate::models::SetInput;
use chrono::Local;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

pub fn handle_key(app: &mut App, key: KeyEvent) -> Result<()> {
    // Ignore key-release and key-repeat events (important on Windows).
    if key.kind != KeyEventKind::Press {
        return Ok(());
    }
    // Always clear the previous status; each handler re-sets it as needed.
    app.status_msg = None;

    match app.screen {
        Screen::CatalogList => handle_catalog_list(app, key)?,
        Screen::AddExercise => handle_add_exercise(app, key)?,
        Screen::MainMenu => handle_main_menu(app, key),
        Screen::WorkoutDate => handle_workout_date(app, key),
        Screen::ExercisePicker => handle_exercise_picker(app, key),
        Screen::SetLogger => handle_set_logger(app, key),
        Screen::WorkoutSummary => handle_workout_summary(app, key)?,
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// CatalogList
// ---------------------------------------------------------------------------

fn handle_catalog_list(app: &mut App, key: KeyEvent) -> Result<()> {
    let filtered = app.filtered_catalog();

    match key.code {
        KeyCode::Char('q') => {
            app.delete_confirm = false;
            app.should_quit = true;
        }
        KeyCode::Char('a') => {
            app.delete_confirm = false;
            app.add_name.clear();
            app.add_category_idx = 0;
            app.add_focus = AddField::Name;
            app.screen = Screen::AddExercise;
        }
        KeyCode::Char('f') => {
            app.delete_confirm = false;
            app.catalog_filter_idx = cycle_filter(app.catalog_filter_idx, app.categories.len());
            let new_len = app.filtered_catalog().len();
            app.catalog_selected = clamp_cursor(app.catalog_selected, new_len);
        }
        KeyCode::Up => {
            app.delete_confirm = false;
            if app.catalog_selected > 0 {
                app.catalog_selected -= 1;
            }
        }
        KeyCode::Down => {
            app.delete_confirm = false;
            if !filtered.is_empty() && app.catalog_selected + 1 < filtered.len() {
                app.catalog_selected += 1;
            }
        }
        KeyCode::Char('d') => {
            if filtered.is_empty() {
                app.delete_confirm = false;
                return Ok(());
            }
            if app.delete_confirm {
                if let Some(ex) = filtered.get(app.catalog_selected) {
                    let id = ex.id.clone();
                    let name = ex.name.clone();
                    queries::delete_exercise(&app.db, &id)?;
                    app.reload_catalog()?;
                    app.status_msg = Some(format!("Deleted '{name}'"));
                }
                app.delete_confirm = false;
            } else if let Some(ex) = filtered.get(app.catalog_selected) {
                app.status_msg = Some(format!(
                    "Press [d] again to confirm deleting '{}'",
                    ex.name
                ));
                app.delete_confirm = true;
            }
        }
        KeyCode::Enter => {
            app.delete_confirm = false;
            if !app.catalog.is_empty() {
                app.screen = Screen::MainMenu;
            }
        }
        _ => {
            app.delete_confirm = false;
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// AddExercise
// ---------------------------------------------------------------------------

fn handle_add_exercise(app: &mut App, key: KeyEvent) -> Result<()> {
    match app.add_focus {
        AddField::Name => match key.code {
            KeyCode::Esc => app.screen = Screen::CatalogList,
            KeyCode::Tab => app.add_focus = AddField::Category,
            KeyCode::Enter => try_insert_exercise(app)?,
            KeyCode::Backspace => {
                app.add_name.pop();
            }
            KeyCode::Char(c) => app.add_name.push(c),
            _ => {}
        },
        AddField::Category => match key.code {
            KeyCode::Esc => app.screen = Screen::CatalogList,
            KeyCode::Tab | KeyCode::BackTab => app.add_focus = AddField::Name,
            KeyCode::Enter => try_insert_exercise(app)?,
            KeyCode::Up => {
                if app.add_category_idx > 0 {
                    app.add_category_idx -= 1;
                }
            }
            KeyCode::Down => {
                if !app.categories.is_empty()
                    && app.add_category_idx + 1 < app.categories.len()
                {
                    app.add_category_idx += 1;
                }
            }
            _ => {}
        },
    }
    Ok(())
}

fn try_insert_exercise(app: &mut App) -> Result<()> {
    let name = app.add_name.trim().to_string();
    if name.is_empty() {
        app.status_msg = Some("Exercise name cannot be empty".to_string());
        return Ok(());
    }
    if let Some(cat) = app.categories.get(app.add_category_idx) {
        let category_id = cat.id.clone();
        match queries::insert_exercise(&app.db, &name, &category_id) {
            Ok(_) => {
                app.reload_catalog()?;
                app.status_msg = Some(format!("Added '{name}'"));
                app.screen = Screen::CatalogList;
            }
            Err(crate::error::AppError::Db(ref e))
                if e.to_string().contains("UNIQUE constraint failed") =>
            {
                app.status_msg = Some(format!("'{name}' already exists in the catalog"));
            }
            Err(e) => return Err(e),
        }
    } else {
        app.status_msg = Some("Please select a category".to_string());
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// MainMenu
// ---------------------------------------------------------------------------

fn handle_main_menu(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('l') => {
            app.session_date = Local::now().format("%Y-%m-%d").to_string();
            app.session_exercises.clear();
            app.picker_selected = 0;
            app.picker_filter_idx = None;
            app.screen = Screen::WorkoutDate;
        }
        KeyCode::Char('c') => {
            app.catalog_selected = 0;
            app.catalog_filter_idx = None;
            app.delete_confirm = false;
            app.screen = Screen::CatalogList;
        }
        KeyCode::Char('q') => app.should_quit = true,
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// WorkoutDate
// ---------------------------------------------------------------------------

fn handle_workout_date(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => app.screen = Screen::MainMenu,
        KeyCode::Enter => {
            if chrono::NaiveDate::parse_from_str(&app.session_date, "%Y-%m-%d").is_err() {
                app.status_msg = Some("Invalid date — use YYYY-MM-DD format".to_string());
            } else {
                app.screen = Screen::ExercisePicker;
            }
        }
        KeyCode::Backspace => {
            app.session_date.pop();
        }
        KeyCode::Char(c) => {
            if app.session_date.len() < 10 {
                app.session_date.push(c);
            }
        }
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// ExercisePicker
// ---------------------------------------------------------------------------

fn handle_exercise_picker(app: &mut App, key: KeyEvent) {
    let filtered = app.filtered_picker_catalog();

    match key.code {
        KeyCode::Esc => app.screen = Screen::WorkoutDate,
        KeyCode::Char('f') => {
            app.picker_filter_idx = cycle_filter(app.picker_filter_idx, app.categories.len());
            let new_len = app.filtered_picker_catalog().len();
            app.picker_selected = clamp_cursor(app.picker_selected, new_len);
        }
        KeyCode::Up => {
            if app.picker_selected > 0 {
                app.picker_selected -= 1;
            }
        }
        KeyCode::Down => {
            if !filtered.is_empty() && app.picker_selected + 1 < filtered.len() {
                app.picker_selected += 1;
            }
        }
        KeyCode::Enter => {
            if let Some(exercise) = filtered.get(app.picker_selected).cloned() {
                if app.is_exercise_in_session(&exercise) {
                    app.status_msg =
                        Some("Already added — pick a different exercise".to_string());
                } else {
                    app.current_exercise = Some(exercise);
                    app.current_sets.clear();
                    app.set_reps_input.clear();
                    app.set_weight_input.clear();
                    app.set_focus = SetField::Reps;
                    app.screen = Screen::SetLogger;
                }
            }
        }
        KeyCode::Char('d') => {
            if app.session_exercises.is_empty() {
                app.status_msg = Some("Add at least one exercise first".to_string());
            } else {
                app.screen = Screen::WorkoutSummary;
            }
        }
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// SetLogger
// ---------------------------------------------------------------------------

fn handle_set_logger(app: &mut App, key: KeyEvent) {
    match app.set_focus {
        SetField::Reps => match key.code {
            KeyCode::Esc => discard_current_exercise(app),
            KeyCode::Tab => app.set_focus = SetField::Weight,
            KeyCode::Char(c) if c.is_ascii_digit() => app.set_reps_input.push(c),
            KeyCode::Backspace => {
                app.set_reps_input.pop();
            }
            KeyCode::Enter => try_add_set(app),
            KeyCode::Char('d') => complete_set_logger(app),
            _ => {}
        },
        SetField::Weight => match key.code {
            KeyCode::Esc => discard_current_exercise(app),
            KeyCode::Tab => app.set_focus = SetField::Reps,
            KeyCode::Char(c) if c.is_ascii_digit() || c == '.' => app.set_weight_input.push(c),
            KeyCode::Backspace => {
                app.set_weight_input.pop();
            }
            KeyCode::Enter => try_add_set(app),
            KeyCode::Char('d') => complete_set_logger(app),
            _ => {}
        },
    }
}

fn try_add_set(app: &mut App) {
    let reps: u32 = match app.set_reps_input.parse::<u32>() {
        Ok(v) if v > 0 => v,
        _ => {
            app.status_msg = Some("Reps must be a positive whole number".to_string());
            return;
        }
    };
    let weight: f64 = match app.set_weight_input.parse::<f64>() {
        Ok(v) if v >= 0.0 => v,
        _ => {
            app.status_msg =
                Some("Weight must be a non-negative number (e.g. 60.5)".to_string());
            return;
        }
    };
    app.current_sets.push(SetInput { reps, weight });
    app.set_reps_input.clear();
    app.set_weight_input.clear();
    app.set_focus = SetField::Reps;
    let count = app.current_sets.len();
    app.status_msg = Some(format!(
        "Set {count} recorded — press [d] when done or add more"
    ));
}

fn complete_set_logger(app: &mut App) {
    if app.current_sets.is_empty() {
        app.status_msg = Some("Record at least one set first".to_string());
        return;
    }
    if let Some(exercise) = app.current_exercise.take() {
        app.session_exercises.push(crate::models::SessionExercise {
            exercise,
            sets: app.current_sets.drain(..).collect(),
        });
    }
    app.screen = Screen::ExercisePicker;
}

fn discard_current_exercise(app: &mut App) {
    app.current_exercise = None;
    app.current_sets.clear();
    app.set_reps_input.clear();
    app.set_weight_input.clear();
    app.screen = Screen::ExercisePicker;
}

// ---------------------------------------------------------------------------
// WorkoutSummary
// ---------------------------------------------------------------------------

fn handle_workout_summary(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Esc => app.screen = Screen::ExercisePicker,
        KeyCode::Char('s') => save_workout(app)?,
        _ => {}
    }
    Ok(())
}

fn save_workout(app: &mut App) -> Result<()> {
    let session_id = queries::insert_session(&app.db, &app.session_date)?;

    for (order, se) in app.session_exercises.iter().enumerate() {
        let entry_id = queries::insert_entry(&app.db, &session_id, &se.exercise.id, order)?;
        for (set_idx, set) in se.sets.iter().enumerate() {
            queries::insert_set(&app.db, &entry_id, set_idx + 1, set.reps, set.weight)?;
        }
    }

    let exercise_count = app.session_exercises.len();
    let set_count: usize = app.session_exercises.iter().map(|se| se.sets.len()).sum();
    let date = app.session_date.clone();
    app.session_exercises.clear();

    app.status_msg = Some(format!(
        "Saved! {exercise_count} exercise(s), {set_count} set(s) on {date}"
    ));
    app.screen = Screen::MainMenu;
    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn cycle_filter(current: Option<usize>, count: usize) -> Option<usize> {
    if count == 0 {
        return None;
    }
    match current {
        None => Some(0),
        Some(idx) => {
            if idx + 1 < count {
                Some(idx + 1)
            } else {
                None
            }
        }
    }
}

fn clamp_cursor(cursor: usize, len: usize) -> usize {
    if len == 0 {
        0
    } else {
        cursor.min(len - 1)
    }
}
