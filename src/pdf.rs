// โมดูล PDF viewer + เครื่องมือวัด (Phase 3)
use anyhow::Result;
use pdfium_render::prelude::*;

pub struct PdfViewer {
    pub path: String,
    pub scale: f32,
    pub real_unit_per_px: f64,
    pub page_index: i32,
    pub total_pages: i32,
    pub points: Vec<(f32, f32)>,
}

fn pdfium() -> Result<Pdfium> {
    let bindings = Pdfium::bind_to_library("/home/linuxbrew/.linuxbrew/lib/python3.14/site-packages/pypdfium2_raw/libpdfium.so")
        .or_else(|_| Pdfium::bind_to_system_library())?;
    Ok(Pdfium::new(bindings))
}

impl PdfViewer {
    pub fn new(path: &str) -> Result<Self> {
        let pdfium = pdfium()?;
        let pdf = pdfium.load_pdf_from_file(path, None)?;
        let total = pdf.pages().len();
        Ok(Self { path: path.to_string(), scale: 1.5, real_unit_per_px: 0.01, page_index: 0, total_pages: total, points: vec![] })
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
