// Physics constants
const M: f32 = 1000.0;           // Aircraft mass in kg
const G: f32 = 9.8;              // Gravity in m/s^2
const T_MAX: f32 = 10000.0;      // Maximum thrust in N
const K_D: f32 = 0.1;            // Drag coefficient
const K_L: f32 = 10.0;           // Lift coefficient
const PITCH_RATE_MAX: f32 = 0.1745; // Max pitch rate in rad/s (10 deg/s)
const THROTTLE_CHANGE_RATE: f32 = 0.5; // Throttle change rate per second

// Input state structure
pub struct InputState {
    pub pitch_up: bool,
    pub pitch_down: bool,
    pub throttle_up: bool,
    pub throttle_down: bool,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            pitch_up: false,
            pitch_down: false,
            throttle_up: false,
            throttle_down: false,
        }
    }
}

// Aircraft struct to hold state
pub struct Aircraft {
    pub x: f32,           // Horizontal position in meters
    pub y: f32,           // Altitude in meters
    pub vx: f32,          // Horizontal velocity in m/s
    pub vy: f32,          // Vertical velocity in m/s
    pub theta: f32,       // Pitch angle in radians
    pub throttle_level: f32, // Throttle level (0.0 to 1.0)
    pub input: InputState, // Current input state
}

impl Aircraft {
    /// Create a new aircraft with initial state
    pub fn new() -> Self {
        Aircraft {
            x: 0.0,
            y: 100.0,    // Start at 100m altitude
            vx: 50.0,    // Initial horizontal speed of 50 m/s
            vy: 0.0,
            theta: 0.0,
            throttle_level: 0.0,
            input: InputState::default(),
        }
    }

    /// Update aircraft state based on physics and input
    pub fn update(&mut self, dt: f32) {
        // Process control inputs
        let mut pitch_rate = 0.0;
        if self.input.pitch_up {
            pitch_rate = PITCH_RATE_MAX;
        } else if self.input.pitch_down {
            pitch_rate = -PITCH_RATE_MAX;
        }

        let mut throttle_change = 0.0;
        if self.input.throttle_up {
            throttle_change = THROTTLE_CHANGE_RATE;
        } else if self.input.throttle_down {
            throttle_change = -THROTTLE_CHANGE_RATE;
        }
        self.throttle_level += throttle_change * dt;
        self.throttle_level = self.throttle_level.clamp(0.0, 1.0);

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