use eframe::egui;
use eframe::App;
use std::error::Error;
use std::fs::File;
use csv::ReaderBuilder;

struct MyApp;

impl App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello, World!");
            ui.label("This is your first eframe app.");
        });
    }
}


struct ProbabilityRecord {
    min_diff: u16,
    max_diff: u16,
    prob_big: u8,
    prob_small: u8,
}

impl ProbabilityRecord {
    fn new() -> Self {
        Self {
            min_diff: 0,
            max_diff: 0,
            prob_big: 0,
            prob_small: 0,
        }
    }
    
    fn new_from(min_diff: u16, max_diff: u16, prob_big: u8, prob_small: u8) -> Self {
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
            record[0].parse()?,
            record[1].parse()?,
            record[2].parse()?,
            record[3].parse()?,
        ))
    }
}

fn load_from_csv(file: &str) -> Result<Vec<ProbabilityRecord>, Box<dyn Error>> {
    let file = File::open(file)?;
    let mut rdr = ReaderBuilder::new()
        .delimiter(b',') 
        .has_headers(true)
        .from_reader(file);

    let mut records = Vec::new();
    for result in rdr.records() {
        let record = result?;
        records.push(ProbabilityRecord::from_csv_record(&record)?);
    }
    Ok(records)
}

fn main() {
    let native_options = eframe::NativeOptions::default();
    let _ =
    eframe::run_native(
        "Hello World",
        native_options,
        Box::new(|_cc| Box::new(MyApp)),
    );

    let probability = load_from_csv(r"src\table.csv").unwrap();

}
