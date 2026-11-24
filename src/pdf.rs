// โมดูล PDF viewer + เครื่องมือวัด (Phase 3 + Phase 12: มาตราส่วน + พิกัด PDF)
use anyhow::Result;
use pdfium_render::prelude::*;
use std::collections::HashMap;

pub struct PdfViewer {
    pub path: String,
    pub scale: f32,
    pub page_index: i32,
    pub total_pages: i32,
    // มาตราส่วนแยกตามหน้า: page_index -> real_unit_per_px
    pub page_scales: HashMap<i32, f64>,
    // เก็บขนาดหน้า (pdf-px) แคชไว้
    pub page_w: f64,
    pub page_h: f64,
}

fn pdfium() -> Result<Pdfium> {
    let exe_dir = std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.to_path_buf()));
    if let Some(dir) = exe_dir {
        let candidates = ["libpdfium.dll", "libpdfium.so", "libpdfium.dylib"];
        for name in &candidates {
            let bundled = dir.join(name);
            if bundled.exists() {
                if let Ok(bindings) = Pdfium::bind_to_library(bundled.to_string_lossy().as_ref()) {
                    return Ok(Pdfium::new(bindings));
                }
            }
        }
    }
    let bindings = Pdfium::bind_to_system_library()
        .or_else(|_| Pdfium::bind_to_library("/home/linuxbrew/.linuxbrew/lib/python3.14/site-packages/pypdfium2_raw/libpdfium.so"))?;
    Ok(Pdfium::new(bindings))
}

impl PdfViewer {
    pub fn new(path: &str) -> Result<Self> {
        let pdfium = pdfium()?;
        let pdf = pdfium.load_pdf_from_file(path, None)?;
        let total = pdf.pages().len();
        let page = pdf.pages().get(0)?;
        let page_w = page.width().value as f64;
        let page_h = page.height().value as f64;
        Ok(Self { path: path.to_string(), scale: 1.5, page_index: 0, total_pages: total, page_scales: HashMap::new(), page_w, page_h })
    }

    pub fn update_page_dims(&mut self) -> Result<()> {
        let pdfium = pdfium()?;
        let pdf = pdfium.load_pdf_from_file(&self.path, None)?;
        let page = pdf.pages().get(self.page_index)?;
        self.page_w = page.width().value as f64;
        self.page_h = page.height().value as f64;
        Ok(())
    }

    pub fn current_scale(&self) -> f64 {
        self.page_scales.get(&self.page_index).copied().unwrap_or(0.01)
    }

    pub fn set_scale_for_page(&mut self, page: i32, real_unit_per_px: f64) {
        self.page_scales.insert(page, real_unit_per_px);
    }

    // แปลงพิกัดหน้าจอ (local) เป็นพิกัด PDF
    pub fn screen_to_pdf(&self, local_x: f32, local_y: f32) -> (f64, f64) {
        let rendered_w = self.page_w * self.scale as f64;
        let rendered_h = self.page_h * self.scale as f64;
        let pdf_x = local_x as f64 / rendered_w * self.page_w;
        let pdf_y = local_y as f64 / rendered_h * self.page_h;
        (pdf_x, pdf_y)
    }

    // แปลงพิกัด PDF เป็นพิกัดหน้าจอ (local)
    pub fn pdf_to_screen(&self, pdf_x: f64, pdf_y: f64) -> (f32, f32) {
        let rendered_w = self.page_w * self.scale as f64;
        let rendered_h = self.page_h * self.scale as f64;
        let screen_x = pdf_x / self.page_w * rendered_w;
        let screen_y = pdf_y / self.page_h * rendered_h;
        (screen_x as f32, screen_y as f32)
    }

    pub fn render_page(&self) -> Result<(Vec<u8>, i32, i32)> {
        let pdfium = pdfium()?;
        let pdf = pdfium.load_pdf_from_file(&self.path, None)?;
        let page = pdf.pages().get(self.page_index)?;
        let w = (page.width().value * self.scale) as i32;
        let h = (page.height().value * self.scale) as i32;
        let config = PdfRenderConfig::new().set_target_width(w).set_target_height(h);
        let bitmap = page.render_with_config(&config)?;
        let bytes = bitmap.as_rgba_bytes();
        Ok((bytes, w, h))
    }
}
