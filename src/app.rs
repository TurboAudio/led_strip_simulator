use std::{
    io::Read,
    net::{TcpListener, TcpStream},
    sync::mpsc,
    thread,
};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:
    label: String,

    // this how you opt-out of serialization of a member
    #[serde(skip)]
    value: f32,

    #[serde(skip)]
    rx: mpsc::Receiver<Vec<egui::Color32>>,
}

impl Default for TemplateApp {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel::<Vec<egui::Color32>>();
        thread::spawn(move || {
            let listener = TcpListener::bind("127.0.0.1:42069").unwrap();
            #[allow(clippy::all)]
            for stream in listener.incoming() {
                if let Ok(mut stream) = stream {
                    thread::spawn({
                        let tx = tx.clone();
                        move || {
                            let tx = tx.clone();
                            let mut buffer = [0_u8; 900];
                            loop {
                                match stream.read_exact(&mut buffer) {
                                    Err(e) => {
                                        eprintln!("Err: {}", e);
                                        break;
                                    }
                                    Ok(_) => {
                                        let colors = buffer
                                            .chunks_exact(3)
                                            .map(|data| {
                                                egui::Color32::from_rgb(data[0], data[1], data[2])
                                            })
                                            .collect::<Vec<_>>();
                                        if let Err(e) = tx.send(colors) {
                                            eprintln!("Bruv: {:?}", e);
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    });
                }
            }
        });
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            value: 2.7,
            rx,
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        cc.egui_ctx.set_visuals(egui::Visuals::dark());

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Ok(mut colors) = self.rx.recv() {
                if let Ok(newer_colors) = self.rx.try_recv() {
                    colors = newer_colors;
                }
                for (index, color) in colors.iter().enumerate() {
                    let circle = egui::Shape::Circle(eframe::epaint::CircleShape {
                        center: egui::Pos2 {
                            x: 10.0 + 6.0 * index as f32,
                            y: 10.0,
                        },
                        radius: 3.0,
                        fill: *color,
                        stroke: egui::Stroke::NONE,
                    });
                    ui.painter().add(circle);
                }
            }

            // egui::warn_if_debug_build(ui);
        });
    }
}
