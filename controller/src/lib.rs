// Determined experimentally, see heater-control.nb
// Temperatures must be in °C and time in seconds.
const C1: f32 = 1501.64;
const C2: f32 = 1.09785;

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
