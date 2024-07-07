/// # lib.rs - A Rust Library for N-Body Simulation
///
/// This library provides a simple implementation of an N-body simulation. The simulation
/// involves a collection of celestial bodies, such as planets or stars, interacting with
/// each other through gravitational forces.
///
/// ## Features
///
/// - Define a `Body` struct to represent a celestial body with properties like position,
///   velocity, and mass.
/// - Implement methods to update the position and velocity of each body based on
///   gravitational forces.
/// - Calculate the total kinetic and potential energy of the system.
/// - Parse body data from a JSON file and create a collection of `Body` instances.
/// - Draw the bodies in a 3D space using the `macroquad` graphics library.
///
pub mod n_body {
    use macroquad::prelude::*;
    use std::fs::File;
    use std::io;
    use io::Read;
    use json;
    const G: f32 = 6.67430e-11;
    const NUM_BODIES: usize = 3;
    const PAN_SPEED: f32 = 300.0;

    pub struct Body {
        pub position: Vec3,
        velocity: Vec3,
        mass: f32,
        trajectory: Vec<Vec3>,
        name: String,
        pub radius:f32 //This is calculated from mass. Its value will be  updated by Body::new().
    }

    impl Body {
        pub fn new(position: Vec3, velocity: Vec3, mass: f32,name:String) -> Self {
            Self {
                position,
                velocity,
                mass,
                trajectory: vec![],
                name,
                radius : mass.clone().powf(1.0/15.0) // The tenth root.
            }
        }

        /// Get the new position of the body.
       pub fn update(&mut self, dt: f32) {
            self.position += self.velocity * dt;
            self.trajectory.push(self.position);
            if self.trajectory.len() > 500 {
                self.trajectory.remove(0);
            }
        }

        //Apply the force to the body to change velocity
        pub fn apply_force(&mut self, force: Vec3, dt: f32) {
            let acceleration = force / self.mass;
            self.velocity += acceleration * dt;
        }

        //Draw the body
        pub fn draw(&self,radius:f32) {

            draw_sphere(self.position,radius,None, WHITE);
            for window in self.trajectory.windows(2) {
                if let [p1, p2] = window {
                    draw_line_3d(p1.clone(), p2.clone(), WHITE);
                }
            }
        }
    }

    //Calculate the force between two bodies
    fn gravitational_force(body1: &Body, body2: &Body) -> Vec3 {
        let direction = body2.position - body1.position;
        let distance = direction.length().max(1.0); // Prevent division by zero
        let force_magnitude = G * body1.mass * body2.mass / (distance * distance);
        direction.normalize() * force_magnitude
    }

    fn kinetic_energy(body: &Body) -> f32 {
        0.5 * body.mass * body.velocity.length_squared()
    }

    fn potential_energy(body1: &Body, body2: &Body) -> f32 {
        let distance = (body2.position - body1.position).length().max(1.0); // Prevent division by zero
        -G * body1.mass * body2.mass / distance
    }

    //Struct that will be used to calculate the motion of all bodies.
    pub struct Bodies {
        pub bodies: Vec<Body>,
        pub total_potential_energy:f32, //Total potential energy over time
        pub total_kinetic_energy: f32, // Total kinetic energy over time
        pub time_averaged_potential_energy:f32,
        pub time_averaged_kinetic_energy: f32,
        pub total_time: f32,
        pub kinetic_energy: f32, //Kinetic energy of the system.
        pub potential_energy: f32,//Potential energy of the system.
    }

    impl Bodies {
        pub fn new() -> Self {
            Self {
                bodies: Vec::new(),
                total_potential_energy: 0.0,
                total_kinetic_energy: 0.0,
                time_averaged_potential_energy: 0.0,
                time_averaged_kinetic_energy: 0.0,
                total_time: 0.0,
                kinetic_energy: 0.0,
                potential_energy: 0.0,
            }
        }

        pub fn add_body(&mut self, body: Body) {
            self.bodies.push(body);
        }
        //get the body data from data.json. Example data.json:
        /*
           [
              {
                "name": "Body 1",
                "position": [200, 20, 344],
                "velocity": [1, -2, 5],
                "mass": 1.0e12
              },
              {
                "name": "Body 2",
                "position": [400, 300, 50],
                "velocity": [-1, 0, 6],
                "mass": 2.0e16
              }
           ]
           */

        pub fn parse_json(&mut self,file_path: &str) {

            let mut file = match File::open(file_path) {
                Ok(file) => file,
                Err(e) => {
                    eprintln!("Error opening file: {}", e);
                    return;
                }
            };

            let mut data = String::new();
            if let Err(e) = file.read_to_string(&mut data) {
                eprintln!("Error reading file: {}", e);
                return ;
            }

            let parsed = match json::parse(&data) {
                Ok(parsed) => parsed,
                Err(e) => {
                    eprintln!("Error parsing JSON: {}", e);
                    return ;
                }
            };

            for body in parsed.members() {
                let name = match body["name"].as_str() {
                    Some(name) => name.to_string(),
                    None => {
                        eprintln!("Error: Missing or invalid name");
                        continue;
                    }
                };

                let position = match (
                    body["position"][0].as_f64(),
                    body["position"][1].as_f64(),
                    body["position"][2].as_f64(),
                ) {
                    (Some(x), Some(y), Some(z)) => Vec3::new(x as f32, y as f32, z as f32),
                    _ => {
                        eprintln!("Error: Invalid position");
                        continue;
                    }
                };

                let velocity = match (
                    body["velocity"][0].as_f64(),
                    body["velocity"][1].as_f64(),
                    body["velocity"][2].as_f64(),
                ) {
                    (Some(x), Some(y), Some(z)) => Vec3::new(x as f32, y as f32, z as f32),
                    _ => {
                        eprintln!("Error: Invalid velocity");
                        continue;
                    }
                };

                let mass = match body["mass"].as_f32() {
                    Some(mass) => mass,
                    None => {
                        eprintln!("Error: Invalid mass");
                        continue;
                    }
                };

                let body_instance = Body::new(position,velocity,mass,name);

                self.add_body(body_instance);
            }

        }
        //updates the positions of the bodies and energetics.
        pub fn update(&mut self, dt: f32) {
            // update the positions of the bodies
            for body in &mut self.bodies {
                body.update(dt);
            }
            //update energetics
            self.kinetic_energy = self.total_kinetic_energy(); // Energy at the current instant
            self.potential_energy = self.total_potential_energy();
            self.total_kinetic_energy += self.kinetic_energy; //Energy sum over time. This is used to calculate time averaged energy
            self.total_potential_energy += self.potential_energy;
            self.total_time += dt;
            self.time_averaged_kinetic_energy = self.total_kinetic_energy / self.total_time; //time averaged energy
            self.time_averaged_potential_energy = self.total_potential_energy / self.total_time;
        }

        //Apply the force on each body due to all other bodies. This must be called before calling body.update(),
        //because it changes the velocity of each body, which will be used by body.update().
        pub fn apply_force(&mut self, dt: f32) {
            for i in 0..self.bodies.len() {
                let mut total_force = Vec3::new(0.0, 0.0, 0.0);
                for j in 0..self.bodies.len() {
                    if i != j {
                        let force = gravitational_force(&self.bodies[i], &self.bodies[j]);
                        total_force += force;
                    }
                }
                self.bodies[i].apply_force(total_force, dt);
            }
        }
        pub fn draw(&self) {
            for body in &self.bodies {
                body.draw(body.radius);
            }
        }

        //Get the total kinetic energy of the system at the current instant
        fn total_kinetic_energy(&self) -> f32 {
            self.bodies.iter().map(|body| kinetic_energy(body)).sum()
        }

        //Get the total potential energy of the system at the current instant
        fn total_potential_energy(&self) -> f32 {
            let mut total_energy = 0.0;
            for i in 0..self.bodies.len() {
                for j in i + 1..self.bodies.len() {
                    total_energy += potential_energy(&self.bodies[i], &self.bodies[j]);
                }
            }
            total_energy
        }
    }
}