use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

use anyhow::Result;
use arc_swap::ArcSwap;
use clone_macro::clone;
use eframe::{
    NativeOptions,
    egui::{CentralPanel, Color32, Grid, Sense, Slider, Ui, ViewportBuilder, Widget, vec2},
};
use ftdi_embedded_hal::{FtHal, eh1::digital::OutputPin};

#[derive(Default)]
struct App {
    interface: Control,

    control: Arc<ArcSwap<Control>>,
    output: Arc<AtomicBool>,
}

#[derive(Clone)]
struct Control {
    freq: f32,
    duty: f32,
    active: bool,
}

fn main() -> Result<()> {
    let control = Arc::new(ArcSwap::new(Arc::new(Control::default())));
    let output = Arc::new(AtomicBool::new(false));

    thread::spawn(clone!([control, output], move || {
        let device = ftdi::find_by_vid_pid(0x0403, 0x6014)
            .interface(ftdi::Interface::A)
            .open()
            .unwrap();

        let hal = FtHal::init_default(device).unwrap();
        let mut gpio = hal.ad0().unwrap();

        let mut next_cycle = Instant::now();
        loop {
            let control = control.load();
            let period = Duration::from_secs_f32(control.freq.recip());
            let on_time = Duration::from_secs_f32(control.freq.recip() * control.duty);

            let on_deadline = next_cycle + on_time;
            next_cycle = next_cycle + period;

            if control.active {
                gpio.set_high().unwrap();
                output.store(true, Ordering::Relaxed);
            }

            thread::sleep(on_deadline.saturating_duration_since(Instant::now()));
            gpio.set_low().unwrap();
            output.store(false, Ordering::Relaxed);
            thread::sleep(next_cycle.saturating_duration_since(Instant::now()));
        }
    }));

    eframe::run_native(
        "Heater Control",
        NativeOptions {
            viewport: ViewportBuilder::default().with_inner_size([250.0, 120.0]),
            ..Default::default()
        },
        Box::new(|_cc| {
            Ok(Box::new(App {
                interface: Control::default(),
                control,
                output,
            }))
        }),
    )?;
    Ok(())
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ui, |ui| {
            ui.heading("Heater Control");
            ui.add_space(8.0);

            Grid::new("settings").striped(true).show(ui, |ui| {
                ui.label("Frequency");
                Slider::new(&mut self.interface.freq, 0.1..=60.0)
                    .suffix(" Hz")
                    .ui(ui);
                ui.end_row();

                ui.label("Duty");
                Slider::new(&mut self.interface.duty, 0.01..=1.0).ui(ui);
                ui.end_row();
            });

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                let (rect, _response) = ui.allocate_exact_size(vec2(16.0, 16.0), Sense::hover());
                let painter = ui.painter();
                ui.request_repaint();
                painter.circle_filled(
                    rect.center(),
                    8.0,
                    if self.output.load(Ordering::Relaxed) {
                        Color32::RED
                    } else {
                        Color32::GRAY
                    },
                );

                self.interface.active ^= ui
                    .button(if self.interface.active {
                        "Stop"
                    } else {
                        "Start"
                    })
                    .clicked();
            });
        });

        self.control.swap(Arc::new(self.interface.clone()));
    }
}

impl Default for Control {
    fn default() -> Self {
        Self {
            freq: 10.0,
            duty: 0.5,
            active: false,
        }
    }
}
