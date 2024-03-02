use std::cell::RefCell;
use std::cmp::max;

use audio_visualizer::dynamic::live_input::{list_input_devs, AudioDevAndCfg};
use audio_visualizer::dynamic::window_top_btm::{
    open_window_connect_audio, TransformFn
};
use clap::Parser;
use cpal::Device;
use cpal::traits::DeviceTrait;
use spectrum_analyzer::{
    samples_fft_to_spectrum, FrequencyLimit, FrequencyValue
};
use spectrum_analyzer::windows::hann_window;
use spectrum_analyzer::scaling::divide_by_N;

#[derive(Parser)]
#[clap(author="Anne Isabelle Macedo", version, about)]
/// A toolbox for voice feminization
struct Arguments {
    #[clap(long)]
    /// the input device to use
    device: Option<i32>,
}

fn create_spectrum_fn() -> Box<(dyn Fn(&[f32], f32) -> Vec<(f64, f64)> + 'static)> {
    let visualize_spectrum: RefCell<Vec<(f64, f64)>> = RefCell::new(vec![(0.0, 0.0); 1024]);
    Box::new(move | data: &[f32], sampling_rate | {
        let sample_start = data.len() - 2048;
        let hann_window = hann_window(
            &data[sample_start..sample_start + 2048]
        );
        samples_fft_to_spectrum(
            &hann_window,
            sampling_rate as u32,
            FrequencyLimit::All,
            Some(&divide_by_N)
        ).unwrap()
         .data()
         .iter()
         .zip(visualize_spectrum.borrow_mut().iter_mut())
         .for_each(|((fr_new, fr_val_new), (fr_old, fr_val_old))| {
             // actually only required in very first iteration
             *fr_old = fr_new.val() as f64;
             let old_val = *fr_val_old * 0.84;
             let max = max(
                 *fr_val_new * 5000.0_f32.into(),
                 FrequencyValue::from(old_val as f32),
             );
             *fr_val_old = max.val() as f64;
         });
        visualize_spectrum.borrow().clone()
    })
}

fn main() {
    let args = Arguments::parse();
    let chosen_dev_index = args.device.unwrap_or_else(|| -1);
    let devs = list_input_devs();
    if chosen_dev_index < 0 {
        devs.iter().enumerate().for_each(
            |(i, dev)| println!("device: {i}, {}", dev.0)
        );
        eprintln!("choose one of those input devices!");
        std::process::exit(1);
    }
    let dev: Device = devs.into_iter()
                  .enumerate()
                  .filter(|i| i.0 as i32 == chosen_dev_index)
                  .map(|(_, dev)| dev.1)
                  .next()
                  .expect("");
    println!("chosen device: {}", dev.name().expect(""));
    open_window_connect_audio(
        "voice fem tools",
        None,
        None,
        Some(0.0..5000.0),
        Some(0.0..10.0),
        "frequency",
        "amplitude",
        AudioDevAndCfg::new(Some(dev), None),
        TransformFn::Complex(&create_spectrum_fn()),
    );
}
