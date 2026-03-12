use crate::error::Result;
use crate::models::{Category, CategoryId, EntryId, Exercise, ExerciseId, SessionId, SetId};
use rusqlite::Connection;

pub fn list_categories(conn: &Connection) -> Result<Vec<Category>> {
    let mut stmt = conn.prepare("SELECT id, name FROM categories ORDER BY name")?;
    let categories = stmt
        .query_map([], |row| {
            let id_str: String = row.get(0)?;
            let name: String = row.get(1)?;
            Ok((id_str, name))
        })?
        .filter_map(std::result::Result::ok)
        .filter_map(|(id_str, name)| {
            CategoryId::try_from(id_str).ok().map(|id| Category { id, name })
        })
        .collect();
    Ok(categories)
}

pub fn list_exercises(conn: &Connection) -> Result<Vec<Exercise>> {
    let mut stmt = conn.prepare(
        "SELECT e.id, e.name, e.category_id, c.name
         FROM exercises e
         JOIN categories c ON e.category_id = c.id
         ORDER BY c.name, e.name",
    )?;
    let exercises = stmt
        .query_map([], |row| {
            let id_str: String = row.get(0)?;
            let name: String = row.get(1)?;
            let cat_id_str: String = row.get(2)?;
            let cat_name: String = row.get(3)?;
            Ok((id_str, name, cat_id_str, cat_name))
        })?
        .filter_map(std::result::Result::ok)
        .filter_map(|(id_str, name, cat_id_str, cat_name)| {
            let id = ExerciseId::try_from(id_str).ok()?;
            let category_id = CategoryId::try_from(cat_id_str).ok()?;
            Some(Exercise {
                id,
                name,
                category_id,
                category_name: cat_name,
            })
        })
        .collect();
    Ok(exercises)
}

pub fn insert_exercise(
    conn: &Connection,
    name: &str,
    category_id: &CategoryId,
) -> Result<ExerciseId> {
    let id = ExerciseId::new();
    conn.execute(
        "INSERT INTO exercises (id, name, category_id) VALUES (?1, ?2, ?3)",
        rusqlite::params![id.to_db_string(), name, category_id.to_db_string()],
    )?;
    Ok(id)
}

pub fn delete_exercise(conn: &Connection, id: &ExerciseId) -> Result<()> {
    conn.execute(
        "DELETE FROM exercises WHERE id = ?1",
        rusqlite::params![id.to_db_string()],
    )?;
    Ok(())
}

pub fn insert_session(conn: &Connection, date: &str) -> Result<SessionId> {
    let id = SessionId::new();
    conn.execute(
        "INSERT INTO workout_sessions (id, date) VALUES (?1, ?2)",
        rusqlite::params![id.to_db_string(), date],
    )?;
    Ok(id)
}

pub fn insert_entry(
    conn: &Connection,
    session_id: &SessionId,
    exercise_id: &ExerciseId,
    order: usize,
) -> Result<EntryId> {
    let id = EntryId::new();
    #[allow(clippy::cast_possible_wrap)] // sort_order is bounded (session won't have 2^63 exercises)
    let sort_order = order as i64;
    conn.execute(
        "INSERT INTO workout_entries (id, session_id, exercise_id, sort_order)
         VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![
            id.to_db_string(),
            session_id.to_db_string(),
            exercise_id.to_db_string(),
            sort_order
        ],
    )?;
    Ok(id)
}

pub fn insert_set(
    conn: &Connection,
    entry_id: &EntryId,
    set_number: usize,
    reps: u32,
    weight: f64,
) -> Result<SetId> {
    let id = SetId::new();
    #[allow(clippy::cast_possible_wrap)] // set_number is bounded (no workout has 2^63 sets)
    let set_num = set_number as i64;
    conn.execute(
        "INSERT INTO sets (id, entry_id, set_number, reps, weight)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![
            id.to_db_string(),
            entry_id.to_db_string(),
            set_num,
            i64::from(reps),
            weight
        ],
    )?;
    Ok(id)
}
