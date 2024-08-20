pub const DEFAULT_PADDING: f32 = 10.;

pub fn percentage(value: f32, percent: u8) -> f32 {
    percent as f32 * value / 100.
}
