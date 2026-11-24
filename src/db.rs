use anyhow::Result;
use rusqlite::{Connection, params};

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

pub struct Project { pub id: i64, pub code: String, pub name: String, pub owner: String, pub location: String, pub area: f64, pub status: String }
pub struct Material { pub id: i64, pub group_id: i64, pub name_th: String, pub unit: String, pub material_price: f64, pub labor_price: f64 }
pub struct BoqItem { pub id: i64, pub project_id: i64, pub material_id: i64, pub quantity: f64, pub unit_price: f64, pub total: f64 }

pub fn list_projects(conn: &Connection) -> Result<Vec<Project>> {
    let mut stmt = conn.prepare("SELECT id, code, name, owner, location, area, status FROM projects ORDER BY id DESC")?;
    let rows = stmt.query_map([], |r| Ok(Project { id: r.get(0)?, code: r.get(1)?, name: r.get(2)?, owner: r.get(3)?, location: r.get(4)?, area: r.get(5)?, status: r.get(6)? }))?;
    let mut out = vec![];
    for r in rows { out.push(r?); }
    Ok(out)
}

pub fn add_project(conn: &Connection, code: &str, name: &str, owner: &str, location: &str, area: f64) -> Result<i64> {
    conn.execute("INSERT INTO projects (code, name, owner, location, area) VALUES (?1,?2,?3,?4,?5)", params![code, name, owner, location, area])?;
    Ok(conn.last_insert_rowid())
}

pub fn delete_project(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM boq_items WHERE project_id=?1", params![id])?;
    conn.execute("DELETE FROM projects WHERE id=?1", params![id])?;
    Ok(())
}

pub fn list_materials(conn: &Connection) -> Result<Vec<Material>> {
    let mut stmt = conn.prepare("SELECT id, group_id, name_th, unit, material_price, labor_price FROM materials ORDER BY id DESC")?;
    let rows = stmt.query_map([], |r| Ok(Material { id: r.get(0)?, group_id: r.get(1)?, name_th: r.get(2)?, unit: r.get(3)?, material_price: r.get(4)?, labor_price: r.get(5)? }))?;
    let mut out = vec![];
    for r in rows { out.push(r?); }
    Ok(out)
}

pub fn add_material(conn: &Connection, group_id: i64, name_th: &str, unit: &str, mp: f64, lp: f64) -> Result<i64> {
    conn.execute("INSERT INTO materials (group_id, name_th, unit, material_price, labor_price) VALUES (?1,?2,?3,?4,?5)", params![group_id, name_th, unit, mp, lp])?;
    Ok(conn.last_insert_rowid())
}

pub fn list_boq(conn: &Connection, project_id: i64) -> Result<Vec<BoqItem>> {
    let mut stmt = conn.prepare("SELECT id, project_id, material_id, quantity, unit_price, total FROM boq_items WHERE project_id=?1 ORDER BY id")?;
    let rows = stmt.query_map(params![project_id], |r| Ok(BoqItem { id: r.get(0)?, project_id: r.get(1)?, material_id: r.get(2)?, quantity: r.get(3)?, unit_price: r.get(4)?, total: r.get(5)? }))?;
    let mut out = vec![];
    for r in rows { out.push(r?); }
    Ok(out)
}

pub fn add_boq_item(conn: &Connection, project_id: i64, material_id: i64, qty: f64, price: f64) -> Result<()> {
    let total = qty * price;
    conn.execute("INSERT INTO boq_items (project_id, material_id, quantity, unit_price, total) VALUES (?1,?2,?3,?4,?5)", params![project_id, material_id, qty, price, total])?;
    Ok(())
}

pub fn delete_boq_item(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM boq_items WHERE id=?1", params![id])?;
    Ok(())
}
