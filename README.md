### theremini — a theremin using rust and arduino

given a frequency and amplitude, generate f32 samples  
uses:
- [cpal](https://github.com/tomaka/cpal) to make the samples audible  
- [serialport-rs](https://github.com/Susurrus/serialport-rs) to read data from the arduino

todo:
- add another sensor to control the amplitude of the signal
- smooth out the sample generating so that we can get rid of all that cracklin'
