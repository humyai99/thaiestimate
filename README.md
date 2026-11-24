# 🏗️ ThaiEstimate

โปรแกรมถอดแบบและประมาณราคาก่อสร้าง — คล้าย mTools Next + Estimate Plus

## ✨ ฟีเจอร์

- 📋 จัดการโครงการ (เพิ่ม/ลบ/Export/Import JSON)
- 🧱 คลังราคาวัสดุ (ฐานข้อมูลวัสดุ + ราคา)
- 📊 BOQ — ใบประมาณราคา (คำนวณอัตโนมัติ + ส่งออก Excel)
- 📐 ถอดแบบ PDF — วัดระยะ/พื้นที่/Smart Area/Smart Count/มาร์กอัป
- 🔩 แผนตัดเหล็ก — คำนวณน้ำหนักอัตโนมัติ
- 📤 ส่งออก BOQ เป็น Excel (rust_xlsxwriter)
- 📥 Import/Export โครงการเป็น JSON

## 🛠️ เทคโนโลยี

- Rust + egui (eframe) — UI
- rusqlite (bundled) — SQLite
- pdfium-render — PDF rendering
- rust_xlsxwriter — Excel export
- serde / serde_json — JSON

## 🚀 การติดตั้ง

### Linux / macOS

```bash
cargo build --release
cp /path/to/libpdfium.so ./libpdfium.so
./target/release/thaiestimate
```

### Windows

```bash
cargo build --release
copy libpdfium.dll .\thaiestimate.exe
.\thaiestimate.exe
```

## 📦 Releases

ดาวน์โหลดไฟล์ `.exe` / `.msi` จาก [Releases](https://github.com/humyai99/thaiestimate/releases)

## 📝 การใช้งาน

1. สร้างโครงการในแท็บ 📋 โครงการ
2. เพิ่มวัสดุในแท็บ 🧱 คลังราคา
3. วัดแบบ PDF ในแท็บ 📐 ถอดแบบ (ระยะ/พื้นที่/Smart Area/Smart Count/มาร์กอัป)
4. ส่งเข้า BOQ ในแท็บ 📊 BOQ
5. ส่งออก Excel หรือดูแผนตัดเหล็กในแท็บ 🔩 เหล็ก

## 📄 License

MIT
