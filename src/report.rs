// รายงาน BOQ — ส่งออก Excel (Phase 6)
use anyhow::Result;
use rust_xlsxwriter::{Workbook, Format, Color, FormatAlign, FormatBorder};
use rusqlite::Connection;
use crate::db::{self};

pub fn export_boq_excel(conn: &Connection, project_id: i64, path: &str) -> Result<()> {
    let projects = db::list_projects(conn)?;
    let materials = db::list_materials(conn)?;
    let items = db::list_boq(conn, project_id)?;
    let project = projects.iter().find(|p| p.id == project_id);

    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    worksheet.set_name("BOQ")?;

    let title_fmt = Format::new().set_bold().set_font_size(18).set_font_color(Color::from(0x1E5A96));
    let header_fmt = Format::new().set_bold().set_font_color(Color::White).set_background_color(Color::from(0x1E5A96)).set_align(FormatAlign::Center).set_border(FormatBorder::Thin);
    let cell_fmt = Format::new().set_border(FormatBorder::Thin).set_align(FormatAlign::Center);
    let num_fmt = Format::new().set_border(FormatBorder::Thin).set_num_format("#,##0.00").set_align(FormatAlign::Right);
    let total_fmt = Format::new().set_bold().set_font_size(14).set_font_color(Color::from(0xB8860B)).set_border(FormatBorder::Medium);

    worksheet.set_column_width(0, 40)?;
    worksheet.set_column_width(1, 12)?;
    worksheet.set_column_width(2, 16)?;
    worksheet.set_column_width(3, 20)?;
    worksheet.set_column_width(4, 12)?;

    worksheet.merge_range(0, 0, 0, 4, "ThaiEstimate — ใบประมาณราคา (BOQ)", &title_fmt)?;
    worksheet.set_row_height(0, 28)?;

    if let Some(p) = project {
        worksheet.write_string(1, 0, format!("โครงการ: {}", p.name))?;
        worksheet.write_string(2, 0, format!("เลขที่: {} | เจ้าของ: {} | ที่อยู่: {}", p.code, p.owner, p.location))?;
        worksheet.write_string(3, 0, format!("พื้นที่: {:.2} ตร.ม.", p.area))?;
    }

    let header_row = 5;
    worksheet.write_string_with_format(header_row, 0, "รายการ", &header_fmt)?;
    worksheet.write_string_with_format(header_row, 1, "ปริมาณ", &header_fmt)?;
    worksheet.write_string_with_format(header_row, 2, "ราคา/หน่วย", &header_fmt)?;
    worksheet.write_string_with_format(header_row, 3, "รวม", &header_fmt)?;
    worksheet.write_string_with_format(header_row, 4, "หน่วย", &header_fmt)?;

    let mut grand_total = 0.0;
    let mut row = header_row + 1;
    for it in &items {
        let mat = materials.iter().find(|m| m.id == it.material_id);
        let name = mat.map(|m| m.name_th.clone()).unwrap_or_else(|| "?".into());
        let unit = mat.map(|m| m.unit.clone()).unwrap_or_else(|| "หน่วย".into());
        worksheet.write_string_with_format(row, 0, &name, &cell_fmt)?;
        worksheet.write_number_with_format(row, 1, it.quantity, &num_fmt)?;
        worksheet.write_number_with_format(row, 2, it.unit_price, &num_fmt)?;
        worksheet.write_number_with_format(row, 3, it.total, &num_fmt)?;
        worksheet.write_string_with_format(row, 4, &unit, &cell_fmt)?;
        grand_total += it.total;
        row += 1;
    }

    worksheet.merge_range(row, 0, row, 2, "รวมทั้งสิ้น", &total_fmt)?;
    worksheet.write_number_with_format(row, 3, grand_total, &total_fmt)?;

    workbook.save(path)?;
    Ok(())
}
