use std::fs;

use controller::{simulate, steady_state};

fn main() {
    let t0 = 22.9615; // Initial temperature (°C)
    let amb = 25.5; // Ambient temperature (°C)
    let delta = 10.0; // Numerical integration dt (s)

    let goal = 54.44; // Goal temperature (°C)
    let time_offset = 60.0; // Steady state switch offset (s)
    let power_max = 600.0 * 1.0; // Power for initial heating (W)
    let soft_start = 60.0; // Time to ramp up to max power (s)
    let power_steady = steady_state(goal, amb);

    let mut temp = t0;
    let mut goal_power = power_max;
    let mut out = String::new();

    // Simulate 1 hour, data point for each minute
    for i in 0..(60 * 1) {
        let t = i as f32 * 60.0;
        let power = goal_power * (t / soft_start).min(1.0);

        temp = simulate(temp, amb, power, 60.0, delta); // simulate next minute
        let next = simulate(temp, amb, power, 60.0 + time_offset, delta);
        if next >= goal {
            println!("Switching to steady state at t={:.1}m", t / 60.0);
            goal_power = power_steady;
        }

        out.push_str(&format!("{t},{temp}\n"));
    }

    fs::write("../data/sim-1.csv", out).unwrap();
}
