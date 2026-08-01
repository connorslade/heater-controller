use embassy_time::Instant;
use micromath::F32Ext;

use crate::config::system::{C1, C2, DT, HEATER_POWER, POWER_MAX, SOFT_START, TIME_OFFSET};

pub struct Controller {
    pub start: Instant,
    pub state: State,

    pub goal: f32,
}

pub enum State {
    Heating,
    // Cooling,
    Holding,
    // Idle,
}

impl Controller {
    pub fn new(goal: f32) -> Self {
        Self {
            start: Instant::now(),
            state: State::Heating,
            goal,
        }
    }

    // Returns the fraction of MAX_POWER to output
    pub fn update(&mut self, temp: f32, amb: f32) -> f32 {
        match self.state {
            State::Heating => {
                let t = self.start.elapsed().as_millis() as f32 / 1000.0;
                let power = POWER_MAX * (t / SOFT_START).min(1.0);

                // Simulate TIME_OFFSET seconds into the future. If the
                // temperature is above the goal, switch to hold mode.
                let next = simulate(temp, amb, power * HEATER_POWER, TIME_OFFSET, DT);
                (next >= self.goal).then(|| self.state = State::Holding);

                power
            }
            // State::Cooling => {
            //     let next = simulate(temp, amb, 0.0, TIME_OFFSET, DT);
            //     (next <= self.goal).then(|| self.state = State::Holding);
            //     0.0
            // }
            State::Holding => {
                let power_steady = steady_state(self.goal, amb);
                (power_steady / HEATER_POWER).clamp(0.0, 1.0)
            } // State::Idle => 0.0,
        }
    }
}

impl State {
    pub fn name(&self) -> &str {
        match self {
            State::Heating => "HEATING",
            // State::Cooling => "COOLING",
            State::Holding => "HOLDING",
            // State::Idle => "IDLE",1
        }
    }
}

// (P - C2(T - Tamb)) / C1
pub fn simulate(mut temp: f32, amb: f32, power: f32, time: f32, delta: f32) -> f32 {
    let steps = (time / delta).ceil() as u32;
    for _ in 0..steps {
        let dt = (power - C2 * (temp - amb)) / C1;
        temp += dt * delta;
    }

    temp
}

// P = C2(T - T_amb)
pub fn steady_state(goal: f32, amb: f32) -> f32 {
    C2 * (goal - amb)
}
