use minifb::{Key, Window, WindowOptions};
use std::time::Instant;

// Window dimensions
const WIDTH: usize = 800;
const HEIGHT: usize = 600;

// Scale factor: pixels per meter
const SCALE: f32 = 10.0;

// Physics constants
const M: f32 = 1000.0;           // Aircraft mass in kg
const G: f32 = 9.8;              // Gravity in m/s^2
const T_MAX: f32 = 10000.0;      // Maximum thrust in N
const K_D: f32 = 0.1;            // Drag coefficient
const K_L: f32 = 10.0;           // Lift coefficient
const PITCH_RATE_MAX: f32 = 0.1745; // Max pitch rate in rad/s (10 deg/s)
const THROTTLE_CHANGE_RATE: f32 = 0.5; // Throttle change rate per second

// Color constants (0xRRGGBB)
const SKY_COLOR: u32 = 0x87CEEB;
const GROUND_COLOR: u32 = 0x228B22;
const AIRCRAFT_COLOR: u32 = 0xFF0000;

// Aircraft struct to hold state
struct Aircraft {
    x: f32,           // Horizontal position in meters
    y: f32,           // Altitude in meters
    vx: f32,          // Horizontal velocity in m/s
    vy: f32,          // Vertical velocity in m/s
    theta: f32,       // Pitch angle in radians
    throttle_level: f32, // Throttle level (0.0 to 1.0)
}

impl Aircraft {
    /// Create a new aircraft with initial state
    fn new() -> Self {
        Aircraft {
            x: 0.0,
            y: 100.0,    // Start at 100m altitude
            vx: 50.0,    // Initial horizontal speed of 50 m/s
            vy: 0.0,
            theta: 0.0,
            throttle_level: 0.0,
        }
    }

    /// Update aircraft state based on physics and input
    fn update(&mut self, dt: f32, pitch_rate: f32) {
        // Update pitch angle and clamp it between -PI/2 and PI/2 radians (-90 to +90 degrees)
        self.theta += pitch_rate * dt;
        self.theta = self.theta.clamp(-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2);

        // Compute total speed
        let s = (self.vx.powi(2) + self.vy.powi(2)).sqrt();

        // Compute velocity direction
        let phi = self.vy.atan2(self.vx);

        // Angle of attack
        let alpha = self.theta - phi;

        // Calculate forces
        let lift = K_L * s.powi(2) * alpha;
        let drag = K_D * s.powi(2);
        let thrust = T_MAX * self.throttle_level;

        // Forces in x and y directions
        let (f_x, f_y) = if s > 0.001 { // Avoid division by zero
            let drag_x = drag * self.vx / s;
            let drag_y = drag * self.vy / s;
            let lift_dir_x = -self.vy / s; // Perpendicular to velocity
            let lift_dir_y = self.vx / s;
            (
                thrust * self.theta.cos() - drag_x - lift * lift_dir_x,
                thrust * self.theta.sin() - drag_y + lift * lift_dir_y - M * G,
            )
        } else {
            (
                thrust * self.theta.cos(),
                thrust * self.theta.sin() - M * G,
            )
        };

        // Update velocities
        self.vx += (f_x / M) * dt;
        self.vy += (f_y / M) * dt;

        // Update position
        self.x += self.vx * dt;
        self.y += self.vy * dt;

        // Prevent aircraft from going below ground and stop movement
        if self.y < 0.0 {
            self.y = 0.0;
            self.vy = 0.0;
            self.vx = 0.0; // Stop horizontal movement on ground impact
            self.theta = 0.0; // Level the aircraft on ground impact
        }
    }
}

/// Draw a line on the buffer using Bresenham's algorithm
fn draw_line(buffer: &mut [u32], width: usize, height: usize, x0: i32, y0: i32, x1: i32, y1: i32, color: u32) {
    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx - dy;
    let mut x = x0;
    let mut y = y0;

    loop {
        if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
            buffer[y as usize * width + x as usize] = color;
        }
        if x == x1 && y == y1 { break; }
        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            x += sx;
        }
        if e2 < dx {
            err += dx;
            y += sy;
        }
    }
}

fn main() {
    // Initialize window
    let mut window = Window::new(
        "Flight Simulator - Press ESC to Exit",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    ).unwrap_or_else(|e| panic!("Failed to create window: {}", e));

    // Create pixel buffer
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    // Initialize aircraft
    let mut aircraft = Aircraft::new();

    // Timing variables
    let mut last_time = Instant::now();
    let frame_time = 1.0 / 60.0; // Target 60 FPS

    // Main game loop
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let current_time = Instant::now();
        let dt = current_time.duration_since(last_time).as_secs_f32().min(0.1); // Cap dt to prevent large jumps
        last_time = current_time;

        // Handle input
        let mut pitch_rate = 0.0;
        if window.is_key_down(Key::Up) {
            pitch_rate = PITCH_RATE_MAX;
        } else if window.is_key_down(Key::Down) {
            pitch_rate = -PITCH_RATE_MAX;
        }

        let mut throttle_change = 0.0;
        if window.is_key_down(Key::W) {
            throttle_change = THROTTLE_CHANGE_RATE;
        } else if window.is_key_down(Key::S) {
            throttle_change = -THROTTLE_CHANGE_RATE;
        }
        aircraft.throttle_level += throttle_change * dt;
        aircraft.throttle_level = aircraft.throttle_level.clamp(0.0, 1.0);

        // Update aircraft physics
        aircraft.update(dt, pitch_rate);

        // Render scene
        // Clear buffer with sky color
        buffer.fill(SKY_COLOR);

        // Draw ground
        let screen_y_ground = (HEIGHT as f32 / 2.0 - aircraft.y * SCALE).round() as i32;
        let start_y = screen_y_ground.max(0).min(HEIGHT as i32);
        for y in start_y..HEIGHT as i32 {
            let row_start = y as usize * WIDTH;
            buffer[row_start..row_start + WIDTH].fill(GROUND_COLOR);
        }

        // Draw aircraft as a triangle
        let cos_theta = aircraft.theta.cos();
        let sin_theta = aircraft.theta.sin();
        // Define local coordinates (in meters)
        let nose_local = (1.0 * cos_theta, 1.0 * sin_theta);
        let left_wing_local = (-0.5 * cos_theta - (-0.2) * sin_theta, -0.5 * sin_theta + (-0.2) * cos_theta);
        let right_wing_local = (-0.5 * cos_theta - 0.2 * sin_theta, -0.5 * sin_theta + 0.2 * cos_theta);

        // Convert to screen coordinates (aircraft is centered)
        let nose_screen = (
            (WIDTH as f32 / 2.0 + nose_local.0 * SCALE).round() as i32,
            (HEIGHT as f32 / 2.0 - nose_local.1 * SCALE).round() as i32,
        );
        let left_wing_screen = (
            (WIDTH as f32 / 2.0 + left_wing_local.0 * SCALE).round() as i32,
            (HEIGHT as f32 / 2.0 - left_wing_local.1 * SCALE).round() as i32,
        );
        let right_wing_screen = (
            (WIDTH as f32 / 2.0 + right_wing_local.0 * SCALE).round() as i32,
            (HEIGHT as f32 / 2.0 - right_wing_local.1 * SCALE).round() as i32,
        );

        // Draw aircraft lines
        draw_line(
            &mut buffer, WIDTH, HEIGHT,
            nose_screen.0, nose_screen.1,
            left_wing_screen.0, left_wing_screen.1,
            AIRCRAFT_COLOR,
        );
        draw_line(
            &mut buffer, WIDTH, HEIGHT,
            nose_screen.0, nose_screen.1,
            right_wing_screen.0, right_wing_screen.1,
            AIRCRAFT_COLOR,
        );
        draw_line(
            &mut buffer, WIDTH, HEIGHT,
            left_wing_screen.0, left_wing_screen.1,
            right_wing_screen.0, right_wing_screen.1,
            AIRCRAFT_COLOR,
        );

        // Update window
        window.update_with_buffer(&buffer, WIDTH, HEIGHT)
            .unwrap_or_else(|e| panic!("Failed to update window: {}", e));

        // Maintain frame rate
        let elapsed = current_time.elapsed().as_secs_f32();
        if elapsed < frame_time {
            std::thread::sleep(std::time::Duration::from_secs_f32(frame_time - elapsed));
        }
    }
}