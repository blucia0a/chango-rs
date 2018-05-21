extern crate cv;
extern crate cpal;
use cv::highgui::*;
use cv::videoio::VideoCapture;
use std::thread;

fn main() {

  let audio_thread = thread::spawn(|| {
    do_audio();
  }); 

  //Apparently OpenCV doesn't like to be in a thread...
  do_video();

  audio_thread.join().unwrap(); 
}

fn do_audio() {

  let device = cpal::default_output_device().expect("Failed to get default output device");
  let format = device.default_output_format().expect("Failed to get default output format");
  let event_loop = cpal::EventLoop::new();
  let stream_id = event_loop.build_output_stream(&device, &format).unwrap();
  event_loop.play_stream(stream_id.clone());

  let sample_rate = format.sample_rate.0 as f32;
  const NUMGENS: usize = 8;

  let c_scale = [261.,293.,329.,349.,392.,440.,493.,523.]; 
  //let c_scale = [349.,392.,440.,493.,523.]; 

  //Initialize an array of wave generator closures
  let mut gens: [_; NUMGENS] = unsafe {std::mem::uninitialized() };
  for i in 0..NUMGENS {
    let mut clk = 0f32;
    let mut freq = c_scale[i];
    gens[i] = move || {
      //println!("fatty: c {} f {}",clk, freq);
      clk = (clk + 1.0) % sample_rate;
      (clk * freq * 2.0 * 3.1415926 / sample_rate).sin()
    };
  } 

  // Produce a sinusoid of maximum amplitude.
  let mut getgen = |gen : usize| {
    gens[gen]()
  };

  //let mut gclk = 0f32;
  let mut next_value = || {
      let mut out = 0f32; 
      for i in 0..NUMGENS {
        //println!("i {}",i);
        out += getgen(i); 
      }
      out / (NUMGENS as f32)
      //getgen(0)
      //gclk = (gclk + 1.0) % sample_rate;
      //(gclk * 220. * 2.0 * 3.1415926 / sample_rate).sin() / 1.0
  };


  event_loop.run(move |_, data| {
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

fn do_video() { 
  let cap = VideoCapture::new(0);
  assert!(cap.is_open());

  highgui_named_window("Window", WindowFlag::Autosize).unwrap();
  while let Some(image) = cap.read() {
      image.show("Window", 30).unwrap();
  }

}
