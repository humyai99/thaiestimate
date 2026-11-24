// ฐานข้อมูล SQLite — โครงสร้าง Phase 1
use anyhow::Result;
use rusqlite::Connection;

pub fn init() -> Result<Connection> {
    let conn = Connection::open("thaiestimate.db")?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS projects (
            id INTEGER PRIMARY KEY,
            code TEXT, name TEXT, owner TEXT, location TEXT,
            area REAL, status TEXT DEFAULT 'draft', created_at TEXT DEFAULT (datetime('now'))
        );
        CREATE TABLE IF NOT EXISTS material_groups (
            id INTEGER PRIMARY KEY,
            parent_id INTEGER, name TEXT, unit TEXT
        );
        CREATE TABLE IF NOT EXISTS materials (
            id INTEGER PRIMARY KEY,
            group_id INTEGER, name_th TEXT, name_en TEXT,
            unit TEXT, material_price REAL, labor_price REAL
        );
        CREATE TABLE IF NOT EXISTS boq_items (
            id INTEGER PRIMARY KEY,
            project_id INTEGER, material_id INTEGER,
            quantity REAL, unit_price REAL, total REAL
        );"
    )?;
    Ok(conn)
}
