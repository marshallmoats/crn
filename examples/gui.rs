use eframe::App;
use egui::{
    plot::{Legend, Line, Plot},
    CentralPanel, Color32, Response, SidePanel, Ui,
};

use crn::{presets, Crn, State};

const CRN_LIST: [(&str, &str); 9] = [
    (presets::MULTIPLY_CATALYZED, "Multiply catalyzed"),
    (presets::ROCK_PAPER_SCISSORS, "Rock paper scissors"),
    (presets::PREDATOR_PREY, "Predator prey"),
    (presets::POLYA, "Polya"),
    (presets::RPSLS, "RPSLS"),
    (presets::MAJORITY, "Majority"),
    (presets::MAJORITY_CATALYZED, "Majority catalyzed"),
    (presets::MULTIPLY, "Multiply"),
    (presets::EQUILIBRIUM, "Equilibrium"),
];

#[derive(Default)]
struct LinePlot {
    data: Vec<Vec<(f64, f64)>>,
}

enum CrnTypes {
    Sto,
    Det,
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
    simulation_length: f64,
    reactions: String,
    error: Option<crn::Error>,
    crn_type: CrnTypes,
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
            });

        CentralPanel::default().show(ctx, |ui| {
            ui.checkbox(&mut self.state.relative, "Relative");
            egui::ComboBox::from_label("Select a CRN")
                .selected_text("Change CRN")
                .show_ui(ui, |ui| {
                    CRN_LIST.iter().for_each(|(crn, name)| {
                        if ui
                            .selectable_value(
                                &mut self.state.reactions,
                                crn.to_string(),
                                name.to_owned(),
                            )
                            .clicked()
                        {
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
                // println!("{:?}", self.crn.names);
                self.crn.reset();
                let new_data = self
                    .crn
                    .simulate_history(self.state.simulation_length, 0.001);
                match new_data {
                    Ok(data) => {
                        self.lp.data = transpose(data);
                        self.state.error = None;
                    }
                    Err(s) => self.state.error = Some(s),
                }
                println!("{:?}", self.crn.state());
            }

            if ui.button("toggle type").clicked() {
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

            let mut input = self.state.simulation_length.to_string();
            ui.text_edit_singleline(&mut input);
            self.state.simulation_length = input.parse().unwrap_or(self.state.simulation_length);

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
                reactions: "A = 50; B = 50; 2A + B -> 3A; A + 2B -> 3B;".to_owned(),
                error: None,
                crn_type: CrnTypes::Sto,
            },
            crn: Box::new(
                crn::StoCrn::parse("A = 50; B = 50; 2A + B -> 3A; A + 2B -> 3B;").unwrap(),
            ),
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
