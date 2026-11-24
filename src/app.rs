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
    measure_points: Vec<(f64, f64)>,
    scale_input: String,
    smart_mode: i32,
    smart_count: i32,
    smart_rect: Option<(egui::Pos2, egui::Pos2)>,
    smart_text: String,
    smart_annotations: Vec<(egui::Pos2, String)>,
    s_diameter: String, s_length: String, s_count: String,
    loaded_measurement: Option<crate::db::TakeoffMeasurement>,
}

impl ThaiEstimateApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "noto_thai".into(),
            egui::FontData::from_static(include_bytes!("../assets/NotoSansThai-Regular.ttf")),
        );
        for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
            fonts.families.entry(family).or_default().insert(0, "noto_thai".into());
        }
        _cc.egui_ctx.set_fonts(fonts);
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
            scale_input: String::new(),
            smart_mode: 0,
            smart_count: 0,
            smart_rect: None,
            smart_text: String::new(),
            smart_annotations: vec![],
            s_diameter: String::new(), s_length: String::new(), s_count: String::new(),
            loaded_measurement: None,
        }
    }
}

impl eframe::App for ThaiEstimateApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // โทนสีน้ำเงินเข้ม — มืออาชีพ
        let mut style = (*ctx.style()).clone();
        style.visuals.dark_mode = true;
        style.visuals.panel_fill = egui::Color32::from_rgb(15, 23, 42);
        style.visuals.window_fill = egui::Color32::from_rgb(30, 41, 59);
        style.visuals.extreme_bg_color = egui::Color32::from_rgb(11, 18, 32);
        style.visuals.faint_bg_color = egui::Color32::from_rgb(30, 41, 59);
        style.visuals.selection.bg_fill = egui::Color32::from_rgb(37, 99, 235);
        style.visuals.hyperlink_color = egui::Color32::from_rgb(96, 165, 250);
        style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(30, 41, 59);
        style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(30, 41, 59);
        style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(51, 65, 85);
        style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(37, 99, 235);
        style.spacing.item_spacing = egui::vec2(10.0, 10.0);
        style.spacing.button_padding = egui::vec2(12.0, 8.0);
        ctx.set_style(style);

        // Sidebar
        egui::SidePanel::left("sidebar").exact_width(232.0).resizable(false).frame(egui::Frame::side_top_panel(&ctx.style()).inner_margin(egui::Margin::same(18.0))).show(ctx, |ui| {
            ui.add_space(8.0);
            ui.label(egui::RichText::new("ThaiEstimate").size(23.0).strong().color(egui::Color32::from_rgb(147, 197, 253)));
            ui.label(egui::RichText::new("ถอดแบบและประมาณราคา").size(13.0).color(egui::Color32::from_rgb(148, 163, 184)));
            ui.separator();
            ui.add_space(8.0);
            let items = [
                (0, "📋 โครงการ", "จัดการโครงการ"),
                (1, "🧱 คลังราคา", "ฐานข้อมูลวัสดุ"),
                (2, "📊 BOQ", "ใบประมาณราคา"),
                (3, "📐 ถอดแบบ", "วัดระยะ/พื้นที่/Smart"),
                (4, "🔩 เหล็ก", "แผนตัดเหล็ก"),
            ];
            for (id, label, desc) in &items {
                let selected = self.tab == *id;
                let btn = egui::Button::new(egui::RichText::new(*label).size(16.0).strong().color(if selected { egui::Color32::WHITE } else { egui::Color32::from_rgb(203, 213, 225) }))
                    .fill(if selected { egui::Color32::from_rgb(30, 90, 160) } else { egui::Color32::from_rgb(22, 30, 48) })
                    .stroke(egui::Stroke::new(1.0, if selected { egui::Color32::from_rgb(90, 170, 255) } else { egui::Color32::from_rgb(40, 50, 70) }))
                    .rounding(egui::Rounding::same(10.0));
                let resp = ui.add(btn.min_size(egui::vec2(196.0, 48.0)));
                if resp.clicked() { self.tab = *id; }
                ui.label(egui::RichText::new(*desc).size(12.0).color(egui::Color32::from_rgb(148, 163, 184)));
                ui.add_space(8.0);
            }
            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                ui.label(egui::RichText::new("v1.0.0").size(10.0).color(egui::Color32::from_rgb(100, 110, 130)));
                ui.add_space(8.0);
            });
        });

        egui::CentralPanel::default().frame(egui::Frame::central_panel(&ctx.style()).inner_margin(egui::Margin::same(26.0))).show(ctx, |ui| {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                let title = match self.tab {
                    0 => "โครงการ", 1 => "คลังราคา", 2 => "BOQ — ใบประมาณราคา",
                    3 => "ถอดแบบ PDF", 4 => "แผนตัดเหล็ก", _ => "",
                };
                ui.label(egui::RichText::new(title).size(27.0).strong().color(egui::Color32::from_rgb(241, 245, 249)));
            });
            ui.separator();
            ui.add_space(14.0);

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
            // ส่วนวัดแบบที่บันทึกไว้ — คลิกเพื่อย้อนกลับไปดูบนแบบ
            ui.separator();
            ui.label(egui::RichText::new("📐 การวัดที่บันทึกไว้").size(16.0).strong().color(egui::Color32::from_rgb(147, 197, 253)));
            if let Ok(measurements) = db::list_takeoff_measurements(&self.conn, self.b_project) {
                if measurements.is_empty() {
                    ui.label("ยังไม่มีการวัดที่บันทึกไว้");
                } else {
                    egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                        for m in &measurements {
                            ui.horizontal(|ui| {
                                let label = format!("หน้า {} | {} | {:.2} {}", m.page_index + 1, m.measure_type, m.value, if m.measure_type == "area" { "m²" } else { "m" });
                                if ui.button(egui::RichText::new(&label).size(13.0)).clicked() {
                                    self.loaded_measurement = Some(m.clone());
                                    self.tab = 3; // ไปแท็บถอดแบบ
                                    self.status_msg = format!("โหลดการวัด: {}", label);
                                }
                                if ui.button("🗑").clicked() {
                                    let _ = db::delete_takeoff_measurement(&self.conn, m.id);
                                }
                            });
                        }
                    });
                }
            }
        } else { ui.label("เลือกโครงการเพื่อดู BOQ"); }
    }

    fn pdf_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("ถอดแบบ PDF");
        // โหลดการวัดที่เลือกจาก BOQ
        if let Some(m) = self.loaded_measurement.take() {
            self.pdf_path = m.pdf_path.clone();
            if let Ok(mut v) = crate::pdf::PdfViewer::new(&m.pdf_path) {
                v.page_index = m.page_index;
                let _ = v.update_page_dims();
                v.set_scale_for_page(m.page_index, m.real_unit_per_px);
                self.viewer = Some(v);
                if let Ok(points) = serde_json::from_str::<Vec<(f64, f64)>>(&m.points_json) {
                    self.measure_points = points;
                }
                self.status_msg = format!("โหลดการวัด: {} (หน้า {})", m.label, m.page_index + 1);
            }
        }
        ui.horizontal(|ui| {
            ui.label("ไฟล์ PDF:");
            ui.text_edit_singleline(&mut self.pdf_path);
            if ui.button("📂 เปิด").clicked() {
                match crate::pdf::PdfViewer::new(&self.pdf_path) {
                    Ok(v) => { self.viewer = Some(v); self.measure_points.clear(); self.smart_rect = None; self.smart_count = 0; self.smart_annotations.clear(); self.status_msg = "เปิด PDF สำเร็จ".into(); }
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
                ui.selectable_value(&mut self.smart_mode, 4, "📐 ตั้งมาตราส่วน");
            });
            ui.horizontal(|ui| {
                ui.label("หน้า:");
                ui.label(format!("{}/{}", viewer.page_index + 1, viewer.total_pages));
                if ui.button("◀").clicked() && viewer.page_index > 0 { viewer.page_index -= 1; let _ = viewer.update_page_dims(); }
                if ui.button("▶").clicked() && viewer.page_index + 1 < viewer.total_pages { viewer.page_index += 1; let _ = viewer.update_page_dims(); }
                ui.label("ซูม:");
                ui.add(egui::Slider::new(&mut viewer.scale, 0.5..=4.0).suffix("x"));
                ui.label("มาตราส่วน:");
                ui.label(format!("{:.4} m/px", viewer.current_scale()));
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
                            let (pdf_x, pdf_y) = viewer.screen_to_pdf(local.x, local.y);
                            match self.smart_mode {
                                0 => self.measure_points.push((pdf_x, pdf_y)),
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
                                4 => {
                                    // ตั้งมาตราส่วน: เก็บ 2 จุด
                                    self.measure_points.push((pdf_x, pdf_y));
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
                    // วาดจุดวัด (แปลงจากพิกัด PDF กลับเป็นพิกัดหน้าจอ)
                    for &(px, py) in &self.measure_points {
                        let (sx, sy) = viewer.pdf_to_screen(px, py);
                        let screen_pos = rect.min + egui::vec2(sx, sy);
                        ui.painter().circle_filled(screen_pos, 4.0, egui::Color32::from_rgb(255, 100, 100));
                    }
                    if self.measure_points.len() >= 2 {
                        let p = self.measure_points[self.measure_points.len()-2];
                        let q = self.measure_points[self.measure_points.len()-1];
                        let dist_pdf = ((p.0-q.0).powi(2) + (p.1-q.1).powi(2)).sqrt();
                        let scale = viewer.current_scale();
                        let dist = dist_pdf * scale;
                        ui.label(egui::RichText::new(format!("ระยะ: {:.2} m ({:.1} pdf-px)", dist, dist_pdf)).color(egui::Color32::from_rgb(120, 220, 140)));
                    }
                    // คำนวณพื้นที่ (polygon)
                    if self.measure_points.len() >= 3 && self.smart_mode == 0 {
                        let n = self.measure_points.len();
                        let mut area = 0.0_f64;
                        for i in 0..n {
                            let j = (i + 1) % n;
                            area += (self.measure_points[i].0 * self.measure_points[j].1)
                                   - (self.measure_points[j].0 * self.measure_points[i].1);
                        }
                        let area_pdf = area.abs() / 2.0;
                        let scale = viewer.current_scale();
                        let area = area_pdf * scale.powi(2);
                        ui.label(egui::RichText::new(format!("พื้นที่: {:.2} m² ({:.1} pdf-px²)", area, area_pdf)).color(egui::Color32::from_rgb(120, 190, 255)));
                        if ui.button("📤 ส่งเข้า BOQ").clicked() {
                            if self.b_project == 0 {
                                self.status_msg = "เลือกโครงการในแท็บ BOQ ก่อน".into();
                            } else if self.b_material == 0 {
                                self.status_msg = "เลือกวัสดุในแท็บ BOQ ก่อน".into();
                            } else {
                                let price: f64 = self.b_price.parse().unwrap_or(0.0);
                                match db::add_boq_item(&self.conn, self.b_project, self.b_material, area, price) {
                                    Ok(_) => self.status_msg = format!("ส่งพื้นที่ {:.1} m² เข้า BOQ แล้ว", area),
                                    Err(e) => self.status_msg = format!("ผิดพลาด: {e}"),
                                }
                            }
                        }
                    }
                    // ตั้งมาตราส่วน: กรอกระยะจริง
                    if self.smart_mode == 4 && self.measure_points.len() >= 2 {
                        let p = self.measure_points[self.measure_points.len()-2];
                        let q = self.measure_points[self.measure_points.len()-1];
                        let dist_pdf = ((p.0-q.0).powi(2) + (p.1-q.1).powi(2)).sqrt();
                        ui.horizontal(|ui| {
                            ui.label("ระยะจริง (m):");
                            ui.text_edit_singleline(&mut self.scale_input);
                            if ui.button("✅ ตั้งมาตราส่วน").clicked() {
                                if let Ok(real_dist) = self.scale_input.parse::<f64>() {
                                    let new_scale = real_dist / dist_pdf;
                                    viewer.set_scale_for_page(viewer.page_index, new_scale);
                                    self.status_msg = format!("ตั้งมาตราส่วนหน้า {} = {:.4} m/px", viewer.page_index + 1, new_scale);
                                    self.measure_points.clear();
                                } else {
                                    self.status_msg = "กรอกระยะจริงไม่ถูกต้อง".into();
                                }
                            }
                        });
                    }
                    // Smart Area display
                    if self.smart_mode == 1 {
                        if let Some((a, b)) = self.smart_rect {
                            let w = (b.x - a.x).abs() as f64 / viewer.scale as f64 * viewer.current_scale();
                            let h = (b.y - a.y).abs() as f64 / viewer.scale as f64 * viewer.current_scale();
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
                    // บันทึกการวัด
                    if ui.button("💾 บันทึกการวัด").clicked() {
                        if self.b_project == 0 {
                            self.status_msg = "เลือกโครงการในแท็บ BOQ ก่อน".into();
                        } else if self.measure_points.len() >= 2 {
                            let points_json = serde_json::to_string(&self.measure_points).unwrap_or_default();
                            let scale = viewer.current_scale();
                            let n = self.measure_points.len();
                            let mut value = 0.0;
                            if self.smart_mode == 0 && n >= 3 {
                                let mut area = 0.0_f64;
                                for i in 0..n {
                                    let j = (i + 1) % n;
                                    area += (self.measure_points[i].0 * self.measure_points[j].1) - (self.measure_points[j].0 * self.measure_points[i].1);
                                }
                                value = area.abs() / 2.0 * scale.powi(2);
                            } else if n >= 2 {
                                let p = self.measure_points[n-2];
                                let q = self.measure_points[n-1];
                                value = ((p.0-q.0).powi(2) + (p.1-q.1).powi(2)).sqrt() * scale;
                            }
                            let label = format!("หน้า {} โหมด {}", viewer.page_index + 1, self.smart_mode);
                            match db::add_takeoff_measurement(&self.conn, self.b_project, &viewer.path, viewer.page_index, if self.smart_mode == 0 { "area" } else { "distance" }, &points_json, scale, value, &label) {
                                Ok(_) => self.status_msg = "บันทึกการวัดแล้ว".into(),
                                Err(e) => self.status_msg = format!("ผิดพลาด: {e}"),
                            }
                        } else {
                            self.status_msg = "ต้องมีจุดวัดอย่างน้อย 2 จุด".into();
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
