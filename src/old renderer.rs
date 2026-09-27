use std::char;

use crate::{
    inputs,
    types::{Line, Size, Vec3, Vec4},
};
use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static! {
    pub static ref WORLD: Mutex<Vec<Line>> = Mutex::new(Vec::new());
}

pub fn draw_object(
    screen: &mut Vec<Vec<Vec<char>>>,
    terminal: &Size,
    fov_degrees: f32,
    aspect: f32,
    near: f32,
    far: f32,
) {
    let world = WORLD.lock().unwrap();
    for i in world.iter() {
        draw_line(
            Vec4 {
                x: i.point1.x,
                y: i.point1.y,
                z: i.point1.z,
                w: 1.0,
            },
            Vec4 {
                x: i.point2.x,
                y: i.point2.y,
                z: i.point2.z,
                w: 1.0,
            },
            screen,
            terminal,
            fov_degrees,
            aspect,
            near,
            far,
        );
    }
}

//convert 3d point to 2d only "add for each point"
fn draw_3d(point: Vec3, screen_size: &Size) -> Vec3 {
    let x = (point.x + 1.0) / 2.0 * screen_size.columns as f32;
    let y = (1.0 - point.y) / 2.0 * screen_size.lines as f32;
    //screen[iy as usize][ix as usize] = 'y';
    return Vec3 {
        x: x,
        y: y,
        z: point.z,
    };
}

fn perspective_matrix(fov_degrees: f32, aspect: f32, near: f32, far: f32) -> [[f32; 4]; 4] {
    let fov_rad = fov_degrees.to_radians();
    let f = 1.0 / (fov_rad / 2.0).tan(); // = 1/tan(FOV/2)

    [
        // Row 1
        [f / aspect, 0.0, 0.0, 0.0],
        // Row 2
        [0.0, f, 0.0, 0.0],
        // Row 3 (depth mapping)
        [
            0.0,
            0.0,
            (near + far) / (near - far),
            (2.0 * near * far) / (near - far),
        ],
        // Row 4 (perspective divide)
        [0.0, 0.0, -1.0, 0.0],
    ]
}

fn perspective(fov_degrees: f32, aspect: f32, near: f32, far: f32, point: Vec4) -> Vec3 {
    let b = mul_matrix_vec(perspective_matrix(fov_degrees, aspect, near, far), point);
    if (b.w <= 0.0) {
        return Vec3 {
            x: 0.0,
            y: 0.0,
            z: -1.0,
        };
    } else {
        let c = Vec3 {
            x: b.x / b.w,
            y: b.y / b.w,
            z: b.z / b.w,
        };
        c
    }
}

fn mul_matrix_vec(m: [[f32; 4]; 4], v: Vec4) -> Vec4 {
    Vec4 {
        x: m[0][0] * v.x + m[0][1] * v.y + m[0][2] * v.z + m[0][3] * v.w,
        y: m[1][0] * v.x + m[1][1] * v.y + m[1][2] * v.z + m[1][3] * v.w,
        z: m[2][0] * v.x + m[2][1] * v.y + m[2][2] * v.z + m[2][3] * v.w,
        w: m[3][0] * v.x + m[3][1] * v.y + m[3][2] * v.z + m[3][3] * v.w,
    }
}

//draw line "first convert points 3d to 2d"
fn draw_line(
    point1: Vec4,
    point2: Vec4,
    screen: &mut Vec<Vec<Vec<char>>>,
    screen_size: &Size,
    fov_degrees: f32,
    aspect: f32,
    near: f32,
    far: f32,
) {
    //convert 3d points to 2d
    let point1 = draw_3d(
        perspective(fov_degrees, aspect, near, far, point1),
        screen_size,
    );
    let point2 = draw_3d(
        perspective(fov_degrees, aspect, near, far, point2),
        screen_size,
    );
    //get 2d points
    //loop the point in Bresenham’s line algorythm
    if (point1.x - point2.x).abs() > (point1.y - point2.y).abs() {
        draw_line_h(point1, point2, screen, screen_size);
    } else {
        draw_line_v(point1, point2, screen, screen_size);
    }
    //draw the points
}
fn draw_line_v(mut p1: Vec3, mut p2: Vec3, screen: &mut Vec<Vec<Vec<char>>>, terminal: &Size) {
    if p1.y > p2.y {
        std::mem::swap(&mut p1, &mut p2);
    }

    let mut dx = p2.x - p1.x;
    let dy = p2.y - p1.y;
    let dir = if dx < 0.0 { -1.0 } else { 1.0 };
    dx *= dir;

    if dy != 0.0 {
        let mut x = p1.x;
        let mut p = 2.0 * dx - dy;

        for i in 0..=dy as i32 {
            let y = p1.y as i32 + i;

            // interpolate z along the line
            let t = if dy != 0.0 { i as f32 / dy } else { 0.0 };
            let z = p1.z + t * (p2.z - p1.z);

            if x >= 0.0 && x < terminal.columns as f32 && y >= 0 && y < terminal.lines as i32 {
                let xi = x as usize;
                let yi = y as usize;
                let zi = z as usize;

                // make sure z is in bounds
                screen[zi][yi][xi] = '.';
            }

            if p >= 0.0 {
                x += dir;
                p -= 2.0 * dy;
            }
            p += 2.0 * dx;
        }
    }
}

fn draw_line_h(mut p1: Vec3, mut p2: Vec3, screen: &mut Vec<Vec<Vec<char>>>, terminal: &Size) {
    if p1.x > p2.x {
        std::mem::swap(&mut p1, &mut p2);
    }

    let dx = p2.x - p1.x;
    let mut dy = p2.y - p1.y;
    let dir = if dy < 0.0 { -1.0 } else { 1.0 };
    dy *= dir;

    if dx != 0.0 {
        let mut y = p1.y;
        let mut p = 2.0 * dy - dx;

        for i in 0..=dx as i32 {
            let x = p1.x as i32 + i;

            // interpolate z
            let t = if dx != 0.0 { i as f32 / dx } else { 0.0 };
            let z = p1.z + t * (p2.z - p1.z);

            if x >= 0 && x < terminal.columns as i32 && y >= 0.0 && y < terminal.lines as f32 {
                let yi = y as usize;
                let xi = x as usize;
                let zi = z as usize;

                // make sure z is within bounds

                screen[zi][yi][xi] = '.';
            }

            if p >= 0.0 {
                y += dir;
                p -= 2.0 * dx;
            }
            p += 2.0 * dy;
        }
    }
}

pub fn draw_text(x: isize, y: isize, text: &str, screen: &mut Vec<Vec<Vec<char>>>, term: &Size) {
    // If vertically outside screen → ignore
    if y < 0 || y >= term.lines as isize {
        return;
    }

    let row = y as usize;

    // Draw each character
    for (i, ch) in text.chars().enumerate() {
        let sx = x + i as isize;

        // Horizontal clipping
        if sx < 0 || sx >= term.columns as isize {
            continue;
        }

        screen[0][row][sx as usize] = ch;
    }
}

// Clear the screen buffer
pub fn clear_screen_buf(screen: &mut Vec<Vec<Vec<char>>>, clear_screen: &Vec<Vec<Vec<char>>>) {
    *screen = clear_screen.clone();
}

// Display the screen buffer
pub fn display_screen(screen: &Vec<Vec<Vec<char>>>, terminal: &Size) {
    let mut buffer = String::with_capacity(terminal.lines * (terminal.columns + 1));

    for y in 0..terminal.lines as usize {
        for x in 0..terminal.columns as usize {
            let z = (0..screen.len())
                .find(|&zi| screen[zi][y][x] != ' ')
                .unwrap_or(0);
            buffer.push(screen[z][y][x]);
        }
        buffer.push('\n');
    }

    print!("\x1B[H{}", buffer);
}

pub fn key_renderer() {
    let mut world = WORLD.lock().unwrap();
    unsafe {
        if inputs::MOVEMENT.left == true {
            for line in world.iter_mut() {
                line.point1.x -= 1.0;
                line.point2.x -= 1.0;
            }
        }
        if inputs::MOVEMENT.right == true {
            for line in world.iter_mut() {
                line.point1.x += 1.0;
                line.point2.x += 1.0;
            }
        }
        if inputs::MOVEMENT.forward == true {
            for line in world.iter_mut() {
                line.point1.z -= 1.0;
                line.point2.z -= 1.0;
            }
        }
        if inputs::MOVEMENT.backward == true {
            for line in world.iter_mut() {
                line.point1.z += 1.0;
                line.point2.z += 1.0;
            }
        }
        if inputs::MOVEMENT.down == true {
            for line in world.iter_mut() {
                line.point1.y += 1.0;
                line.point2.y += 1.0;
            }
        }
        if inputs::MOVEMENT.up == true {
            for line in world.iter_mut() {
                line.point1.y -= 1.0;
                line.point2.y -= 1.0;
            }
        }
    }
}
