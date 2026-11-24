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
    smart_mode: i32,
    smart_count: i32,
    smart_rect: Option<(egui::Pos2, egui::Pos2)>,
    smart_text: String,
    smart_annotations: Vec<(egui::Pos2, String)>,
    s_diameter: String, s_length: String, s_count: String,
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
            smart_mode: 0,
            smart_count: 0,
            smart_rect: None,
            smart_text: String::new(),
            smart_annotations: vec![],
            s_diameter: String::new(), s_length: String::new(), s_count: String::new(),
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
                ui.selectable_value(&mut self.tab, 4, "🔩 เหล็ก");
            });
            ui.separator();

            match self.tab {
                0 => self.projects_tab(ui),
                1 => self.materials_tab(ui),
                2 => self.boq_tab(ui),
                3 => self.pdf_tab(ui),
                4 => self.steel_tab(ui),
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
        ui.horizontal(|ui| {
            if ui.button("📤 Export JSON").clicked() {
                if self.p_name.is_empty() { self.status_msg = "กรอกข้อมูลโครงการก่อน".into(); }
                else {
                    // find existing project by name
                    if let Ok(projects) = db::list_projects(&self.conn) {
                        if let Some(p) = projects.iter().find(|p| p.name == self.p_name) {
                            match db::export_project(&self.conn, p.id) {
                                Ok(data) => {
                                    let path = format!("project_{}.json", p.code);
                                    match std::fs::write(&path, serde_json::to_string_pretty(&data).unwrap()) {
                                        Ok(_) => self.status_msg = format!("Export แล้ว: {}", path),
                                        Err(e) => self.status_msg = format!("ผิดพลาด: {e}"),
                                    }
                                }
                                Err(e) => self.status_msg = format!("ผิดพลาด: {e}"),
                            }
                        } else { self.status_msg = "ไม่พบโครงการ".into(); }
                    }
                }
            }
            if ui.button("📥 Import JSON").clicked() {
                let path = format!("project_{}.json", self.p_code);
                if let Ok(text) = std::fs::read_to_string(&path) {
                    if let Ok(data) = serde_json::from_str::<db::ProjectExport>(&text) {
                        match db::import_project(&self.conn, &data) {
                            Ok(id) => self.status_msg = format!("Import สำเร็จ (id={})", id),
                            Err(e) => self.status_msg = format!("ผิดพลาด: {e}"),
                        }
                    } else { self.status_msg = "JSON ไม่ถูกต้อง".into(); }
                } else { self.status_msg = format!("ไม่พบไฟล์: {}", path); }
            }
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
                if ui.button("📊 ส่งออก Excel").clicked() {
                    let path = format!("BOQ_{}.xlsx", self.b_project);
                    match crate::report::export_boq_excel(&self.conn, self.b_project, &path) {
                        Ok(_) => self.status_msg = format!("ส่งออก Excel แล้ว: {}", path),
                        Err(e) => self.status_msg = format!("ส่งออกไม่สำเร็จ: {e}"),
                    }
                }
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
                ui.label("โหมด:");
                ui.selectable_value(&mut self.smart_mode, 0, "📏 วัดระยะ");
                ui.selectable_value(&mut self.smart_mode, 1, "📐 Smart Area");
                ui.selectable_value(&mut self.smart_mode, 2, "🔢 Smart Count");
                ui.selectable_value(&mut self.smart_mode, 3, "✏️ มาร์กอัป");
            });
            ui.horizontal(|ui| {
                ui.label("หน้า:");
                ui.label(format!("{}/{}", viewer.page_index + 1, viewer.total_pages));
                if ui.button("◀").clicked() && viewer.page_index > 0 { viewer.page_index -= 1; }
                if ui.button("▶").clicked() && viewer.page_index + 1 < viewer.total_pages { viewer.page_index += 1; }
                ui.label("ซูม:");
                ui.add(egui::Slider::new(&mut viewer.scale, 0.5..=4.0).suffix("x"));
                ui.label("หน่วยจริง/px:");
                ui.add(egui::DragValue::new(&mut viewer.real_unit_per_px).speed(0.001).prefix("").suffix(" m"));
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
                            match self.smart_mode {
                                0 => self.measure_points.push((local.x, local.y)),
                                1 => {
                                    if self.smart_rect.is_none() { self.smart_rect = Some((pos, pos)); }
                                    else { self.smart_rect = Some((self.smart_rect.unwrap().0, pos)); }
                                }
                                2 => { self.smart_count += 1; }
                                3 => {
                                    if !self.smart_text.is_empty() {
                                        self.smart_annotations.push((pos, self.smart_text.clone()));
                                        self.smart_text.clear();
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    // วาด annotation และ rectangle
                    for (pos, text) in &self.smart_annotations {
                        ui.painter().text(*pos, egui::Align2::CENTER_TOP, text, egui::FontId::proportional(14.0), egui::Color32::from_rgb(255, 80, 80));
                    }
                    if let Some((a, b)) = self.smart_rect {
                        let rect2 = egui::Rect::from_two_pos(a, b);
                        ui.painter().rect_stroke(rect2, 0.0, egui::Stroke::new(2.0, egui::Color32::from_rgb(80, 200, 255)));
                    }
                    if self.measure_points.len() >= 2 {
                        let p = self.measure_points[self.measure_points.len()-2];
                        let q = self.measure_points[self.measure_points.len()-1];
                        let dist_px = ((p.0-q.0).powi(2) + (p.1-q.1).powi(2)).sqrt();
                        let dist = dist_px as f64 * viewer.real_unit_per_px;
                        ui.label(egui::RichText::new(format!("ระยะ: {:.2} m ({:.1} px)", dist, dist_px)).color(egui::Color32::from_rgb(120, 220, 140)));
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
                        let area_px = area.abs() / 2.0;
                        let area = area_px * viewer.real_unit_per_px.powi(2);
                        ui.label(egui::RichText::new(format!("พื้นที่: {:.2} m² ({:.1} px²)", area, area_px)).color(egui::Color32::from_rgb(120, 190, 255)));
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
                    // Smart Area display
                    if self.smart_mode == 1 {
                        if let Some((a, b)) = self.smart_rect {
                            let w = (b.x - a.x).abs() as f64 * viewer.real_unit_per_px;
                            let h = (b.y - a.y).abs() as f64 * viewer.real_unit_per_px;
                            ui.label(egui::RichText::new(format!("Smart Area: {:.2} m² ({} x {} m)", w * h, w, h)).color(egui::Color32::from_rgb(80, 200, 255)));
                        }
                        if ui.button("🗑 ล้าง Area").clicked() { self.smart_rect = None; }
                    }
                    // Smart Count display
                    if self.smart_mode == 2 {
                        ui.label(egui::RichText::new(format!("Smart Count: {} จุด", self.smart_count)).color(egui::Color32::from_rgb(255, 200, 80)));
                        if ui.button("🗑 ล้าง Count").clicked() { self.smart_count = 0; }
                    }
                    // Markup display
                    if self.smart_mode == 3 {
                        ui.horizontal(|ui| {
                            ui.label("ข้อความ:"); ui.text_edit_singleline(&mut self.smart_text);
                            if ui.button("🗑 ล้าง Markup").clicked() { self.smart_annotations.clear(); }
                        });
                    }
                    if ui.button("🗑 ล้างจุดวัด").clicked() { self.measure_points.clear(); }
                }
                Err(e) => { ui.label(format!("Render ไม่สำเร็จ: {e}")); }
            }
        } else {
            ui.label("กรอก path ไฟล์ PDF แล้วกดเปิด");
        }
    }

    fn steel_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("แผนตัดเหล็ก (Bar Cut List)");
        let projects = db::list_projects(&self.conn).unwrap_or_default();
        ui.horizontal(|ui| {
            ui.label("โครงการ:");
            egui::ComboBox::from_id_salt("steel_proj").selected_text(if self.b_project == 0 { "เลือก".into() } else { projects.iter().find(|p| p.id == self.b_project).map(|p| p.name.clone()).unwrap_or_default() }).show_ui(ui, |ui| {
                for p in &projects { ui.selectable_value(&mut self.b_project, p.id, &p.name); }
            });
        });
        ui.horizontal(|ui| {
            ui.label("เส้นผ่าศูนย์กลาง (mm):"); ui.text_edit_singleline(&mut self.s_diameter);
            ui.label("ความยาว (m):"); ui.text_edit_singleline(&mut self.s_length);
            ui.label("จำนวน:"); ui.text_edit_singleline(&mut self.s_count);
        });
        if ui.button("➕ เพิ่มเหล็ก").clicked() {
            let d: f64 = self.s_diameter.parse().unwrap_or(0.0);
            let l: f64 = self.s_length.parse().unwrap_or(0.0);
            let c: i64 = self.s_count.parse().unwrap_or(0);
            if self.b_project == 0 { self.status_msg = "เลือกโครงการก่อน".into(); }
            else {
                match db::add_steel_bar(&self.conn, self.b_project, d, l, c) {
                    Ok(_) => { self.status_msg = "เพิ่มเหล็กสำเร็จ".into(); self.s_diameter.clear(); self.s_length.clear(); self.s_count.clear(); }
                    Err(e) => self.status_msg = format!("ผิดพลาด: {e}"),
                }
            }
        }
        ui.separator();
        if self.b_project != 0 {
            if let Ok(bars) = db::list_steel_bars(&self.conn, self.b_project) {
                let mut total_weight = 0.0;
                let mut total_count = 0;
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for b in &bars {
                        let weight = b.weight_per_m * b.length * b.count as f64;
                        total_weight += weight;
                        total_count += b.count;
                        ui.horizontal(|ui| {
                            ui.label(format!("Ø{:.0}mm | {:.2}m | x{} | {:.1}kg/m | น้ำหนักรวม {:.1}kg", b.diameter, b.length, b.count, b.weight_per_m, weight));
                            if ui.button("🗑").clicked() { let _ = db::delete_steel_bar(&self.conn, b.id); }
                        });
                    }
                    if bars.is_empty() { ui.label("ยังไม่มีเหล็ก"); }
                });
                ui.separator();
                ui.label(egui::RichText::new(format!("รวม {} เส้น | น้ำหนักรวม {:.1} kg", total_count, total_weight)).size(18.0).strong().color(egui::Color32::from_rgb(255, 210, 90)));
            }
        } else { ui.label("เลือกโครงการเพื่อดูแผนตัดเหล็ก"); }
    }
}
