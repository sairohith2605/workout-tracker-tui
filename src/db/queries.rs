use crate::error::Result;
use crate::models::{
    Category, CategoryId, EntryId, Exercise, ExerciseId, HistoryEntry, HistorySet,
    HistorySummary, SessionId, SetId,
};
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

/// Returns one summary row per unique date in reverse-chronological order.
/// If the user logged multiple sessions on the same day they are merged here,
/// with `exercise_count` reflecting the total across all sessions that day.
pub fn list_sessions(conn: &Connection) -> Result<Vec<HistorySummary>> {
    let mut stmt = conn.prepare(
        "SELECT ws.date, COUNT(we.id) AS exercise_count
         FROM workout_sessions ws
         LEFT JOIN workout_entries we ON we.session_id = ws.id
         GROUP BY ws.date
         ORDER BY ws.date DESC",
    )?;
    let sessions = stmt
        .query_map([], |row| {
            let date: String = row.get(0)?;
            let count: i64 = row.get(1)?;
            Ok((date, count))
        })?
        .filter_map(std::result::Result::ok)
        .map(|(date, count)| HistorySummary {
            date,
            exercise_count: usize::try_from(count).unwrap_or(0),
        })
        .collect();
    Ok(sessions)
}

/// Returns every exercise + set logged on `date` (across all sessions that day),
/// grouped by exercise in recorded order.
pub fn get_day_detail(conn: &Connection, date: &str) -> Result<Vec<HistoryEntry>> {
    let mut stmt = conn.prepare(
        "SELECT e.name, c.name, s.set_number, s.reps, s.weight
         FROM workout_sessions ws
         JOIN workout_entries we ON we.session_id = ws.id
         JOIN exercises e        ON e.id  = we.exercise_id
         JOIN categories c       ON c.id  = e.category_id
         JOIN sets s             ON s.entry_id = we.id
         WHERE ws.date = ?1
         ORDER BY ws.rowid, we.sort_order, s.set_number",
    )?;
    let rows: Vec<(String, String, i64, i64, f64)> = stmt
        .query_map(rusqlite::params![date], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, f64>(4)?,
            ))
        })?
        .filter_map(std::result::Result::ok)
        .collect();

    let mut entries: Vec<HistoryEntry> = Vec::new();
    for (ex_name, cat_name, set_number, reps, weight) in rows {
        let set = HistorySet {
            set_number: usize::try_from(set_number).unwrap_or(0),
            reps: u32::try_from(reps).unwrap_or(0),
            weight,
        };
        match entries.last_mut() {
            Some(last) if last.exercise_name == ex_name => last.sets.push(set),
            _ => entries.push(HistoryEntry {
                exercise_name: ex_name,
                category_name: cat_name,
                sets: vec![set],
            }),
        }
    }
    Ok(entries)
}
