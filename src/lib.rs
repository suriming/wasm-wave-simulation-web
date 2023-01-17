use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn generate_wave(time: f32) -> Vec<f32> {
    let mut wave_data = vec![];
    for x in 0..1200 {
        let y = (x as f32 / 1200.0 * std::f32::consts::PI * 2.0 + time).sin();
        wave_data.push((y+1.0)* 24.0);
    }

    println!("time:{:?}", time);
    println!("wave_data:{:?}", wave_data);
    wave_data
}