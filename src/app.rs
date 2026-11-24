use crate::db::{self};
use rusqlite::Connection;
use eframe::egui;

pub struct ThaiEstimateApp {
    conn: Connection,
    tab: usize,
    // project form
    p_code: String, p_name: String, p_owner: String, p_loc: String, p_area: String,
    // material form
    m_name: String, m_unit: String, m_mp: String, m_lp: String,
    // boq form
    b_project: i64, b_material: i64, b_qty: String, b_price: String,
    status_msg: String,
    pdf_path: String,
    viewer: Option<crate::pdf::PdfViewer>,
    measure_points: Vec<(f32, f32)>,
}

impl ThaiEstimateApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let conn = db::init().expect("init db failed");
        Self {
            conn, tab: 0,
            p_code: String::new(), p_name: String::new(), p_owner: String::new(), p_loc: String::new(), p_area: String::new(),
            m_name: String::new(), m_unit: "หน่วย".into(), m_mp: String::new(), m_lp: String::new(),
            b_project: 0, b_material: 0, b_qty: String::new(), b_price: String::new(),
            status_msg: String::new(),
            pdf_path: String::new(),
            viewer: None,
            measure_points: vec![],
        }
    }
}

impl eframe::App for ThaiEstimateApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // โทนสีน้ำเงินเข้ม — มืออาชีพ
        let mut style = (*ctx.style()).clone();
        style.visuals.dark_mode = true;
        style.visuals.panel_fill = egui::Color32::from_rgb(18, 24, 38);
        style.visuals.window_fill = egui::Color32::from_rgb(22, 30, 48);
        style.visuals.extreme_bg_color = egui::Color32::from_rgb(12, 17, 28);
        style.visuals.faint_bg_color = egui::Color32::from_rgb(28, 36, 56);
        style.visuals.selection.bg_fill = egui::Color32::from_rgb(30, 90, 160);
        style.visuals.hyperlink_color = egui::Color32::from_rgb(90, 170, 255);
        style.spacing.item_spacing = egui::vec2(8.0, 8.0);
        ctx.set_style(style);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("🏗️ ThaiEstimate").size(28.0).strong().color(egui::Color32::from_rgb(120, 190, 255)));
                ui.label(egui::RichText::new("ถอดแบบ & ประมาณราคาก่อสร้าง").size(13.0).color(egui::Color32::from_rgb(160, 170, 190)));
            });
            ui.separator();

            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.tab, 0, "📋 โครงการ");
                ui.selectable_value(&mut self.tab, 1, "🧱 คลังราคา");
                ui.selectable_value(&mut self.tab, 2, "📊 BOQ");
                ui.selectable_value(&mut self.tab, 3, "📐 ถอดแบบ");
            });
            ui.separator();

            match self.tab {
                0 => self.projects_tab(ui),
                1 => self.materials_tab(ui),
                2 => self.boq_tab(ui),
                3 => self.pdf_tab(ui),
                _ => {}
            }

            if !self.status_msg.is_empty() {
                ui.separator();
                ui.label(egui::RichText::new(&self.status_msg).color(egui::Color32::from_rgb(120, 220, 140)));
            }
        });
    }
}

impl ThaiEstimateApp {
    fn projects_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("จัดการโครงการ");
        ui.horizontal(|ui| {
            ui.label("เลขที่:"); ui.text_edit_singleline(&mut self.p_code);
            ui.label("ชื่อโครงการ:"); ui.text_edit_singleline(&mut self.p_name);
        });
        ui.horizontal(|ui| {
            ui.label("เจ้าของ:"); ui.text_edit_singleline(&mut self.p_owner);
            ui.label("ที่อยู่:"); ui.text_edit_singleline(&mut self.p_loc);
            ui.label("พื้นที่:"); ui.text_edit_singleline(&mut self.p_area);
        });
        if ui.button("➕ เพิ่มโครงการ").clicked() {
            let area: f64 = self.p_area.parse().unwrap_or(0.0);
            match db::add_project(&self.conn, &self.p_code, &self.p_name, &self.p_owner, &self.p_loc, area) {
                Ok(_) => { self.status_msg = "เพิ่มโครงการสำเร็จ".into(); self.p_code.clear(); self.p_name.clear(); self.p_owner.clear(); self.p_loc.clear(); self.p_area.clear(); }
                Err(e) => self.status_msg = format!("ผิดพลาด: {e}"),
            }
        }
        ui.separator();
        if let Ok(projects) = db::list_projects(&self.conn) {
            egui::ScrollArea::vertical().show(ui, |ui| {
                for p in &projects {
                    ui.horizontal(|ui| {
                        ui.label(format!("{} | {} | {} | {} ตร.ม. | {}", p.code, p.name, p.owner, p.area, p.status));
                        if ui.button("🗑").clicked() {
                            let _ = db::delete_project(&self.conn, p.id);
                            self.status_msg = "ลบโครงการแล้ว".into();
                        }
                    });
                }
                if projects.is_empty() { ui.label("ยังไม่มีโครงการ"); }
            });
        }
    }

    fn materials_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("คลังราคาวัสดุ");
        ui.horizontal(|ui| {
            ui.label("ชื่อวัสดุ:"); ui.text_edit_singleline(&mut self.m_name);
            ui.label("หน่วย:"); ui.text_edit_singleline(&mut self.m_unit);
        });
        ui.horizontal(|ui| {
            ui.label("ราคาวัสดุ:"); ui.text_edit_singleline(&mut self.m_mp);
            ui.label("ค่าแรง:"); ui.text_edit_singleline(&mut self.m_lp);
        });
        if ui.button("➕ เพิ่มวัสดุ").clicked() {
            let mp: f64 = self.m_mp.parse().unwrap_or(0.0);
            let lp: f64 = self.m_lp.parse().unwrap_or(0.0);
            match db::add_material(&self.conn, 1, &self.m_name, &self.m_unit, mp, lp) {
                Ok(_) => { self.status_msg = "เพิ่มวัสดุสำเร็จ".into(); self.m_name.clear(); self.m_mp.clear(); self.m_lp.clear(); }
                Err(e) => self.status_msg = format!("ผิดพลาด: {e}"),
            }
        }
        ui.separator();
        if let Ok(mats) = db::list_materials(&self.conn) {
            egui::ScrollArea::vertical().show(ui, |ui| {
                for m in &mats {
                    ui.label(format!("{} | {} | วัสดุ {:.2} | แรง {:.2}", m.name_th, m.unit, m.material_price, m.labor_price));
                }
                if mats.is_empty() { ui.label("ยังไม่มีวัสดุ"); }
            });
        }
    }

    fn boq_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("BOQ — ใบประมาณราคา");
        let projects = db::list_projects(&self.conn).unwrap_or_default();
        let materials = db::list_materials(&self.conn).unwrap_or_default();
        ui.horizontal(|ui| {
            ui.label("โครงการ:");
            egui::ComboBox::from_id_salt("proj").selected_text(if self.b_project == 0 { "เลือก".into() } else { projects.iter().find(|p| p.id == self.b_project).map(|p| p.name.clone()).unwrap_or_default() }).show_ui(ui, |ui| {
                for p in &projects { ui.selectable_value(&mut self.b_project, p.id, &p.name); }
            });
        });
        ui.horizontal(|ui| {
            ui.label("วัสดุ:");
            egui::ComboBox::from_id_salt("mat").selected_text(if self.b_material == 0 { "เลือก".into() } else { materials.iter().find(|m| m.id == self.b_material).map(|m| m.name_th.clone()).unwrap_or_default() }).show_ui(ui, |ui| {
                for m in &materials { ui.selectable_value(&mut self.b_material, m.id, &m.name_th); }
            });
        });
        ui.horizontal(|ui| {
            ui.label("ปริมาณ:"); ui.text_edit_singleline(&mut self.b_qty);
            ui.label("ราคา/หน่วย:"); ui.text_edit_singleline(&mut self.b_price);
        });
        if ui.button("➕ เพิ่มรายการ BOQ").clicked() {
            let qty: f64 = self.b_qty.parse().unwrap_or(0.0);
            let price: f64 = self.b_price.parse().unwrap_or(0.0);
            if self.b_project == 0 || self.b_material == 0 { self.status_msg = "เลือกโครงการและวัสดุก่อน".into(); }
            else {
                match db::add_boq_item(&self.conn, self.b_project, self.b_material, qty, price) {
                    Ok(_) => { self.status_msg = "เพิ่มรายการ BOQ สำเร็จ".into(); self.b_qty.clear(); self.b_price.clear(); }
                    Err(e) => self.status_msg = format!("ผิดพลาด: {e}"),
                }
            }
        }
        ui.separator();
        if self.b_project != 0 {
            if let Ok(items) = db::list_boq(&self.conn, self.b_project) {
                let mut grand = 0.0;
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for it in &items {
                        let mat = materials.iter().find(|m| m.id == it.material_id);
                        let name = mat.map(|m| m.name_th.clone()).unwrap_or_else(|| "?".into());
                        ui.horizontal(|ui| {
                            ui.label(format!("{} | qty {:.2} | {:.2}/หน่วย | {:.2}", name, it.quantity, it.unit_price, it.total));
                            if ui.button("🗑").clicked() { let _ = db::delete_boq_item(&self.conn, it.id); }
                        });
                        grand += it.total;
                    }
                    if items.is_empty() { ui.label("ยังไม่มีรายการ"); }
                });
                ui.separator();
                ui.label(egui::RichText::new(format!("รวมทั้งสิ้น: {:.2} บาท", grand)).size(20.0).strong().color(egui::Color32::from_rgb(255, 210, 90)));
            }
        } else { ui.label("เลือกโครงการเพื่อดู BOQ"); }
    }

    fn pdf_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("ถอดแบบ PDF");
        ui.horizontal(|ui| {
            ui.label("ไฟล์ PDF:");
            ui.text_edit_singleline(&mut self.pdf_path);
            if ui.button("📂 เปิด").clicked() {
                match crate::pdf::PdfViewer::new(&self.pdf_path) {
                    Ok(v) => { self.viewer = Some(v); self.measure_points.clear(); self.status_msg = "เปิด PDF สำเร็จ".into(); }
                    Err(e) => self.status_msg = format!("เปิด PDF ไม่สำเร็จ: {e}"),
                }
            }
        });
        if let Some(viewer) = &mut self.viewer {
            ui.horizontal(|ui| {
                ui.label("หน้า:");
                ui.label(format!("{}/{}", viewer.page_index + 1, viewer.total_pages));
                if ui.button("◀").clicked() && viewer.page_index > 0 { viewer.page_index -= 1; }
                if ui.button("▶").clicked() && viewer.page_index + 1 < viewer.total_pages { viewer.page_index += 1; }
                ui.label("ซูม:");
                ui.add(egui::Slider::new(&mut viewer.scale, 0.5..=4.0).suffix("x"));
            });
            match viewer.render_page() {
                Ok((bytes, w, h)) => {
                    let ctx = ui.ctx().clone();
                    let image = egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &bytes);
                    let texture = ctx.load_texture("pdf_page", image, egui::TextureOptions::default());
                    let (rect, response) = ui.allocate_exact_size(egui::vec2(w as f32, h as f32), egui::Sense::click());
                    ui.painter().image(texture.id(), rect, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), egui::Color32::WHITE);
                    if response.clicked() {
                        if let Some(pos) = response.interact_pointer_pos() {
                            let local = pos - rect.min;
                            self.measure_points.push((local.x, local.y));
                        }
                    }
                    if self.measure_points.len() >= 2 {
                        let p = self.measure_points[self.measure_points.len()-2];
                        let q = self.measure_points[self.measure_points.len()-1];
                        let dist = ((p.0-q.0).powi(2) + (p.1-q.1).powi(2)).sqrt();
                        ui.label(egui::RichText::new(format!("ระยะ: {:.1} px", dist)).color(egui::Color32::from_rgb(120, 220, 140)));
                    }
                    // คำนวณพื้นที่ (polygon)
                    if self.measure_points.len() >= 3 {
                        let n = self.measure_points.len();
                        let mut area = 0.0_f64;
                        for i in 0..n {
                            let j = (i + 1) % n;
                            area += (self.measure_points[i].0 as f64 * self.measure_points[j].1 as f64)
                                   - (self.measure_points[j].0 as f64 * self.measure_points[i].1 as f64);
                        }
                        let area = area.abs() / 2.0;
                        ui.label(egui::RichText::new(format!("พื้นที่: {:.1} px²", area)).color(egui::Color32::from_rgb(120, 190, 255)));
                        if ui.button("📤 ส่งเข้า BOQ").clicked() {
                            if self.b_project == 0 {
                                self.status_msg = "เลือกโครงการในแท็บ BOQ ก่อน".into();
                            } else if self.b_material == 0 {
                                self.status_msg = "เลือกวัสดุในแท็บ BOQ ก่อน".into();
                            } else {
                                let price: f64 = self.b_price.parse().unwrap_or(0.0);
                                match db::add_boq_item(&self.conn, self.b_project, self.b_material, area, price) {
                                    Ok(_) => self.status_msg = format!("ส่งพื้นที่ {:.1} px² เข้า BOQ แล้ว", area),
                                    Err(e) => self.status_msg = format!("ผิดพลาด: {e}"),
                                }
                            }
                        }
                    }
                    if ui.button("🗑 ล้างจุดวัด").clicked() { self.measure_points.clear(); }
                }
                Err(e) => { ui.label(format!("Render ไม่สำเร็จ: {e}")); }
            }
        } else {
            ui.label("กรอก path ไฟล์ PDF แล้วกดเปิด");
        }
    }
}
