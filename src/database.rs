use rusqlite::{Connection, Result};

use serde::Serialize;

#[derive(Serialize)]
pub struct LeaderboardEntry {
    pub id: i32,
    pub username: String,
    pub highscore: i32,
    pub date_created: String,
}

pub fn create_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS leaderboard (
            id INTEGER PRIMARY KEY,
            username TEXT NOT NULL,
            highscore INTEGER NOT NULL,
            date_created TEXT NOT NULL
        )",
        [],
    )?;
    Ok(())
}

pub fn get_all_leaderboard_data(conn: &Connection) -> Result<Vec<LeaderboardEntry>> {
    let mut stmt = conn.prepare("SELECT * FROM leaderboard ORDER BY highscore DESC")?;
    let rows = stmt.query_map([], |row| {
        Ok(LeaderboardEntry {
            id: row.get(0)?,
            username: row.get(1)?,
            highscore: row.get(2)?,
            date_created: row.get(3)?,
        })
    })?;
    let leaderboard_data: Result<Vec<_>> = rows.collect();
    leaderboard_data
}

pub fn update_leaderboard(conn: &Connection, id: i32, new_score: i32) -> Result<()> {
    conn.execute(
        "UPDATE leaderboard SET highscore = ?1 WHERE id = ?2",
        &[&new_score, &id],
    )?;
    Ok(())
}
