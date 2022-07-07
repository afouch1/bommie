mod models;
mod json;

use models::*;
use json::*;

use egui;
use eframe;

use rfd::FileDialog;

use serde_json::{Result};

const SAMPLE_JSON: &'static str = r#"{
    "1": {
        "BFO24I": 234,
        "BFO COIL": 150
    }
}"#;

fn parse_json(json: &str) -> Result<Vec<Print>> {
    let dict: PrintDictionary = serde_json::from_str(json)?;

    let mut print_vec = Vec::new();

    for (print_name, unit_dict) in dict.iter() {
        let mut unit_vec = Vec::new();
        for (unit_name, unit_quantity) in unit_dict.iter() {
            unit_vec.push(Unit {
                name: unit_name.to_owned(),
                quantity: unit_quantity.clone()
            })
        }
        print_vec.push(Print {
            name: print_name.to_owned(),
            units: unit_vec,
            potential_unit: Unit::default()
        })
    }

    Ok(print_vec)
}

fn main() {
    let result = FileDialog::new()
        .add_filter("units", &["units"])
        .pick_file();

    let prints = if let Some(path) = result {
        if let Ok(contents) = std::fs::read_to_string(path)  {
            match parse_json(contents.as_str()) {
                Ok(ok_prints) => ok_prints,
                _ => parse_json(SAMPLE_JSON).unwrap()
            }
        } else {
            parse_json(SAMPLE_JSON).unwrap()
        }
    } else {
        parse_json(SAMPLE_JSON).unwrap()
    };

    let options = eframe::NativeOptions {
        min_window_size: Some(egui::Vec2::new(1000.0, 600.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Bommie",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Box::new(BommieApp {
                prints,
                ..Default::default()
            })
        })
    )
}

impl eframe::App for BommieApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::SidePanel::left("prints").show(ctx, |ui| {
            self.prints_panel(ui);
        });
    }
}

impl BommieApp {
    fn prints_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Prints");
        for i in 0..self.prints.len() {
            let mut should_break = false;
            let button = egui::Button::new(self.prints[i].name.clone());
            ui.horizontal(|ui| {
                if ui.add_sized(egui::Vec2::new(ui.available_width() - 25.0, 20.0), button).clicked() {
                    self.current_print = i;
                }
                if ui.button("X").clicked() {
                    self.prints.remove(i);
                    should_break = true;
                }
            });
            if should_break { break; }
        }
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut self.potential_print);
            if ui.button("Add").clicked() &&
                self.potential_print.len() != 0 {
                if !self.prints.iter().any(|p| p.name == self.potential_print) {
                    self.prints.push(Print {
                        name: self.potential_print.clone(),
                        ..Default::default()
                    });
                    self.potential_print.clear();
                }
            };
        });
    }
}

