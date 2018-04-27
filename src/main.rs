extern crate cpal;

fn main() {
    let device = cpal::default_output_device().expect("failed to get output device");
    let format = device.default_output_format().expect("failed to get output format");
    let event_loop = cpal::EventLoop::new();
    let stream_id = event_loop.build_output_stream(&device, &format).unwrap();
    event_loop.play_stream(stream_id.clone());

    let sample_rate = format.sample_rate.0 as f32;
    let mut sample_clock = 0f32;
    let mut rand_range = RandRange::new(440.0, 220.0, 880.0, 0.005);

    println!("{:?}", sample_rate);
    println!("{:?}", sample_clock);

    let mut next_value = || {
        sample_clock = (sample_clock + 1.0 ) % sample_rate;
        rand_range.next(sample_clock, sample_rate)
        // (sample_clock * frequency * 2.0 * 3.141592 / sample_rate).sin()
    };

    event_loop.run( move |_, data| {
        match data {
            cpal::StreamData::Output { buffer: cpal::UnknownTypeOutputBuffer::U16(mut buffer) } => {
                for sample in buffer.chunks_mut(format.channels as usize) {
                    let value = ((next_value() * 0.5 + 0.5) * std::u16::MAX as f32) as u16;
                    for out in sample.iter_mut() {
                        *out = value;
                    }
                }
            },
            cpal::StreamData::Output { buffer: cpal::UnknownTypeOutputBuffer::I16(mut buffer) } => {
                for sample in buffer.chunks_mut(format.channels as usize) {
                    let value = (next_value() * std::i16::MAX as f32) as i16;
                    for out in sample.iter_mut() {
                        *out = value;
                    }
                }
            },
            cpal::StreamData::Output { buffer: cpal::UnknownTypeOutputBuffer::F32(mut buffer) } => {
                for sample in buffer.chunks_mut(format.channels as usize) {
                    let value = next_value();
                    for out in sample.iter_mut() {
                        *out = value;
                    }
                }
            },
            _ => (),
        }
    });
}

struct RandRange {
    curr: f32,
    next: f32,
    step: f32,
    direction: i8,
    max: f32,
    min: f32,
}

impl RandRange {
    fn next(&mut self, sample_clock: f32, sample_rate: f32) -> f32 {
        let mut new_next = self.next + (self.step * self.direction as f32);
        if new_next > self.max || new_next < self.min {
            self.direction *= -1;
            new_next = self.next + (self.step * self.direction as f32);
        }
        self.curr = self.next;
        self.next = new_next;
        (sample_clock * self.curr * 2.0 * 3.141592 / sample_rate).sin() * 7.0
    }

    fn new(curr: f32, min: f32, max: f32, step: f32) -> RandRange {
        let next = curr + step;
        let direction = 1;
        RandRange{curr, next, step, direction, max, min}
    }
}
