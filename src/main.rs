/* FIDE Elo Rating Calculator */

/// CSV
use std::fs::File;
use csv::ReaderBuilder;
/// CSV + GUI
use std::thread::spawn;
use std::sync::mpsc::{channel, Sender, Receiver};
/// GUI
use eframe::egui;
use eframe::App as EframeApp;
use egui::ViewportBuilder;
use std::error::Error;

/////////////////////////////////////////////////////////////////////////////////
/// CSV

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

////////////////////////////////////////////////////////////////////////////////
/// GUI

struct Message {
    text: String,
    data: Option<AppState>,
    msg: Option<String>,
}

struct AppState {
    my_rating: u16,
    k_factor: u16,
    opponents_rating: Vec<u16>,
    is_eighteen: bool,
    played_in_tour_with_at_least_30_games: bool,
    had_2300: bool,
    had_2400: bool,
}

struct App {
    last_record: AppState,
    actual_record: AppState,
    tx: Sender<Message>,
    rx: Receiver<Message>,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let (tx, rxforcsv) = channel();
        let (txforcsv, rx) = channel();
        spawn(move || {
            fn main_loop(tx: Sender<Message>, rx: Receiver<Message>) -> Result<(), Box<dyn Error>> {
                let send = |msg: Message, tx: Sender<Message>| -> Result<(), Box<dyn Error>> {
                    tx.send(msg).map_err(|_| return Err("Failed to send message".into()));

                }
                
                txforcsv.send(Message {
                    text: "upsplash".to_string(),
                    data: None,
                    msg: Some("Wait, files are loading...|by N".to_string()),
                });
                let res = load_from_csv("probabilities.csv");
                
                let records;
                match res {
                    Ok(r) => {records = r;},
                    Err(_) => {
                        txforcsv.send(Message {
                            text: "upsplash".to_string(),
                            data: None,
                            msg: Some("Error!|The `probabilities.csv` file is missing or inaccessible.|OK".to_string()),
                        });
                        break 'c;
                    }
                }
                for i in 0..=5000 {
                    if *(&records.iter().all(|r| r.min_diff <= i && r.max_diff >= i)) {
                        txforcsv.send(Message {
                            text: "upsplash".to_string(),
                            data: None,
                            msg: Some("Error!|The `probabilities.csv` file is invalid.|OK".to_string()),
                            break 'c
                        });
                    }
                }
                loop {
                    if let Ok(message) =rxforcsv.try_recv() {
                        match message.text.to_str() {
                            "close" => break 'c
                            "calc"
                        }
                    }
                }
            }
        });
    }
}

impl EframeApp for App { 
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




fn main() {
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder {
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
        Box::new(|cc| Box::new(App::new(cc))),
    );
    match runner {
        Ok(_) => {},
        Err(e) => eprintln!("Error running the application: {}", e),
    }

}
