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
    egui::{
        Align, CentralPanel, Color32, DragValue, Grid, Layout, ProgressBar, Sense, Slider, Ui,
        Vec2, ViewportBuilder, ViewportCommand, Widget, vec2,
    },
};
use ftdi_embedded_hal::{FtHal, eh1::digital::OutputPin};

#[derive(Default)]
struct App {
    interface: Control,
    soft_start: SoftStart,
    start: Option<Instant>,

    control: Arc<ArcSwap<Control>>,
    output: Arc<AtomicBool>,

    prev_size: Vec2,
}

#[derive(Clone)]
struct Control {
    freq: f32,
    duty: f32,
    active: bool,
}

struct SoftStart {
    enabled: bool,
    duration: f32,
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
            viewport: ViewportBuilder::default().with_inner_size([1.0, 1.0]),
            ..Default::default()
        },
        Box::new(|_cc| {
            Ok(Box::new(App {
                interface: Control::default(),
                soft_start: SoftStart::default(),
                start: None,

                control,
                output,

                prev_size: Default::default(),
            }))
        }),
    )?;
    Ok(())
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        ui.visuals_mut().collapsing_header_frame = true;

        CentralPanel::default().show(ui, |ui| {
            Grid::new("settings").striped(true).show(ui, |ui| {
                ui.label("Frequency");
                Slider::new(&mut self.interface.freq, 0.1..=30.0)
                    .suffix(" Hz")
                    .ui(ui);
                ui.end_row();

                ui.label("Duty");
                Slider::new(&mut self.interface.duty, 0.01..=1.0).ui(ui);
                ui.end_row();
            });

            ui.add_space(8.0);
            self.soft_start.enabled = collapsing_toggle(
                "Soft Start",
                self.soft_start.enabled,
                |ui| {
                    Grid::new("soft start").striped(true).show(ui, |ui| {
                        ui.label("Startup Time");
                        DragValue::new(&mut self.soft_start.duration)
                            .suffix(" s")
                            .ui(ui);
                        ui.end_row();
                    });

                    ui.add_space(8.0);
                    if let Some(start) = self.start
                        && self.soft_start.enabled
                    {
                        let t = start.elapsed().as_secs_f32() / self.soft_start.duration;
                        if t < 1.0 {
                            ProgressBar::new(t.clamp(0.0, 1.0)).ui(ui);
                        }
                    }
                },
                ui,
            );

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

                let clicked = ui
                    .button(["Start", "Stop"][self.interface.active as usize])
                    .clicked();
                self.interface.active ^= clicked;

                if clicked {
                    match self.interface.active {
                        true => self.start = Some(Instant::now()),
                        false => self.start = None,
                    }
                }
            });
        });

        let duty = if let Some(start) = self.start
            && self.soft_start.enabled
        {
            let t = start.elapsed().as_secs_f32() / self.soft_start.duration;
            self.interface.duty * t.clamp(0.0, 1.0)
        } else {
            self.interface.duty
        };

        self.control.swap(Arc::new(Control {
            freq: self.interface.freq,
            duty,
            active: self.interface.active,
        }));

        let size = ui.ctx().globally_used_rect().size();
        if size != self.prev_size {
            self.prev_size = size;
            ui.send_viewport_cmd(ViewportCommand::InnerSize(size));
        }
    }
}

fn collapsing_toggle(
    title: &str,
    mut toggle: bool,
    content: impl FnOnce(&mut Ui),
    ui: &mut Ui,
) -> bool {
    ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
        ui.checkbox(&mut toggle, "");
        ui.collapsing(title, |ui| content(ui));
    });
    toggle
}

impl Default for Control {
    fn default() -> Self {
        Self {
            freq: 5.0,
            duty: 0.5,
            active: false,
        }
    }
}

impl Default for SoftStart {
    fn default() -> Self {
        Self {
            enabled: false,
            duration: 60.0,
        }
    }
}
