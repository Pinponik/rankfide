/* FIDE Elo Rating Calculator */

use csv::ReaderBuilder;
use eframe::App as EframeApp;
/// GUI
use eframe::egui;
use eframe::egui::Widget;
use egui::ViewportBuilder;
use std::error::Error;
/// CSV
use std::fs::File;
use std::sync::mpsc::{Receiver, Sender, channel};
/// CSV + GUI
use std::thread::spawn;

/////////////////////////////////////////////////////////////////////////////////
/// CSV

struct ProbabilityRecord {
    min_diff: u16,
    max_diff: u16,
    prob_big: f32,
    prob_small: f32,
}

impl ProbabilityRecord {
    #[allow(dead_code)]
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

#[derive(Copy, Clone)]
struct OnePlayerGame {
    opponent_rating: u16,
    result: f32,
}

#[derive(Clone)]
struct AppState {
    has_rating: bool,
    my_rating: u16,
    k_factor: u16,
    games: Vec<OnePlayerGame>,
    is_eighteen: bool,
    played_in_tour_30_games: bool,
    had_2300: bool,
    had_2400: bool,
    games_text: String,
}

struct App {
    last_record: AppState,
    current_record: AppState,
    tx: Sender<Message>,
    rx: Receiver<Message>,
    manually: bool,
    splash: bool,
    splash_msg: String,
    rating: String,
    wait: bool,
    text: Vec<String>,
}

impl App {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (tx, rxforcsv) = channel();
        let (txforcsv, rx) = channel();
        spawn(move || {
            fn main_loop(
                sender: Sender<Message>,
                receiver: Receiver<Message>,
            ) -> Result<(), Box<dyn Error>> {
                let send =
                    |msg: Message, tx: &Sender<Message>| -> Result<(), Box<dyn Error + 'static>> {
                        tx.send(msg)
                            .map_err(|e| Box::new(e) as Box<dyn Error + 'static>)
                    };

                send(
                    Message {
                        text: "upsplash".to_string(),
                        data: None,
                        msg: Some("Wait, files are loading...|by N".to_string()),
                    },
                    &sender,
                )?;

                let records = match load_from_csv("probabilities.csv") {
                    Ok(r) => r,
                    Err(_) => {
                        send(Message {
                            text: "upsplash".to_string(),
                            data: None,
                            msg: Some("Error!|The `probabilities.csv` file is missing or inaccessible.|OK".to_string()),
                        }, &sender)?;
                        return Ok(());
                    }
                };

                for i in 0..=5000 {
                    if !records.iter().any(|r| r.min_diff <= i && r.max_diff >= i) {
                        send(
                            Message {
                                text: "upsplash".to_string(),
                                data: None,
                                msg: Some(
                                    "Error!|The `probabilities.csv` file is invalid.|OK"
                                        .to_string(),
                                ),
                            },
                            &sender,
                        )?;
                        return Ok(());
                    }
                }

                send(
                    Message {
                        text: "downsplash".to_string(),
                        data: None,
                        msg: None,
                    },
                    &sender,
                )?;

                loop {
                    if let Ok(message) = receiver.recv() {
                        match message.text.as_str() {
                            "close" => return Ok(()),
                            "k-factor" => {
                                if let Some(mut data) = message.data {
                                    data.k_factor =
                                        if !(data.is_eighteen && data.played_in_tour_30_games) {
                                            10
                                        } else if !(data.is_eighteen && data.had_2300) {
                                            40
                                        } else if !(data.had_2400) {
                                            20
                                        } else {
                                            10
                                        };

                                    send(
                                        Message {
                                            text: "k-factor".to_string(),
                                            data: None,
                                            msg: Some(data.k_factor.to_string()),
                                        },
                                        &sender,
                                    )?;
                                }
                            }
                            "calc" => {
                                if let Some(data) = message.data {
                                    if data.games.is_empty() {
                                        send(
                                            Message {
                                                text: "calc".to_string(),
                                                data: None,
                                                msg: Some(
                                                    "The rating will change by 0 points."
                                                        .to_string(),
                                                ),
                                            },
                                            &sender,
                                        )?;
                                        continue;
                                    }

                                    if !data.has_rating {
                                        send(
                                            Message {
                                                text: "calc".to_string(),
                                                data: None,
                                                msg: Some(
                                                    "Currently, the program does not support calculating rating changes for unrated players."
                                                        .to_string(),
                                                ),
                                            },
                                            &sender,
                                        )?;
                                        continue;
                                    }

                                    for game in data.games.iter() {
                                        if !(game.result == 1.0
                                            || game.result == 0.5
                                            || game.result == 0.0)
                                        {
                                            send(
                                                Message {
                                                    text: "calc".to_string(),
                                                    data: None,
                                                    msg: Some(
                                                        "Game result must be known and can only be Win, Draw, or Loss."
                                                            .to_string(),
                                                    ),
                                                },
                                                &sender,
                                            )?;
                                            return Ok(());
                                        }
                                    }

                                    let mut sum: f64 = 0.0;
                                    for game in data.games.iter() {
                                        let diff = data.my_rating.abs_diff(game.opponent_rating);
                                        let bigger = data.my_rating > game.opponent_rating;
                                        let prob = records
                                            .iter()
                                            .find(|r| r.min_diff <= diff && r.max_diff >= diff)
                                            .unwrap();
                                        sum += game.result as f64
                                            - if bigger {
                                                prob.prob_big as f64
                                            } else {
                                                prob.prob_small as f64
                                            };
                                    }
                                    send(
                                        Message {
                                            text: "calc".to_string(),
                                            data: None,
                                            msg: Some(format!(
                                                "The rating will change by {} points.",
                                                (data.k_factor as f64 * sum)
                                            )),
                                        },
                                        &sender,
                                    )?;
                                }
                            }
                            _ => {}
                        }
                    } else {
                        return Ok(());
                    }
                }
            }

            let _ = main_loop(txforcsv, rxforcsv);
        });
        Self {
            last_record: AppState {
                has_rating: false,
                my_rating: 0,
                k_factor: 0,
                games: Vec::new(),
                is_eighteen: false,
                played_in_tour_30_games: false,
                had_2300: false,
                had_2400: false,
                games_text: "".to_string(),
            },
            current_record: AppState {
                has_rating: true,
                my_rating: 0,
                k_factor: 0,
                games: Vec::new(),
                is_eighteen: false,
                played_in_tour_30_games: false,
                had_2300: false,
                had_2400: false,
                games_text: "".to_string(),
            },
            tx,
            rx,
            manually: false,
            splash: true,
            splash_msg: "".to_string(),
            rating: "".to_string(),
            wait: false,
            text: Vec::new(),
        }
    }
}

impl EframeApp for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input(|i| i.viewport().close_requested()) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        if let Ok(data) = self.rx.try_recv()
            && !self.wait
        {
            match data.text.as_str() {
                "upsplash" => {
                    self.splash = true;
                    self.splash_msg = data.msg.unwrap_or("".to_string());
                    self.text = self.splash_msg.split('|').map(|s| s.to_string()).collect();
                }
                "downsplash" => {
                    self.splash = false;
                    self.splash_msg = "".to_string();
                    ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(
                        700.0, 500.0,
                    )));
                }
                "k-factor" => {
                    self.current_record.k_factor = data
                        .msg
                        .unwrap_or("0".to_string())
                        .parse::<u16>()
                        .unwrap_or(0);
                }
                "calc" => {
                    self.rating = data.msg.unwrap_or("".to_string());
                }
                _ => {}
            }
        } else if self
            .tx
            .send(Message {
                text: "test".to_string(),
                data: None,
                msg: None,
            })
            .is_err()
            && !self.wait
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.splash {
                let mut text = Vec::new();
                for i in 0..self.text.len() {
                    text.push(self.text[i].clone());
                }
                ui.heading("FIDE Elo Rating Calculator");
                let mut layout = if !text.is_empty() {
                    text[0].clone()
                } else {
                    "".to_string()
                };
                if text.len() > 1 {
                    layout.push_str(&text[1]);
                }
                ui.label(layout);
                if self.text.len() > 2 {
                    self.wait = true;
                    if ui.button(text[2].clone()).clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                }
            } else {
                egui::TopBottomPanel::top("custom_title_bar").show(ctx, |ui| {
                    let height = 28.0;
                    let (rect, response) = ui.allocate_exact_size(
                        egui::vec2(ui.available_width(), height),
                        egui::Sense::click_and_drag(),
                    );
                    ui.painter()
                        .rect_filled(rect, 0.0, ui.visuals().window_fill());
                    ui.allocate_ui_at_rect(rect, |ui| {
                        ui.horizontal(|ui| {
                            if ui.button("X").clicked() {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                            if ui.button("–").clicked() {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                            }
                            ui.add_space((ui.available_width() / 2.0 - 40.0) - 40.0);
                            ui.label("FIDE Elo Rating Calculator");
                            ui.add_space((ui.available_width() / 2.0 - 40.0) - 40.0);
                        });
                    });
                    if response.dragged() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                    }
                });

                egui::CentralPanel::default().show(ctx, |_ui| {});
                ui.add_space(40.0);
                ui.horizontal(|ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.vertical_centered_justified(|ui| {
                            for (index, game) in self.current_record.games.iter_mut().enumerate() {
                                ui.horizontal(|ui| {
                                    ui.add(|ui: &mut egui::Ui| {
                                        egui::DragValue::new(&mut game.opponent_rating)
                                            .clamp_range(1400..=5000)
                                            .prefix("Opponent rating: ")
                                            .ui(ui)
                                    });
                                    ui.add(|ui: &mut egui::Ui| {
                                        egui::ComboBox::from_id_source(index)
                                            .selected_text(match game.result {
                                                1.0 => "Win",
                                                0.5 => "Draw",
                                                0.0 => "Loss",
                                                _ => "Unknown",
                                            })
                                            .show_ui(ui, |ui: &mut egui::Ui| {
                                                ui.selectable_value(&mut game.result, 1.0, "Win");
                                                ui.selectable_value(&mut game.result, 0.5, "Draw");
                                                ui.selectable_value(&mut game.result, 0.0, "Loss");
                                            })
                                            .response
                                    });
                                });
                                ui.separator();
                            }
                            if ui.button("Add Game").clicked() {
                                self.current_record.games.push(OnePlayerGame {
                                    opponent_rating: 1400,
                                    result: -1.0,
                                });
                            }
                        });
                    });
                    ui.separator();
                    ui.vertical(|ui: &mut egui::Ui| {
                        ui.checkbox(&mut self.manually, "Manually");
                        ui.add_enabled(!self.manually, |ui: &mut egui::Ui| {
                            ui.checkbox(
                                &mut self.current_record.is_eighteen,
                                "Is 18 years old or older",
                            );
                            ui.checkbox(
                                &mut self.current_record.played_in_tour_30_games,
                                "Played at least 30 games in the tournament",
                            );
                            ui.checkbox(
                                &mut self.current_record.had_2300,
                                "Had a rating of at least 2300",
                            );
                            ui.add_enabled(self.current_record.had_2300, |ui: &mut egui::Ui| {
                                ui.checkbox(
                                    &mut self.current_record.had_2400,
                                    "Had a rating of at least 2400",
                                )
                            });
                            ui.separator()
                        });
                        ui.checkbox(&mut self.current_record.has_rating, "Have a rating");
                        if !self.current_record.had_2300 {
                            self.current_record.had_2400 = false;
                        }
                        ui.add_enabled(self.current_record.has_rating, |ui: &mut egui::Ui| {
                            ui.add(
                                egui::DragValue::new(&mut self.current_record.my_rating)
                                    .clamp_range(1400..=5000)
                                    .prefix("Have rating: "),
                            )
                        });
                        ui.add_space(10.0);
                        ui.label(self.rating.clone());
                    });
                });
            }
        });
    }
}

fn main() {
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder {
            title: Some("FIDE Elo Rating Calculator".to_string()),
            decorations: Some(false),
            resizable: Some(false),
            min_inner_size: Some(egui::vec2(270.0, 80.0)),
            inner_size: Some(egui::vec2(270.0, 80.0)),
            ..Default::default()
        },
        //centered: true,
        ..Default::default()
    };

    let runner = eframe::run_native(
        "FIDE Elo Rating Calculator",
        options,
        Box::new(|cc| Box::new(App::new(cc))),
    );
    match runner {
        Ok(_) => {}
        Err(e) => eprintln!("Error running the application: {}", e),
    }
}
