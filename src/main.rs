use std::thread::spawn;
use std::sync::mpsc::channel;
use eframe::egui;
use eframe::App;
use egui::ViewportBuilder;
use std::error::Error;
use std::fs::File;
use csv::ReaderBuilder;

struct MyApp;

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input(|i| i.viewport().close_requested()) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("FIDE Elo Rating Calculator");
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui|{
                ui.label("Wait, files are loading..."); 
                ui.separator();
                ui.label("by N")
            });
        });
    }
}


struct ProbabilityRecord {
    min_diff: u16,
    max_diff: u16,
    prob_big: f32,
    prob_small: f32,
}

impl ProbabilityRecord {
    fn new() -> Self {
        Self {
            min_diff: 0,
            max_diff: 0,
            prob_big: 0.0,
            prob_small: 0.0,
        }
    }
    
    fn new_from(min_diff: u16, max_diff: u16, prob_big: f32, prob_small: f32) -> Self {
        Self {
            min_diff,
            max_diff,
            prob_big,
            prob_small,
        }
    }
    
    fn from_csv_record(record: &csv::StringRecord) -> Result<Self, Box<dyn Error>> {
        if record.len() != 4 {
            return Err("Invalid record length".into());
        }
        Ok(Self::new_from(
            record[0].parse::<u16>()?,
            record[1].parse::<u16>()?,
            record[2].parse::<f32>()?,
            record[3].parse::<f32>()?,
        ))
    }
}

fn load_from_csv(file: &str) -> Result<Vec<ProbabilityRecord>, Box<dyn Error>> {
    let file = File::open(file)?;
    let mut rdr = ReaderBuilder::new()
        .delimiter(b',') 
        .has_headers(false)
        .from_reader(file);

    let mut records = Vec::new();
    for result in rdr.records() {
        let record = result?;
        records.push(ProbabilityRecord::from_csv_record(&record)?);
    }
    Ok(records)
}

fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder {
            title: Some("FIDE Elo Rating Calculator".to_string()),
            decorations: Some(false),
            resizable: Some(false),
            max_inner_size: Some(egui::vec2(230.0, 60.0)),
            min_inner_size: Some(egui::vec2(230.0, 60.0)),
            inner_size: Some(egui::vec2(230.0, 60.0)),
            ..Default::default()
        },
        centered: true,
        ..Default::default()
    };
    
    let runner =
    eframe::run_native(
        "FIDE Elo Rating Calculator",
        options,
        Box::new(|_cc| Box::new(MyApp)),
    );

    let probability = load_from_csv(r"table.csv").unwrap();

}
