pub mod queries;

use crate::error::{AppError, Result};
use rusqlite::Connection;
use std::path::PathBuf;
use uuid::Uuid;

const SEED_CATEGORIES: &[&str] = &[
    "Back",
    "Biceps",
    "Calves",
    "Cardio",
    "Chest",
    "Core / Abs",
    "Forearms",
    "Full Body",
    "Glutes",
    "Legs (Hamstrings)",
    "Legs (Quads)",
    "Shoulders",
    "Triceps",
];

pub fn db_path() -> Result<PathBuf> {
    let data_dir = dirs::data_dir().ok_or(AppError::NoDataDir)?;
    let app_dir = data_dir.join("fitness-tracker");
    std::fs::create_dir_all(&app_dir)?;
    Ok(app_dir.join("data.db"))
}

pub fn open_db() -> Result<Connection> {
    let path = db_path()?;
    let conn = Connection::open(path)?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    init_schema(&conn)?;
    seed_categories(&conn)?;
    Ok(conn)
}

fn init_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS categories (
            id   TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE
        );

        CREATE TABLE IF NOT EXISTS exercises (
            id          TEXT PRIMARY KEY,
            name        TEXT NOT NULL UNIQUE,
            category_id TEXT NOT NULL REFERENCES categories(id)
        );

        CREATE TABLE IF NOT EXISTS workout_sessions (
            id   TEXT PRIMARY KEY,
            date TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS workout_entries (
            id          TEXT PRIMARY KEY,
            session_id  TEXT NOT NULL REFERENCES workout_sessions(id),
            exercise_id TEXT NOT NULL REFERENCES exercises(id),
            sort_order  INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS sets (
            id         TEXT PRIMARY KEY,
            entry_id   TEXT NOT NULL REFERENCES workout_entries(id),
            set_number INTEGER NOT NULL,
            reps       INTEGER NOT NULL,
            weight     REAL    NOT NULL
        );
        ",
    )?;
    Ok(())
}

fn seed_categories(conn: &Connection) -> Result<()> {
    let count: i64 =
        conn.query_row("SELECT COUNT(*) FROM categories", [], |row| row.get(0))?;
    if count == 0 {
        for name in SEED_CATEGORIES {
            let id = Uuid::new_v4().hyphenated().to_string();
            conn.execute(
                "INSERT INTO categories (id, name) VALUES (?1, ?2)",
                rusqlite::params![id, name],
            )?;
        }
    }
    Ok(())
}
