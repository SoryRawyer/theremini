extern crate cpal;
extern crate serialport;

use std::io;

use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::thread;

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
    let (tx, rx): (Sender<i32>, Receiver<i32>) = mpsc::channel();
    let mut i = 0;
    let mut j = 0;

    thread::spawn(move || {
        loop {
            // let mut now = Instant::now();
            j += 1;
            if j % 1000 == 0 {
                tx.send(port_reader.read_value()).unwrap();
            }
            // times.push(now.elapsed().subsec_nanos());
        }
    });

    println!("{:?}", sample_rate);
    println!("{:?}", sample_clock);

    let mut next_value = move || {
        // let now = Instant::now();
        i += 1;
        if i % 1000 == 0 {
            if let Ok(val) = rx.recv() {
                // println!("{:?}", val);
                frequency = (val + 100) as f32; 
            }
        }

        // frequency = (port_reader.read_value() + 300) as f32;
        sample_clock = (sample_clock + 1.0 ) % sample_rate;
        // println!("{:?}", now.elapsed());
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
                        // if we're at a newline:
                        //   append buffer to self.unread_stuff, then clear self.unread_stuff
                        if serial_buf[i] == 10 {
                            // println!("{:?}", value);
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

