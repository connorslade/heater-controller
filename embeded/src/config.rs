pub mod thermister {
    pub const R0: f32 = 10_000.0;
    pub const T0: f32 = 298.15;
    pub const BETA: f32 = 3470.0;

    pub const R2: f32 = 10_000.0;
}

pub mod system {
    use embassy_time::Duration;

    // Determined experimentally, see heater-control.nb
    // Temperatures must be in °C and time in seconds.
    pub const C1: f32 = 1501.64;
    pub const C2: f32 = 1.09785;

    pub const DT: f32 = 10.0; // Numerical integration dt (s)
    pub const TIME_OFFSET: f32 = 10.0; // Steady state switch offset (s)
    pub const HEATER_POWER: f32 = 600.0; // Heater power rating (W)
    pub const POWER_MAX: f32 = 0.4; // Max fraction of HEATER_POWER to use while heating
    pub const SOFT_START: f32 = 60.0; // Time to ramp up to max power (s)

    pub const SAMPLE_PEROID: Duration = Duration::from_secs(1); // How often to sample the temp
    pub const TEMP_SAMPLES: usize = 10; // How many samples to average to get the temperature
}
