// N-Body Problem Simulation
//
// This project simulates the n-body problem using the macroquad library for graphics.
// The simulation includes a first-person camera, body interactions, and energy calculations.
//
// The main logic is implemented in the `main` function, which is defined in `src/main.rs`.
// Functions and data structures for all calculations  are defined in a separate module, n-body in `src/lib.rs`.
//
// A data.json file is required in the root directory of the program. It contains all data on bodies.
// Sample data.json containing 3 bodies:
////////////////////////////////
 /*
 [
  {
    "name": "Body1",
    "position": [100, 200, -150],
    "velocity": [10, -20, 5],
    "mass": 5.0e16
  },
  {
    "name": "Body2",
    "position": [-120, 150, 200],
    "velocity": [20, 10, -10],
    "mass": 4.5e16
  },
  {
    "name": "Body3",
    "position": [300, -500, 400],
    "velocity": [-15, 20, -5],
    "mass": 6.0e16
  }
 ]
*/
////////////////////////////////

use macroquad::prelude::*;
use n_body_problem::n_body::Bodies;

const MOVE_SPEED: f32 = 3.6;
const LOOK_SPEED: f32 = 0.1;
//const ZOOM_SPEED: f32 = 0.1;


fn conf() -> Conf {
    Conf {
        window_title: String::from("N Body Problem Simulation"),
        window_width: 1260,
        window_height: 768,
        fullscreen: false,
        ..Default::default()
    }
}

#[macroquad::main(conf)]
async fn main() {

    //Initialize  bodies
    let mut bodies = Bodies::new();
    bodies.parse_json("data.json");

    //axis configuration variables
    let world_up = vec3(0.0, 1.0, 0.0);
    let mut yaw: f32 = 1.18;
    let mut pitch: f32 = 0.0;

    let mut front = vec3(
        yaw.cos() * pitch.cos(),
        pitch.sin(),
        yaw.sin() * pitch.cos(),
    )
        .normalize();
    let mut right = front.cross(world_up).normalize();
    let mut up;

    let  default_position = bodies.bodies[0].position +
        vec3(
            bodies.bodies[0].radius + 100.0,
            bodies.bodies[0].radius + 100.0,
            bodies.bodies[0].radius + 100.0,) ; // initial camera position. Close to one of the bodies.
    let mut position = default_position; //Camera position
    let mut last_mouse_position: Vec2 = mouse_position().into();

    let mut grabbed = true; // For when the program grabs the mouse. <TAB> will be used to toggle the grab.
    set_cursor_grab(grabbed);
    show_mouse(false);

    //let mut zoom: f32 = 1.0; //Initial zoom level
    let mut is_paused: bool = false;
    let mut second:f32 = 0.0;
    let mut fps=0;


    loop {
        //Clear background to render new image
        clear_background(BLACK);

        let delta = get_frame_time(); //Time elapsed since last frame was drawn.

        // Handle mouse movement and control events.
            //quit
        if is_key_pressed(KeyCode::Escape) {
            break;
        }
         //Mouse grab
        if is_key_pressed(KeyCode::Tab) {
            grabbed = !grabbed;
            set_cursor_grab(grabbed);
            show_mouse(!grabbed);
        }
            //Pause functionality
        // Check if the space key is pressed to pause or resume the program
        if is_key_pressed(KeyCode::Space) {
            is_paused = !is_paused;
            grabbed  = !grabbed; //Pausing will release the mouse
            set_cursor_grab(grabbed);
            show_mouse(!grabbed);
        }
        // If the program is paused, skip the rest of the loop
        if is_paused {
            set_default_camera();
            //print paused on the screen
            draw_text("Paused", 50.0, 50.0, 50.0, BLUE);
            next_frame().await;
            continue;
        }


        //Button to restart the simulation
        if is_key_pressed(KeyCode::R) {
            bodies = Bodies::new();
            bodies.parse_json("data.json");

        }
        if is_key_pressed(KeyCode::C){
            position = default_position; //reset camera
        }
            //Movement
        if is_key_down(KeyCode::Up) ||is_key_down(KeyCode::W) {
            position += front * MOVE_SPEED;
        }
        if is_key_down(KeyCode::Down) ||is_key_down(KeyCode::S) {
            position -= front * MOVE_SPEED;
        }
        if is_key_down(KeyCode::Left) ||is_key_down(KeyCode::A) {
            position -= right * MOVE_SPEED;
        }
        if is_key_down(KeyCode::Right) ||is_key_down(KeyCode::D) {
            position += right * MOVE_SPEED;
        }
        // Handle mouse wheel zoom
        /*if is_mouse_button_down(MouseButton::Middle) {
            let mouse_wheel_delta: f32 = mouse_wheel().1;
            zoom *= 1.0 + mouse_wheel_delta * ZOOM_SPEED;
        }*/

        let mouse_position: Vec2 = mouse_position().into();
        let mouse_delta = mouse_position - last_mouse_position;
        last_mouse_position = mouse_position;

        yaw += mouse_delta.x * delta * LOOK_SPEED;
        pitch += mouse_delta.y * delta * -LOOK_SPEED;

        pitch = if pitch > 1.5 { 1.5 } else { pitch };
        pitch = if pitch < -1.5 { -1.5 } else { pitch };

        front = vec3(
            yaw.cos() * pitch.cos(),
            pitch.sin(),
            yaw.sin() * pitch.cos(),
        )
            .normalize();

        right = front.cross(world_up).normalize();
        up = right.cross(front).normalize();

           // Going 3d!
        set_camera(&Camera3D {
            position: position,
            up: up,
            target: position + front,
            ..Default::default()
        });

        //draw_grid(2000, 50., BLACK, GRAY);

        //Render bodies.
        bodies.draw();
        bodies.apply_force(delta); //Change the velocities of the bodies due to G forces.
        bodies.update(delta); //Update the positions of the bodies.

        // Back to screen space, render info text

        set_default_camera();
        draw_text("First Person Camera: Press [C] to reset", 10.0, 20.0, 18.0, WHITE);

        draw_text(
            format!("Press [TAB] to toggle mouse grab: {} Press [R] to restart simulation", grabbed).as_str(),
            10.0,
            50.0,
            18.0,
            WHITE,
        );

        //Status bar
        // Draw a rectangle at the bottom of the screen to display status information
        draw_rectangle(
            0.0,
            screen_height() - 50.0,
            screen_width(),
            60.0,
            DARKGRAY
        );

        // Display the number of bodies
        draw_text(
            &format!("Bodies: {}", bodies.bodies.len()),
            10.0,
            screen_height() - 30.0,
            17.0,
            WHITE
        );

        // Display the kinetic energy
        draw_text(
            &format!("Kinetic Energy: {:.4e}", bodies.kinetic_energy),
            150.0,
            screen_height() - 30.0,
            17.0,
            WHITE
        );

        // Display the potential energy
        draw_text(
            &format!("Potential Energy: {:.4e}", bodies.potential_energy),
            500.0,
            screen_height() - 30.0,
            17.0,
            WHITE
        );

        // Display the total energy
        draw_text(
            &format!(
                "Total Energy: {:.4e}",
                bodies.potential_energy + bodies.kinetic_energy
            ),
            800.0,
            screen_height() - 30.0,
            17.0,
            WHITE
        );

        // Display the time-averaged kinetic energy
        draw_text(
            &format!("Time Averaged: {:.4e}", bodies.time_averaged_kinetic_energy),
            150.0,
            screen_height() - 5.0,
            17.0,
            WHITE
        );

        // Display the time-averaged potential energy
        draw_text(
            &format!("Time Averaged: {:.4e}", bodies.time_averaged_potential_energy),
            500.0,
            screen_height() - 5.0,
            17.0,
            WHITE
        );

        // Display the time-averaged total energy
        draw_text(
            &format!(
                "Time Averaged: {:.4e}",
                bodies.time_averaged_kinetic_energy + bodies.time_averaged_potential_energy
            ),
            800.0,
            screen_height() - 5.0,
            17.0,
            WHITE
        );

        // Display elapsed time
        draw_text(
            &format!("Elapsed Time: {:.3}s", bodies.total_time),
            1000.0,
            screen_height() - 30.0,
            17.0,
            WHITE
        );

        // Display FPS
        second += delta; // increment counter. About 1 second has passed if second.floor()==1.0.
        if second.floor() ==1.0 { // get fps only if a second has elapsed.
            fps = get_fps();
            second = 0.0; //reset counter.
        }
        draw_text(
            &format!("FPS: {}",fps ),
            1000.0,
            screen_height() - 5.0,
            17.0,
            WHITE
        );

        next_frame().await
    }
}
