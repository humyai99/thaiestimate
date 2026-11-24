// ThaiEstimate — โปรแกรมถอดแบบ & ประมาณราคาก่อสร้าง (Rust)
// Phase 1: โครงสร้างโปรเจกต์ + ฐานข้อมูล SQLite
mod db;

fn main() -> anyhow::Result<()> {
    let conn = db::init()?;
    println!("ThaiEstimate v0.1.0 — database initialized OK");
    // TODO: เปิดหน้าต่าง UI (egui)
    Ok(())
}
