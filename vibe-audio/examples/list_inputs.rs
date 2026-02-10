use cpal::traits::{DeviceTrait, HostTrait};

fn main() {
    let target = std::env::args().nth(1);
    if let Some(ref t) = target {
        println!("Setting PIPEWIRE_NODE={}", t);
        std::env::set_var("PIPEWIRE_NODE", t);
    }

    let host = cpal::default_host();
    let device = host.default_input_device().expect("default input device");
    println!("Default input device: {:?}", device.name());

    let config = cpal::StreamConfig {
        channels: 2,
        sample_rate: 44100,
        buffer_size: cpal::BufferSize::Default,
    };

    let stream = device
        .build_input_stream(
            &config,
            |data: &[f32], _: &cpal::InputCallbackInfo| {
                let max = data.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
                if max > 0.001 {
                    println!("Audio detected! max={:.4}", max);
                }
            },
            |err| eprintln!("Error: {}", err),
            None,
        )
        .expect("build input stream");

    use cpal::traits::StreamTrait;
    stream.play().unwrap();

    println!("Listening for 5 seconds...");
    std::thread::sleep(std::time::Duration::from_secs(5));
}
