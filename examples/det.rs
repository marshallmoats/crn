use eframe::App;
use egui::{
    plot::{Legend, Line, Plot},
    CentralPanel, Color32, Response, SidePanel, Ui,
};

use crn::DetCrn;

#[derive(Default)]
struct LinePlot {
    data: Vec<Vec<(f64, f64)>>,
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

#[derive(Default)]
pub struct CrnApp {
    lp: LinePlot,
    crn: DetCrn,
    state: CrnAppState,
}

#[derive(Default)]
struct CrnAppState {
    relative: bool,
    simulation_length: usize,
    reactions: String,
    error: Option<crn::Error>,
    dt: f64,
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
                    match crn::DetCrn::parse(&self.state.reactions) {
                        Ok(crn) => self.crn = crn,
                        Err(e) => {
                            println!("Error: {:?}", e);
                        }
                    }
                }
            });

        CentralPanel::default().show(ctx, |ui| {
            ui.checkbox(&mut self.state.relative, "Relative");
            if ui.button("Resimulate").clicked() {
                // println!("{:?}", self.crn.names);
                self.crn.reset();
                let new_data = self
                    .crn
                    .simulate_data(self.state.simulation_length, self.state.dt);
                self.lp.data = new_data;
                self.state.error = None;
            }
            self.state
                .error
                .as_ref()
                .map(|e| ui.label(format!("Error: {:?}", e)));
            // ui.label(format!("Error: {:?}", self.state.error));

            let mut input = self.state.simulation_length.to_string();
            ui.text_edit_singleline(&mut input);
            self.state.simulation_length = input.parse().unwrap_or(self.state.simulation_length);

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
                simulation_length: 10000,
                reactions: String::new(),
                dt: 0.01,
                ..Default::default()
            },
            crn: crn::DetCrn::parse("A = 50; B = 50; 2A + B -> 3A; A + 2B -> 3B;").unwrap(),
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
