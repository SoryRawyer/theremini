extern crate cpal;
extern crate serialport;

use std::io;

use std::thread;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::Arc;

fn main() {
    let device = cpal::default_output_device().expect("failed to get output device");
    let format = device.default_output_format().expect("failed to get output format");
    let event_loop = cpal::EventLoop::new();
    let stream_id = event_loop.build_output_stream(&device, &format).unwrap();
    event_loop.play_stream(stream_id.clone());

    let sample_rate = format.sample_rate.0 as f32;
    let mut sample_clock = 0f32;
    let sensor_value = Arc::new(AtomicIsize::new(0));
    let sensor_clone = sensor_value.clone();
    let mut frequency: f32 = 440.0;
    let mut port_reader = PortReader::new("/dev/cu.usbmodem1411");

    thread::spawn(move || {
        // store the average of the last 100 values
        let mut sensor_values: Vec<i32> = Vec::new();
        loop {
            if sensor_values.len() == 100 {
                sensor_values.remove(0);
            }
            sensor_values.push(port_reader.read_value());
            let mut sum = 0;
            for i in &sensor_values{
                sum += i;
            }
            let avg = sum as f32 / (sensor_values.len() as f32);
            sensor_clone.store(avg as isize, Ordering::Relaxed);
        }
    });

    println!("{:?}", sample_rate);
    println!("{:?}", sample_clock);

    // only read from the channel every thousandth sample
    // otherwise we won't be able to generate samples fast enough to produce an audible sound
    let mut next_value = || {;
        frequency = ((sensor_value.load(Ordering::Relaxed) as i32) + 100) as f32;
        // println!("{:?}", frequency);
        sample_clock = (sample_clock + 1.0 ) % sample_rate;
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
                        if serial_buf[i] == 10 {
                            self.unread_stuff = serial_buf[(i+1)..t].to_vec();
                            if let Ok(val) = String::from_utf8(value).unwrap().parse() {
                                return val
                            } else {
                                return 0
                            }
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

