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
        CREATE TABLE IF NOT EXISTS takeoff_measurements (
            id INTEGER PRIMARY KEY,
            project_id INTEGER,
            pdf_path TEXT,
            page_index INTEGER,
            measure_type TEXT,
            points_json TEXT,
            real_unit_per_px REAL,
            value REAL,
            label TEXT,
            created_at TEXT DEFAULT (datetime('now'))
        );
        CREATE TABLE IF NOT EXISTS steel_bars (
            id INTEGER PRIMARY KEY,
            project_id INTEGER, diameter REAL, length REAL, count INTEGER,
            weight_per_m REAL
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

pub struct SteelBar { pub id: i64, pub project_id: i64, pub diameter: f64, pub length: f64, pub count: i64, pub weight_per_m: f64 }

pub fn add_steel_bar(conn: &Connection, project_id: i64, diameter: f64, length: f64, count: i64) -> Result<i64> {
    // น้ำหนักเหล็ก (kg/m) = diameter^2 * 0.00617
    let weight_per_m = diameter * diameter * 0.00617;
    conn.execute("INSERT INTO steel_bars (project_id, diameter, length, count, weight_per_m) VALUES (?1,?2,?3,?4,?5)", params![project_id, diameter, length, count, weight_per_m])?;
    Ok(conn.last_insert_rowid())
}

pub fn list_steel_bars(conn: &Connection, project_id: i64) -> Result<Vec<SteelBar>> {
    let mut stmt = conn.prepare("SELECT id, project_id, diameter, length, count, weight_per_m FROM steel_bars WHERE project_id=?1 ORDER BY id")?;
    let rows = stmt.query_map(params![project_id], |r| Ok(SteelBar { id: r.get(0)?, project_id: r.get(1)?, diameter: r.get(2)?, length: r.get(3)?, count: r.get(4)?, weight_per_m: r.get(5)? }))?;
    let mut out = vec![];
    for r in rows { out.push(r?); }
    Ok(out)
}

pub fn delete_steel_bar(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM steel_bars WHERE id=?1", params![id])?;
    Ok(())
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ProjectExport {
    pub code: String, pub name: String, pub owner: String, pub location: String, pub area: f64,
    pub boq: Vec<BoqItemExport>,
    pub steel: Vec<SteelBarExport>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct BoqItemExport { pub material_id: i64, pub quantity: f64, pub unit_price: f64, pub total: f64 }

#[derive(serde::Serialize, serde::Deserialize)]
pub struct SteelBarExport { pub diameter: f64, pub length: f64, pub count: i64, pub weight_per_m: f64 }

pub fn export_project(conn: &Connection, project_id: i64) -> Result<ProjectExport> {
    let projects = list_projects(conn)?;
    let project = projects.iter().find(|p| p.id == project_id).ok_or_else(|| anyhow::anyhow!("ไม่พบโครงการ"))?;
    let items = list_boq(conn, project_id)?;
    let bars = list_steel_bars(conn, project_id)?;
    Ok(ProjectExport {
        code: project.code.clone(), name: project.name.clone(), owner: project.owner.clone(),
        location: project.location.clone(), area: project.area,
        boq: items.iter().map(|i| BoqItemExport { material_id: i.material_id, quantity: i.quantity, unit_price: i.unit_price, total: i.total }).collect(),
        steel: bars.iter().map(|b| SteelBarExport { diameter: b.diameter, length: b.length, count: b.count, weight_per_m: b.weight_per_m }).collect(),
    })
}

pub fn import_project(conn: &Connection, data: &ProjectExport) -> Result<i64> {
    let new_id = add_project(conn, &data.code, &data.name, &data.owner, &data.location, data.area)?;
    for item in &data.boq {
        add_boq_item(conn, new_id, item.material_id, item.quantity, item.unit_price)?;
    }
    for bar in &data.steel {
        add_steel_bar(conn, new_id, bar.diameter, bar.length, bar.count)?;
    }
    Ok(new_id)
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct TakeoffMeasurement {
    pub id: i64,
    pub project_id: i64,
    pub pdf_path: String,
    pub page_index: i32,
    pub measure_type: String,
    pub points_json: String,
    pub real_unit_per_px: f64,
    pub value: f64,
    pub label: String,
}

pub fn add_takeoff_measurement(conn: &Connection, project_id: i64, pdf_path: &str, page_index: i32, measure_type: &str, points_json: &str, real_unit_per_px: f64, value: f64, label: &str) -> Result<i64> {
    conn.execute("INSERT INTO takeoff_measurements (project_id, pdf_path, page_index, measure_type, points_json, real_unit_per_px, value, label) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
        params![project_id, pdf_path, page_index, measure_type, points_json, real_unit_per_px, value, label])?;
    Ok(conn.last_insert_rowid())
}

pub fn list_takeoff_measurements(conn: &Connection, project_id: i64) -> Result<Vec<TakeoffMeasurement>> {
    let mut stmt = conn.prepare("SELECT id, project_id, pdf_path, page_index, measure_type, points_json, real_unit_per_px, value, label FROM takeoff_measurements WHERE project_id=?1 ORDER BY id DESC")?;
    let rows = stmt.query_map(params![project_id], |r| Ok(TakeoffMeasurement {
        id: r.get(0)?, project_id: r.get(1)?, pdf_path: r.get(2)?, page_index: r.get(3)?,
        measure_type: r.get(4)?, points_json: r.get(5)?, real_unit_per_px: r.get(6)?,
        value: r.get(7)?, label: r.get(8)?,
    }))?;
    let mut out = vec![];
    for r in rows { out.push(r?); }
    Ok(out)
}

pub fn delete_takeoff_measurement(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM takeoff_measurements WHERE id=?1", params![id])?;
    Ok(())
}
