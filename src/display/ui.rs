use micromath::F32Ext;

use crate::{
    config::system::{C2, HEATER_POWER},
    control::{Controller, simulate},
    display::{Bounds, Ui, Vec2, font::CHAR_SIZE},
};

const POWER_METER: Bounds = Bounds::new((0, 0), (6, 63));
const TEMPATURE_PANEL: Bounds = Bounds::new((8, 0), (50, 35));
const HEATER_PANEL: Bounds = Bounds::new((8, 37), (50, 63));
const TEMPATURE_HISTORY: Bounds = Bounds::new((52, 30), (127, 63));
const STATUS_BAR: Bounds = Bounds::new((52, 0), (127, 28));

const PADDING: u32 = 2;
const LINE_HEIGHT: u32 = CHAR_SIZE.y + PADDING;

pub fn ui(ui: &mut Ui, cpu_usage: f32, controller: &Controller, power: f32, temp: f32) {
    ui.frame(STATUS_BAR, |ui| {
        ui.space(Vec2::splat(PADDING));
        ui.draw_string_inverted((1, 1), "STATUS");
        ui.draw_string_inverted((55, 1), "S");
        ui.draw_string_inverted((64, 1), "\u{E006}");
        ui.space_y(LINE_HEIGHT + 2);

        let elapsed = controller.start.elapsed().as_secs();
        if elapsed < 60 {
            ui.draw_value((0, 0), "\u{E000}\u{E008}", "S", elapsed as u32);
        } else {
            ui.draw_value((0, 0), "\u{E000}\u{E008}", "M", (elapsed / 60) as u32);
        }
        ui.space_y(LINE_HEIGHT + 2);

        let cpu_percent = (cpu_usage * 100.0).round() as u32;
        ui.draw_value((0, 0), "\u{E007}\u{E008}", "%", cpu_percent);
    });

    ui.frame(POWER_METER, |ui| {
        let height = (power * POWER_METER.height() as f32) as u32;
        ui.draw_progress_vertical(height);
    });

    ui.frame(TEMPATURE_PANEL, |ui| {
        ui.space(Vec2::splat(PADDING));

        ui.draw_string_inverted((1, 1), "THERMO");
        ui.space_y(LINE_HEIGHT + 2);

        ui.draw_value((0, 0), "T\u{E004}\u{E005}", "°C", controller.goal as u32);
        ui.space_y(LINE_HEIGHT);
        ui.draw_value((0, 0), "T\u{E003}\u{E005}", "°C", temp as u32);
        ui.space_y(LINE_HEIGHT);
        ui.draw_value((0, 0), "T\u{E002}\u{E005}", "°C", temp as u32);
    });

    ui.frame(HEATER_PANEL, |ui| {
        ui.space(Vec2::splat(PADDING));

        ui.draw_string_inverted((1, 1), controller.state.name());
        ui.space_y(LINE_HEIGHT + 2);

        ui.draw_value((0, 0), "P\u{E005}", "W", (power * HEATER_POWER) as u32);
        ui.space_y(LINE_HEIGHT);

        ui.draw_value((0, 0), "U\u{E005}", "%", (power * 100.0) as u32);
    });

    ui.frame(TEMPATURE_HISTORY, |ui| {
        let amb = 28.2303;
        let goal = 10.0 / C2 + amb;

        let mut data = [0.0; 10];
        let mut t = amb;
        for item in data.iter_mut() {
            *item = t;
            t = simulate(t, amb, 10.0, 600.0, 60.0);
        }

        let range = (26.0, 40.0);
        let y_amb = (1.0 - (amb - range.0) / (range.1 - range.0)) * ui.bounds.height() as f32;
        let y_goal = (1.0 - (goal - range.0) / (range.1 - range.0)) * ui.bounds.height() as f32;

        ui.draw_line_horizontal((0, y_amb as u32), ui.bounds.width());
        ui.draw_line_horizontal((0, y_goal as u32), ui.bounds.width());
        ui.draw_chart::<10>(range, data);
        ui.draw_string_inverted((36, 24), "HISTORY");
    });
}
