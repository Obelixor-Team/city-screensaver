use crossterm::{
    cursor::{self, Hide, Show},
    event::{self, Event},
    style::{self, Color, Print},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand, QueueableCommand,
};
use rand::Rng;
use std::io::{self, stdout, Write};
use std::time::Duration;

const WINDOW_ON_COLOR: Color = Color::Rgb { r: 255, g: 255, b: 0 };
const WINDOW_OFF_COLOR: Color = Color::Rgb { r: 40, g: 40, b: 40 };
const ROAD_COLOR: Color = Color::Rgb { r: 20, g: 20, b: 20 };
const MOON_COLOR: Color = Color::Rgb { r: 240, g: 240, b: 240 };
const STAR_COLOR: Color = Color::Rgb { r: 255, g: 255, b: 255 };

struct Star {
    x: u16,
    y: u16,
    char: char,
}

struct Window {
    on: bool,
}

struct Building {
    x: u16,
    width: u16,
    height: u16,
    color: Color,
    windows: Vec<Vec<Window>>,
}

struct Vehicle {
    x: f32,
    y: u16,
    style: &'static str,
    color: Color,
    speed: f32,
}

fn main() -> io::Result<()> {
    let mut stdout = stdout();
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(Hide)?;
    terminal::enable_raw_mode()?;

    let (width, height) = terminal::size()?;
    let mut buildings = create_buildings(width, height);
    let mut vehicles = create_vehicles(height);
    let mut stars = create_stars(width, height);

    let mut running = true;
    while running {
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(_) = event::read()? {
                running = false;
            }
        }

        update_windows(&mut buildings);
        update_vehicles(&mut vehicles, width);
        update_stars(&mut stars);
        draw_scene(&mut stdout, &buildings, &vehicles, &stars, width, height)?;
    }

    terminal::disable_raw_mode()?;
    stdout.execute(Show)?;
    stdout.execute(LeaveAlternateScreen)?;
    Ok(())
}

fn create_buildings(term_width: u16, term_height: u16) -> Vec<Building> {
    let mut buildings = Vec::new();
    let mut x = 0;
    let mut rng = rand::thread_rng();
    let building_colors = [
        Color::Rgb { r: 60, g: 60, b: 60 },
        Color::Rgb { r: 70, g: 70, b: 70 },
        Color::Rgb { r: 80, g: 80, b: 80 },
        Color::Rgb { r: 90, g: 90, b: 90 },
    ];

    while x < term_width {
        let width = rng.gen_range(5..15);
        let height = rng.gen_range(5..(term_height - 5));
        let color = building_colors[rng.gen_range(0..building_colors.len())];
        let mut windows = Vec::new();

        for y in 1..height-1 {
            let mut row = Vec::new();
            for wx in 1..width-1 {
                if (y % 2 != 0) && (wx % 2 != 0) {
                    row.push(Window { on: rng.gen_bool(0.3) });
                }
            }
            windows.push(row);
        }

        buildings.push(Building { x, width, height, color, windows });
        x += width + rng.gen_range(1..5);
    }
    buildings
}

fn create_vehicles(term_height: u16) -> Vec<Vehicle> {
    let road_y = term_height - 3;
    vec![
        Vehicle { x: 10.0, y: road_y, style: "─=≡(°o°)", color: Color::Yellow, speed: 5.0 },
        Vehicle { x: 30.0, y: road_y - 1, style: "[\\__\\]", color: Color::Green, speed: -3.0 },
        Vehicle { x: 50.0, y: road_y, style: "o-o-o", color: Color::Cyan, speed: 4.0 },
        Vehicle { x: 70.0, y: road_y - 1, style: "[##-##]", color: Color::Magenta, speed: -2.5 },
        Vehicle { x: 20.0, y: road_y, style: "<(o.o)>", color: Color::Red, speed: 2.0 },
        Vehicle { x: 60.0, y: road_y - 1, style: "🚚", color: Color::Blue, speed: -2.0 },
    ]
}

fn create_stars(term_width: u16, term_height: u16) -> Vec<Star> {
    let mut stars = Vec::new();
    let mut rng = rand::thread_rng();
    for _ in 0..50 {
        stars.push(Star {
            x: rng.gen_range(0..term_width),
            y: rng.gen_range(0..term_height / 2),
            char: ['.', '*', '+', '\''].get(rng.gen_range(0..4)).unwrap().clone(),
        });
    }
    stars
}

fn update_windows(buildings: &mut [Building]) {
    let mut rng = rand::thread_rng();
    for building in buildings {
        for row in &mut building.windows {
            for window in row {
                if rng.gen_bool(0.01) {
                    window.on = !window.on;
                }
            }
        }
    }
}

fn update_vehicles(vehicles: &mut [Vehicle], term_width: u16) {
    for vehicle in vehicles {
        vehicle.x += vehicle.speed * 0.1;
        if vehicle.x < 0.0 {
            vehicle.x = term_width as f32;
        } else if vehicle.x > term_width as f32 {
            vehicle.x = 0.0;
        }
    }
}

fn update_stars(stars: &mut [Star]) {
    let mut rng = rand::thread_rng();
    for star in stars {
        if rng.gen_bool(0.05) {
            star.char = ['.', '*', '+', '\''].get(rng.gen_range(0..4)).unwrap().clone();
        }
    }
}

fn draw_scene(
    stdout: &mut io::Stdout,
    buildings: &[Building],
    vehicles: &[Vehicle],
    stars: &[Star],
    term_width: u16,
    term_height: u16,
) -> io::Result<()> {
    stdout.queue(Clear(ClearType::All))?;

    // Draw stars
    for star in stars {
        stdout
            .queue(cursor::MoveTo(star.x, star.y))?
            .queue(style::SetForegroundColor(STAR_COLOR))?
            .queue(Print(star.char))?;
    }

    // Draw moon
    stdout
        .queue(cursor::MoveTo(term_width - 15, 1))?
        .queue(style::SetForegroundColor(MOON_COLOR))?
        .queue(Print("  ,'.'."))?
        .queue(cursor::MoveTo(term_width - 15, 2))?
        .queue(Print(" ,'. ..'."))?
        .queue(cursor::MoveTo(term_width - 15, 3))?
        .queue(Print(".' .. '. '."))?;


    // Draw buildings
    for building in buildings {
        for y in 0..building.height {
            for x in 0..building.width {
                stdout
                    .queue(cursor::MoveTo(building.x + x, term_height - building.height - 3 + y))?
                    .queue(style::SetForegroundColor(building.color))?
                    .queue(Print("█"))?;
            }
        }

        for (wy, row) in building.windows.iter().enumerate() {
            for (wx, window) in row.iter().enumerate() {
                let color = if window.on { WINDOW_ON_COLOR } else { WINDOW_OFF_COLOR };
                stdout
                    .queue(cursor::MoveTo(building.x + (wx as u16 * 2) + 1, term_height - building.height - 2 + (wy as u16 * 2)))?
                    .queue(style::SetForegroundColor(color))?
                    .queue(Print("■"))?;
            }
        }
    }

    // Draw road
    let road_y = term_height - 3;
    stdout.queue(cursor::MoveTo(0, road_y))?;
    stdout.queue(style::SetForegroundColor(ROAD_COLOR))?;
    for _ in 0..term_width {
        stdout.queue(Print("="))?;
    }
    stdout.queue(cursor::MoveTo(0, road_y+1))?;
    for _ in 0..term_width {
        stdout.queue(Print("="))?;
    }


    // Draw vehicles
    for vehicle in vehicles {
        stdout
            .queue(cursor::MoveTo(vehicle.x as u16, vehicle.y))?
            .queue(style::SetForegroundColor(vehicle.color))?
            .queue(Print(vehicle.style))?;
    }

    stdout.flush()
}
