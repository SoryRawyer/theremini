extern crate cpal;
extern crate serialport;

use std::io;

fn main() {
    let device = cpal::default_output_device().expect("failed to get output device");
    let format = device.default_output_format().expect("failed to get output format");
    let event_loop = cpal::EventLoop::new();
    let stream_id = event_loop.build_output_stream(&device, &format).unwrap();
    event_loop.play_stream(stream_id.clone());

    let sample_rate = format.sample_rate.0 as f32;
    let mut sample_clock = 0f32;
    let mut frequency: f32 = 440.0;
    let mut port_reader = PortReader::new("/dev/cu.usbmodem1411");
    // let mut amplitude: f32 = 0.0;
    // let mut frequency_range = RandRange::new(440.0, 220.0, 880.0, 0.005);
    // let mut amplitude_range = RandRange::new(1.0, 0.5, 7.0, 0.005);

    // let mut port = serialport::open("/dev/cu.usbmodem1411").unwrap();
    // let mut serial_buf: Vec<u8> = vec![0; 1];

    println!("{:?}", sample_rate);
    println!("{:?}", sample_clock);

    let mut next_value = || {
        sample_clock = (sample_clock + 1.0 ) % sample_rate;
        // frequency = (port_reader.read_value() + 300) as f32;
        // println!("{:?}", frequency);
        // amplitude = amplitude_range.next();
        // println!("frequency: {}", frequency);
        // println!("amplitude: {}", amplitude);
        // A * sin(2 * pi * frequency * (sample clock / sample rate (44.1khz)))
        // amplitude * (sample_clock * frequency * 2.0 * 3.141592 / sample_rate).sin()
        // match port.read(serial_buf.as_mut_slice()) {
        //     // Ok(t) => io::stdout().write_all(&serial_buf[..t]).unwrap(),
        //     Ok(_) => {
        //         frequency = serial_buf[0] as f32;
        //         println!("frequency: {}", frequency);
        //     },
        //     Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
        //     Err(e) => println!("got this error: {:?}", e),
        // };
        (2.0 * (sample_clock * frequency * 2.0 * 3.141592 / sample_rate).sin())
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

// RandRange will take maximum and minimum values and a step, then fluctuate
struct RandRange {
    curr: f32,
    next: f32,
    step: f32,
    direction: i8,
    max: f32,
    min: f32,
}

impl RandRange {
    fn next(&mut self) -> f32 {
        let mut new_next = self.next + (self.step * self.direction as f32);
        if new_next > self.max || new_next < self.min {
            self.direction *= -1;
            new_next = self.next + (self.step * self.direction as f32);
        }
        self.curr = self.next;
        self.next = new_next;
        self.curr
    }

    fn new(curr: f32, min: f32, max: f32, step: f32) -> RandRange {
        let next = curr + step;
        let direction = 1;
        RandRange{curr, next, step, direction, max, min}
    }
}

struct PortReader {
    port: Box<serialport::SerialPort>,
    unread_stuff: Vec<u8>,
}

impl PortReader {

    // new: the PortReader has logged on
    fn new(port_addr: &str) -> PortReader {
        if let Ok(port) = serialport::open(&port_addr) {
            let unread_stuff: Vec<u8> = Vec::new();
            PortReader{port, unread_stuff}
        } else {
            panic!("omgomgomg")
        }
    }

    // read from the port until we have a whole value
    fn read_value(&mut self) -> i32 {
        let mut serial_buf: Vec<u8> = vec![0,5];
        let mut value: Vec<u8> = self.unread_stuff.clone();
        loop {
            match self.port.read(serial_buf.as_mut_slice()) {
                Ok(t) => {
                    for i in 0..t {
                        // if we're at a newline:
                        //   append buffer to self.unread_stuff, then clear self.unread_stuff
                        if serial_buf[i] == 10 {
                            // println!("{:?}", value);
                            self.unread_stuff = serial_buf[(i+1)..t].to_vec();
                            return String::from_utf8(value).unwrap().parse().unwrap()
                        } else {
                            value.extend(vec!(serial_buf[i]));
                        }
                    }
                },
                Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                Err(e) => println!("got this error: {:?}", e),
            }
        }
    }

}

