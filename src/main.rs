extern crate cv;
extern crate cpal;
use cv::highgui::*;
use cv::videoio::VideoCapture;
use std::thread;
use std::mem;
use std::ptr;
use std::sync::{Arc,Mutex};

//Constants
const NUMGENS: usize = 8;

fn main() {

  let mut amps: [Arc<Mutex<f32>>; NUMGENS]; 
  println!("did the unsafe thing");

  unsafe{
  amps = mem::uninitialized();
    for elem in &mut amps[..] {
  
      ptr::write( elem, Arc::new( Mutex::new( 0.5 ) ) );
  
    }
  }
  
  println!("did the init");
  
  { 
    let ampsa = amps.clone();
    let _ = thread::spawn(move || {
        do_audio(ampsa);
    }); 
  }

  //Apparently OpenCV doesn't like to be in a thread...
  {
    let ampsv = amps.clone();
    let video = move ||{
      let cap = VideoCapture::new(0);
      assert!(cap.is_open());
      
      let mut framecount = 2; //3 times per second
      highgui_named_window("Window", WindowFlag::Autosize).unwrap();
      while let Some(image) = cap.read() {
   
        framecount = framecount - 1; 
        if framecount == 0 {
          framecount = 2;
          for i in 0..NUMGENS{
            {
              let mut a = ampsv[i].lock().unwrap();
              *a = *a + 0.01;
              if *a >= 1.0 { *a = 0.0; }
            }
          }  
        }

        let windowname = "Rusty Chango"; 
        image.show(&windowname, 1).unwrap();
      }
    };
    video();
  }
  
}


fn do_audio(vamps: [Arc<Mutex<f32>>; NUMGENS]) {

  let device = cpal::default_output_device().expect("Failed to get default output device");
  let format = device.default_output_format().expect("Failed to get default output format");
  let event_loop = cpal::EventLoop::new();
  let stream_id = event_loop.build_output_stream(&device, &format).unwrap();
  event_loop.play_stream(stream_id.clone());

  let sample_rate = format.sample_rate.0 as f32;

  let c_scale = [261.,293.,329.,349.,392.,440.,493.,523.]; 

  //Initialize an array of wave generator closures
  let mut gens: [_; NUMGENS] = unsafe {std::mem::uninitialized()};
  let mut amps: [_; NUMGENS] = unsafe {std::mem::uninitialized()};

  for i in 0..NUMGENS {
    let mut clk = 0f32;
    let mut freq = c_scale[i];
    gens[i] = move || {
      clk = (clk + 1.0) % sample_rate;
      (clk * freq * 2.0 * 3.1415926 / sample_rate).sin()
    };
    amps[i] = 0.5;
  } 

  // Produce a sinusoid of maximum amplitude.
  let mut getgen = |gen : usize| {
    gens[gen]()
  };

  let finit = (sample_rate / 2000.) as u32; //every 2000 frames (~30x a second)
  let mut fcount = finit;
  let mut next_value = move || {

      let mut out = 0f32; 

      fcount = fcount - 1;
      if fcount == 0 {

        fcount = finit;
        for i in 0..NUMGENS {
          amps[i] = *vamps[i].lock().unwrap();
          out += getgen(i) * amps[i]; 
        }

      }else{

        for i in 0..NUMGENS {
          out += getgen(i) * amps[i]; 
        }

      }
      out / (NUMGENS as f32)

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

