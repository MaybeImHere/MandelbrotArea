use rand;
use ctrlc;

use std::{thread, time};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;

#[derive(Clone)]
struct Point {
    x: f64,
    y: f64
}

impl Point {
    // returns a random point within (not on the boundary) the unit box.
    fn rand_point() -> Point {
        Point {
            x: rand::random::<f64>() * 2.0 - 1.0,
            y: rand::random::<f64>() * 2.0 - 1.0
        }
    }
    
    // returns the coordinate (0, 0)
    fn origin() -> Point {
        Point {
            x: 0.0,
            y: 0.0
        }
    }
    
    // returns the distance of the point to the origin.
    fn dist_origin(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
    
    // multiplies the point by a scalar real number.
    fn mult_scalar(&mut self, scalar: f64) {
        self.x *= scalar;
        self.y *= scalar;
    }
    
    // generates a random point contained within (not on the boundary) the circle of radius `radius` centered at the origin.
    fn rand_circle(radius: f64) -> Point {
        loop {
            let mut p = Point::rand_point();
            p.mult_scalar(radius);
            
            if p.dist_origin() < radius {
                return p;
            }
        }
    }
    
    // treats this point as the complex number x + y * i and squares it.
    fn complex_square(&mut self) -> &mut Point {
        // the real component
        let new_x = self.x * self.x - self.y * self.y;
        // the imaginary component
        let new_y = 2.0 * self.x * self.y;
        
        self.x = new_x;
        self.y = new_y;
        
        self
    }
    
    // treats this point as the complex number x + y * i and adds other to it (treating other as a complex number in the same way).
    fn complex_add(&mut self, other: &Point) -> &mut Point {
        self.x += other.x;
        self.y += other.y;
        
        self
    }
    
    // returns whether this point is in the mandelbrot set.
    fn is_mandelbrot(&self, mandelbrot_iters: u64) -> bool {
        let mut z = Point::origin();
        z.complex_add(self);
        // &self is c in this scenario.
        // we will do 50 iterations.
        
        let mut i: u64 = 0;
        while i < mandelbrot_iters {
            z.complex_square().complex_add(self);
            
            // escape condition.
            if z.x * z.x + z.y + z.y > 4.0 {
                return false;
            }
            
            i += 1;
        }
        
        return true;
    }
}

// will calculate how many random (on a circle of radius 2) fall within the mandelbrot set.
// returns a points inside set
// iterations is how many points to sample.
fn calc_points(iterations: u64, mandelbrot_iters: u64) -> u64 {
    let mut i: u64 = 0;
    let mut ret: u64 = 0;
    
    while i < iterations {
        let p = Point::rand_circle(2.0);
        if p.is_mandelbrot(mandelbrot_iters) { ret += 1; }
        i += 1;
    }
    
    ret
}

const FOURPI: f64 = 4.0 * 3.141592653589793238462643383279502;

fn main() {
    // section for setting up ctrl c handler.
    let tried_exiting = Arc::new(AtomicBool::new(false));
    
    let ctrlc_tried_exiting = Arc::clone(&tried_exiting);
    ctrlc::set_handler(move || {
        if ctrlc_tried_exiting.load(Ordering::SeqCst) {
            println!("Force exiting now!");
            std::process::exit(1);
        } else {
            ctrlc_tried_exiting.store(true, Ordering::SeqCst);
        }
        
        println!("Ending program.");
    }).expect("Error setting Ctrl-C handler.");
    
    // the part to calculate points.
    
    // -- Constants --
    // how many points to sample.
    let iter_group_size: u64 = 100000;
    
    // how many times to iterate each point.
    let mandelbrot_iters: u64 = 1000;
    
    // how many threads to run.
    let threads = 6;
    
    // -- Thread Variables --
    // these will be used to transmit all of the mandelbrot data from the child threads to the main thread.
    let (tx_main, rx_main) = mpsc::channel();
    
    // will be used to send the exit signal to every thread.
    let mut thread_exit_channels: Vec<mpsc::Sender<()>> = vec![];
    
    // -- Thread Creation --
    // for counting number of threads.
    let mut threads_temp = threads;
    while threads_temp > 0 {
        let tx_clone = tx_main.clone();
        
        let (tx_exiting, rx_exiting) = mpsc::channel();
        thread_exit_channels.push(tx_exiting.clone());
        
        thread::spawn(move || {
            loop {
                match rx_exiting.try_recv() {
                    // we need to exit.
                    Ok(()) => {
                        break;
                    },
                    // no data received, we are fine.
                    Err(mpsc::TryRecvError::Empty) => {
                        tx_clone.send(calc_points(iter_group_size, mandelbrot_iters)).expect("Could not send mandelbrot data through channel!");
                    },
                    // thread somehow got disconnected from main, exit now.
                    Err(mpsc::TryRecvError::Disconnected) => {
                        break;
                    }
                }
            }
        });
        
        threads_temp -= 1;
    }
    
    println!("Running threads...");
    
    // -- Mutables --
    // number of points in the set.
    let mut in_set: u64 = 0;
    
    // number of points total.
    let mut total: u64 = 0;

    let mut last_area: f64 = f64::NAN;
   
    // -- Data Collection --
    let data_col_tried_exiting = Arc::clone(&tried_exiting);
    
    let mut start = time::Instant::now();
    loop {
        // calculate if we should print the area.
        let area = FOURPI * (in_set as f64) / (total as f64);
        if start.elapsed().as_secs() > 1 && area != last_area && !area.is_nan() {
            start = time::Instant::now();
            println!("Area: {}", area);
            last_area = area;
        }
        
        // if tried_exiting is true, we want to begin the exit process.
        if data_col_tried_exiting.load(Ordering::SeqCst) {
            for thread_exit_channel in thread_exit_channels.iter() {
                thread_exit_channel.send(()).expect("Could not send exit signal to child threads!");
            }
            
            break;
        }
        
        match rx_main.try_recv() {
            // we have data
            Ok(points_inside) => {
                in_set += points_inside;
                total += iter_group_size;
            },
            // no data, just ignore
            Err(mpsc::TryRecvError::Empty) => {},
            
            // thread somehow got disconnected from all of the others, very weird.
            Err(mpsc::TryRecvError::Disconnected) => {
                panic!("Main thread disconnected from child threads!");
            }
        }
    }
    
    println!("Final data: \n\tInside set: {}\n\tTotal: {}\n\tArea: {}", in_set, total, FOURPI * (in_set as f64) / (total as f64));

    /*
    ctrlc::set_handler(move || running.store(false, Ordering::Relaxed))
        .expect("Error setting Ctrl-C handler.");
    */
}
