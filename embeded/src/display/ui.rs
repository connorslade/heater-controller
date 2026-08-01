use crate::{
    config::system::HEATER_POWER,
    control::{Controller, State, simulate},
    display::{Bounds, Ui, Vec2, font::CHAR_SIZE},
};

const POWER_METER: Bounds = Bounds::new((0, 0), (6, 63));
const TEMPATURE_PANEL: Bounds = Bounds::new((8, 0), (50, 35));
const HEATER_PANEL: Bounds = Bounds::new((8, 37), (50, 63));
const TEMPATURE_HISTORY: Bounds = Bounds::new((52, 21), (127, 63));
const STATUS_BAR: Bounds = Bounds::new((52, 0), (127, 19));

const PADDING: u32 = 2;
const LINE_HEIGHT: u32 = CHAR_SIZE.y + PADDING;

pub fn ui(ui: &mut Ui, controller: &Controller, power: f32, temp: f32) {
    ui.frame(STATUS_BAR, |ui| {
        ui.space(Vec2::splat(PADDING));
        ui.draw_string_inverted((1, 1), "STATUS");
        ui.space_y(LINE_HEIGHT + 2);

        let elapsed = controller.start.elapsed().as_secs();
        if elapsed < 60 {
            ui.draw_value((0, 0), "\u{E000} ", "S", elapsed as u32);
        } else {
            ui.draw_value((0, 0), "\u{E000} ", "M", (elapsed / 60) as u32);
        }
    });

    ui.frame(POWER_METER, |ui| {
        let height = (power * POWER_METER.height() as f32) as u32;
        ui.draw_progress_vertical(height);
    });

    ui.frame(TEMPATURE_PANEL, |ui| {
        ui.space(Vec2::splat(PADDING));

        ui.draw_string_inverted((1, 1), "THERMO");
        ui.space_y(LINE_HEIGHT + 2);

        ui.draw_value((0, 0), "TG ", "°C", controller.goal as u32);
        ui.space_y(LINE_HEIGHT);
        ui.draw_value((0, 0), "T\u{E002} ", "°C", temp as u32);
        ui.space_y(LINE_HEIGHT);
        ui.draw_value((0, 0), "T\u{E003} ", "°C", temp as u32);
    });

    ui.frame(HEATER_PANEL, |ui| {
        ui.space(Vec2::splat(PADDING));

        let mode = match controller.state {
            State::Heating => "HEATING",
            State::Holding => "HOLDING",
        };
        ui.draw_string_inverted((1, 1), mode);
        ui.space_y(LINE_HEIGHT + 2);

        ui.draw_value((0, 0), "", "W", (power * HEATER_POWER) as u32);
        ui.space_y(LINE_HEIGHT);

        ui.draw_value((0, 0), "", "%", (power * 100.0) as u32);
    });

    ui.frame(TEMPATURE_HISTORY, |ui| {
        let mut data = [0.0; 10];
        let mut t = 28.2303;
        for item in data.iter_mut() {
            *item = t;
            t = simulate(t, 28.2303, 10.0, 600.0, 60.0);
        }

        ui.chart::<10>((26.0, 40.0), data);
        ui.draw_string_inverted((36, 33), "HISTORY");
    });
}
