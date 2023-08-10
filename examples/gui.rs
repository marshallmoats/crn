use std::fmt::Display;

use eframe::App;
use egui::{
    plot::{Legend, Line, Plot},
    CentralPanel, Color32, Response, SidePanel, Ui,
};

use crn::{presets, Crn, state::State};

const CRN_LIST: [(&str, &str, &str); 9] = [
    (presets::RPSLS, "Rock paper scissors lizard spock", "Same as the rock paper scissors CRN, but with two more players."),
    (presets::MULTIPLY_CATALYZED, "Multiply catalyzed", "Calculates the product with some random perturbations of catalysts."),
    (presets::ROCK_PAPER_SCISSORS, "Rock paper scissors", "The molecules play rock paper scissors. The winner transforms the loser into a copy of itself."),
    (presets::PREDATOR_PREY, "Predator prey", "A is the prey and B is the predator."),
    (presets::POLYA, "Polya", "Polya's urn. Draw a marble"),
    (presets::MAJORITY, "Majority", "Determines which of A and B is more abundant."),
    (presets::MAJORITY_CATALYZED, "Majority catalyzed", "The majority CRN, but with catalysts that transform into one another."),
    (presets::MULTIPLY, "Multiply", "Approximately calculates the product of A and B. A deterministic simulation will compute it exactly."),
    (presets::EQUILIBRIUM, "Equilibrium", "A basic CRN with two reactions that reach equilibrium."),
];

#[derive(Default)]
struct LinePlot {
    data: Vec<Vec<(f64, f64)>>,
}

enum CrnTypes {
    Sto,
    Det,
}

impl Display for CrnTypes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CrnTypes::Sto => write!(f, "Stochastic"),
            CrnTypes::Det => write!(f, "Deterministic"),
        }
    }
}

impl LinePlot {
    const COLORS: [Color32; 16] = [
        Color32::RED,
        Color32::GREEN,
        Color32::BLUE,
        Color32::GOLD,
        Color32::LIGHT_BLUE,
        Color32::LIGHT_RED,
        Color32::LIGHT_GREEN,
        Color32::LIGHT_YELLOW,
        Color32::DARK_BLUE,
        Color32::DARK_RED,
        Color32::DARK_GREEN,
        Color32::KHAKI,
        Color32::BROWN,
        Color32::YELLOW,
        Color32::WHITE,
        Color32::GRAY,
    ];

    fn plot(&self, idx: usize) -> Line {
        let points: Vec<[f64; 2]> = self.data[idx].iter().map(|(a, b)| [*a, *b]).collect();
        Line::new(points)
            .color(Self::COLORS[idx % Self::COLORS.len()])
            .name(format!("{}", idx))
    }

    fn ui(&mut self, ui: &mut Ui) -> Response {
        let plot = Plot::new("CRN data").legend(Legend::default());
        plot.show(ui, |plot_ui| {
            for i in 0..self.data.len() {
                plot_ui.line(self.plot(i));
            }
        })
        .response
    }
}

struct CrnApp {
    lp: LinePlot,
    crn: Box<dyn Crn>,
    state: CrnAppState,
}

struct CrnAppState {
    relative: bool,
    dt: f64,
    simulation_length: f64,
    reactions: String,
    error: Option<crn::Error>,
    crn_type: CrnTypes,
    desc: &'static str,
}

impl App for CrnApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        SidePanel::left("left_panel")
            .resizable(true)
            .default_width(150.0)
            .width_range(250.0..=300.0)
            .show(ctx, |ui| {
                ui.label("Reactions");

                ui.code_editor(&mut self.state.reactions);

                if ui.button("Parse").clicked() {
                    match self.state.crn_type {
                        CrnTypes::Sto => match crn::StoCrn::parse(&self.state.reactions) {
                            Ok(crn) => self.crn = Box::new(crn),
                            Err(e) => {
                                println!("Error: {:?}", e);
                            }
                        },
                        CrnTypes::Det => match crn::DetCrn::parse(&self.state.reactions) {
                            Ok(crn) => self.crn = Box::new(crn),
                            Err(e) => {
                                println!("Error: {:?}", e);
                            }
                        },
                    }
                }

                ui.label(self.state.desc);
            });

        CentralPanel::default().show(ctx, |ui| {
            ui.checkbox(&mut self.state.relative, "Relative Proportions");
            egui::ComboBox::from_label("Select a CRN")
                .selected_text("Change CRN")
                .show_ui(ui, |ui| {
                    CRN_LIST.iter().for_each(|(crn, name, desc)| {
                        if ui
                            .selectable_value(
                                &mut self.state.reactions,
                                crn.to_string(),
                                name.to_owned(),
                            )
                            .clicked()
                        {
                            self.state.desc = desc;
                            self.crn.reset();

                            match self.state.crn_type {
                                CrnTypes::Sto => match crn::StoCrn::parse(&self.state.reactions) {
                                    Ok(crn) => self.crn = Box::new(crn),
                                    Err(e) => {
                                        println!("Error: {:?}", e);
                                    }
                                },
                                CrnTypes::Det => match crn::DetCrn::parse(&self.state.reactions) {
                                    Ok(crn) => self.crn = Box::new(crn),
                                    Err(e) => {
                                        println!("Error: {:?}", e);
                                    }
                                },
                            }
                            self.state.reactions = self.crn.to_string();
                        }
                    });
                });
            if ui.button("Resimulate").clicked() {
                self.crn.reset();
                let new_data = self
                    .crn
                    .simulate_history(self.state.simulation_length, self.state.dt);
                match new_data {
                    Ok(data) => {
                        self.lp.data = match self.state.relative {
                            true => normalize(transpose(data)),
                            false => transpose(data),
                        };
                        self.state.error = None;
                    }
                    Err(s) => self.state.error = Some(s),
                }
                println!("{:?}", self.crn.state());
            }

            if ui.button(self.state.crn_type.to_string()).clicked() {
                match self.state.crn_type {
                    CrnTypes::Sto => {
                        self.state.crn_type = CrnTypes::Det;
                        self.crn = Box::new(crn::DetCrn::parse(&self.state.reactions).unwrap());
                    }
                    CrnTypes::Det => {
                        self.state.crn_type = CrnTypes::Sto;
                        self.crn = Box::new(crn::StoCrn::parse(&self.state.reactions).unwrap());
                    }
                }
            }

            self.state
                .error
                .as_ref()
                .map(|e| ui.label(format!("Error: {:?}", e)));
            // ui.label(format!("Error: {:?}", self.state.error));

            ui.label("Simulation length");
            let mut input = self.state.simulation_length.to_string();
            ui.text_edit_singleline(&mut input);
            self.state.simulation_length = input.parse().unwrap_or(self.state.simulation_length);

            ui.label("dt (only affects deterministic runs)");
            let mut input = self.state.dt.to_string();
            ui.text_edit_singleline(&mut input);
            self.state.dt = input.parse().unwrap_or(self.state.dt);

            self.lp.ui(ui);
        });
    }
}

impl CrnApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            lp: LinePlot {
                ..Default::default()
            },
            state: CrnAppState {
                relative: false,
                simulation_length: 1.0,
                reactions: presets::RPSLS.to_string(),
                error: None,
                crn_type: CrnTypes::Sto,
                dt: 0.001,
                desc: CRN_LIST[0].2,
            },
            crn: Box::new(crn::StoCrn::parse(presets::RPSLS).unwrap()),
        }
    }
}

fn main() {
    let native_options = eframe::NativeOptions {
        initial_window_size: Some(egui::Vec2::new(800.0, 450.0)),
        ..Default::default()
    };

    eframe::run_native(
        "CRNSim",
        native_options,
        Box::new(|cc| Box::new(CrnApp::new(cc))),
    )
    .unwrap();
}

fn transpose(data: Vec<State<f64>>) -> Vec<Vec<(f64, f64)>> {
    let mut result = Vec::new();
    for _ in 0..data[0].species.len() {
        result.push(Vec::new());
    }
    for state in data {
        for (i, species) in state.species.iter().enumerate() {
            result[i].push((state.time, *species));
        }
    }
    result
}

fn normalize(data: Vec<Vec<(f64, f64)>>) -> Vec<Vec<(f64, f64)>> {
    let mut result = Vec::new();
    for species in data {
        let mut new_species = Vec::new();
        let mut sum = 0.0;
        for (_, val) in &species {
            sum += val;
        }
        for (time, val) in species {
            new_species.push((time, val / sum));
        }
        result.push(new_species);
    }
    result
}
