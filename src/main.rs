mod db;
mod app;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1100.0, 750.0]).with_title("ThaiEstimate — ถอดแบบ & ประมาณราคา"),
        ..Default::default()
    };
    eframe::run_native("ThaiEstimate", options, Box::new(|cc| Ok(Box::new(app::ThaiEstimateApp::new(cc)))))
}
