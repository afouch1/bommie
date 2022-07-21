#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod models;
mod json;

use std::cmp::Ordering;
use std::collections::HashMap;
use std::path::PathBuf;
use models::*;
use json::*;

use egui;
use eframe;

use rfd::FileDialog;

use serde_json::{Result};

fn print_sort(a: &Print, b: &Print) -> Ordering {
    if a.name == b.name {
        return Ordering::Equal;
    }

    if let (Ok(a_num), Ok(b_num)) = (a.name.parse::<u32>(), b.name.parse::<u32>()) {
        return a_num.cmp(&b_num);
    }

    a.name.cmp(&b.name)
}

fn parse_json(json: String) -> Result<Vec<Print>> {
    let dict: PrintDictionary = serde_json::from_str(json.as_str())?;

    let mut print_vec = Vec::new();

    for (print_name, unit_dict) in dict.iter() {
        let mut unit_vec = Vec::new();
        for (unit_name, unit_quantity) in unit_dict.iter() {
            unit_vec.push(Unit {
                name: unit_name.to_owned(),
                quantity: unit_quantity.clone()
            })
        }
        unit_vec.sort_by(|a, b| a.name.cmp(&b.name));
        print_vec.push(Print {
            name: print_name.to_owned(),
            units: unit_vec,
            potential_unit: Unit::default()
        })
    }

    print_vec.sort_by(print_sort);
    Ok(print_vec)
}

fn get_file_string() -> Option<String> {
    FileDialog::new()
        .add_filter("units", &["units"])
        .pick_file()
        .map(|path| std::fs::read_to_string(path).ok())
        .flatten()
}

fn main() {
    let options = eframe::NativeOptions {
        min_window_size: Some(egui::Vec2::new(1100.0, 600.0)),
        ..Default::default()
    };

    let args: Vec<String> = std::env::args().collect();

    let (file_contents, error_message) = if let Some(path) = args.get(1) {
        if let Some(text) = std::fs::read_to_string(path).ok() {
            (text, None)
        } else {
            (Default::default(), Some("Error reading file".into()))
        }
    } else {
        (Default::default(), None)
    };

    let prints = if error_message.is_none() {
        parse_json(file_contents).unwrap_or(Vec::new())
    } else {
        Vec::new()
    };


    eframe::run_native(
        "Bommie",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Box::new(BommieApp {
                prints,
                error_message,
                ..Default::default()
            })
        })
    )
}

impl eframe::App for BommieApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("prints").resizable(true).show(ctx, |ui| {
            self.show_menu(ui);
            if let Some(message) = self.error_message.clone() {
                ui.horizontal(|ui| {
                    ui.colored_label(egui::Color32::RED, message);
                    if ui.button("Remove Error").clicked() {
                        self.error_message = None;
                    }
                });
            }

            self.prints_panel(ui);
        });

        if self.prints.len() != 0 {
            egui::CentralPanel::default().show(ctx, |ui| {
                self.unit_panel(ui);
            });
        }
    }
}

impl BommieApp {
    fn save_file(&self) -> Option<PathBuf> {
        let path = FileDialog::new()
            .add_filter("units", &["units"])
            .save_file()?;

        let mut dict: PrintDictionary = HashMap::new();

        for print in &self.prints {
            let mut unit_dict: HashMap<String, u32> = HashMap::new();

            for unit in &print.units {
                unit_dict.insert(unit.name.clone(), unit.quantity);
            }
            dict.insert(print.name.clone(), unit_dict);
        }

        if let Ok(prints_json) = serde_json::to_string(&dict) {
            let _ = std::fs::write(path, prints_json);
        };
        None
    }

    fn show_menu(&mut self, ui: &mut egui::Ui) {
        use egui::{menu};

        menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("New").clicked() {
                    self.prints = Vec::new()
                }

                if ui.button("Open").clicked() {
                    let contents = get_file_string().unwrap_or(String::new());
                    let result = parse_json(contents);
                    if let Ok(prints) = result {
                        self.prints = prints
                    } else {
                        self.prints = Vec::new();
                        self.error_message = Some("Error reading file".to_string());
                    }
                }

                if ui.button("Save").clicked() {
                    self.save_file();
                }
            })
        });
    }

    fn unit_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Units");
        let print = &mut self.prints[self.current_print];
        for i in 0..print.units.len() {
            let mut should_break = false;
            ui.horizontal(|ui| {
                let unit = &mut print.units[i];
                let mut temp_quantity: String = unit.quantity.to_string();
                ui.label("Unit");
                ui.text_edit_singleline(&mut print.units[i].name);
                ui.label("Quantity");
                if ui.text_edit_singleline(&mut temp_quantity).changed() {
                    if let Ok(val) = temp_quantity.parse::<u32>() {
                        print.units[i].quantity = val
                    }
                };
                if ui.button("X").clicked() {
                    print.units.remove(i);
                    should_break = true;
                }
            });
            if should_break { break; }
        }

        ui.separator();
        ui.heading("Add Unit");
        ui.horizontal(|ui| {
            let temp_unit = &mut print.potential_unit;
            let mut temp_quantity = temp_unit.quantity.to_string();
            ui.label("Unit:");
            ui.text_edit_singleline(&mut temp_unit.name);
            ui.label("Quantity: ");
            if ui.text_edit_singleline(&mut temp_quantity).changed() {
                if let Ok(val) = temp_quantity.parse::<u32>() {
                    temp_unit.quantity = val;
                }
            }
            if ui.button("Add").clicked() && temp_unit.name.len() != 0 {
                if !print.units.iter().any(|p| p.name == temp_unit.name) {
                    print.units.push(temp_unit.clone());
                    print.units.sort_by(|a, b| a.name.cmp(&b.name));
                    temp_unit.name.clear();
                    temp_unit.quantity = 0;
                }
            }
        });
    }

    fn prints_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Prints");
        let len = self.prints.len();
        for i in 0..len {

            let mut should_break = false;
            let button = if self.current_print == i {
                egui::Button::new(self.prints[i].name.clone())
                    .fill(egui::Color32::from_rgb(36, 71, 156))
            } else {
                egui::Button::new(self.prints[i].name.clone())
            };

            ui.horizontal(|ui| {
                if ui.add_sized(egui::Vec2::new(ui.available_width() - 25.0, 20.0), button).clicked() {
                    self.current_print = i;
                }
                if ui.button("X").clicked() {
                    if self.current_print == len - 1 {
                        self.current_print = len - 2;
                    }
                    self.prints.remove(i);
                    should_break = true;
                }
            });
            if should_break { break; }
        }

        ui.horizontal(|ui| {
            let add_print = |prints: &mut Vec<Print>, pp: &mut String| {
                prints.push(Print {
                    name: pp.clone(),
                    ..Default::default()
                });
                prints.sort_by(print_sort);
                pp.clear();
            };

            if ui.text_edit_singleline(&mut self.potential_print).lost_focus() &&
                ui.input().key_down(egui::Key::Enter) {
                add_print(&mut self.prints, &mut self.potential_print)
            }
            if ui.button("Add").clicked() &&
                self.potential_print.len() != 0 {
                if !self.prints.iter().any(|p| p.name == self.potential_print) {
                    add_print(&mut self.prints, &mut self.potential_print)
                }
            };
        });
    }
}

