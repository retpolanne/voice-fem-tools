use std::time::Duration;
use spectrum_analyzer::{samples_fft_to_spectrum, FrequencyLimit};
use spectrum_analyzer::windows::hann_window;
use spectrum_analyzer::scaling::divide_by_N_sqrt;
use cpal::traits::HostTrait;
use cpal::traits::{DeviceTrait, StreamTrait};

fn main() {
    let host = cpal::default_host();
    let devices = host.devices()
                      .expect("can't open devices")
                      .map(|dev| println!("device {}", dev.name().expect("")));
    let device = host.default_input_device().expect("couldn't get device");
    let config = device.default_input_config().expect("couldn't get config");
    let input_config = device.default_input_config()
                             .expect("failed to get default input config");
    let err_fn = |err| eprintln!("error: {}", err);
    println!(
        "using device {} with input config {:?}", device.name().expect(""),
        input_config
    );
    std::thread::sleep(Duration::from_secs(10));
    let input_stream = device.build_input_stream(
        &input_config.config(),
        // TODO move this to a function somewhere else
        move |data: &[f32], _: &cpal::InputCallbackInfo | {
            let sample_start = data.len() - 2048;
            let hann_window = hann_window(
                &data[sample_start..sample_start + 2048]
            );
            let spectrum_hann_window = samples_fft_to_spectrum(
                &hann_window,
                config.sample_rate().0,
                FrequencyLimit::Range(165.0, 255.0),
                Some(&divide_by_N_sqrt)
            ).unwrap();

            for (fr, fr_val) in spectrum_hann_window.data().iter() {
                println!("{}Hz -> {}", fr, fr_val);
            }
        },
        err_fn,
        None
    ).expect("couldn't create input stream");
    let _ = input_stream.play().expect("couldn't play stream");
    std::thread::sleep(Duration::from_secs(10));
}
