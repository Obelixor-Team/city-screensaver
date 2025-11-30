use crossterm::{
    cursor::{self, Hide, Show},
    event::{self, Event},
    style::{self, Color, Print},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand, QueueableCommand,
};
use rand::{rngs::ThreadRng, Rng};
use std::io::{self, stdout, Write};
use std::time::Duration;

const WINDOW_ON_COLOR: Color = Color::Rgb { r: 255, g: 255, b: 0 };
const WINDOW_OFF_COLOR: Color = Color::Rgb { r: 40, g: 40, b: 40 };
const ROAD_COLOR: Color = Color::Rgb { r: 20, g: 20, b: 20 };
const MOON_COLOR: Color = Color::Rgb { r: 240, g: 240, b: 240 };
const STAR_COLOR: Color = Color::Rgb { r: 255, g: 255, b: 255 };
const RAIN_COLOR: Color = Color::Rgb { r: 100, g: 100, b: 150 };
const SNOW_COLOR: Color = Color::Rgb { r: 200, g: 200, b: 200 };
const CLOUD_COLOR: Color = Color::Rgb { r: 150, g: 150, b: 150 };

struct Star {
    x: u16,
    y: u16,
    char: char,
}

struct RainDrop {
    x: u16,
    y: u16,
    speed: u16,
}

struct Snowflake {
    x: u16,
    y: u16,
    speed_y: u16,
    speed_x: i8, // For horizontal drift
    char: char,
}

struct Cloud {
    x: f32,
    y: u16,
    shape: &'static str,
    speed: f32,
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
    has_antenna: bool,
    antenna_char: char,
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
    let mut rng = rand::rngs::ThreadRng::default();
    let mut buildings = create_buildings(width, height, &mut rng);
    let mut vehicles = create_vehicles(height);
    let mut stars = create_stars(width, height, &mut rng);
    let mut raindrops = create_raindrops(width, height, &mut rng);
    let mut snowflakes = create_snowflakes(width, height, &mut rng);
    let mut clouds = create_clouds(width, height, &mut rng);


    let mut running = true;
    while running {
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(_) = event::read()? {
                running = false;
            }
        }

        if rng.gen_bool(0.1) {
            vehicles.push(spawn_vehicle(width, height, &mut rng));
        }

        update_windows(&mut buildings, &mut rng);
        update_vehicles(&mut vehicles, width);
        update_stars(&mut stars, &mut rng);
        update_raindrops(&mut raindrops, width, height, &mut rng);
        update_snowflakes(&mut snowflakes, width, height, &mut rng);
        update_clouds(&mut clouds, width);
        draw_scene(&mut stdout, &buildings, &vehicles, &stars, &raindrops, &snowflakes, &clouds, width, height)?;
    }

    terminal::disable_raw_mode()?;
    stdout.execute(Show)?;
    stdout.execute(LeaveAlternateScreen)?;
    Ok(())
}



fn create_buildings(term_width: u16, term_height: u16, rng: &mut ThreadRng) -> Vec<Building> {
    let mut buildings = Vec::new();
    let mut x = 0;
    let building_colors = [
        Color::Rgb { r: 60, g: 60, b: 60 },
        Color::Rgb { r: 70, g: 70, b: 70 },
        Color::Rgb { r: 80, g: 80, b: 80 },
        Color::Rgb { r: 90, g: 90, b: 90 },
    ];
    let antenna_chars = ['|', 'Y', 'i'];

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

        let has_antenna = rng.gen_bool(0.3);
        let antenna_char = if has_antenna {
            antenna_chars[rng.gen_range(0..antenna_chars.len())]
        } else {
            ' '
        };

        buildings.push(Building { x, width, height, color, windows, has_antenna, antenna_char });
        x += width + rng.gen_range(1..5);
    }
    buildings
}

fn create_vehicles(term_height: u16) -> Vec<Vehicle> {
    Vec::new()
}

fn spawn_vehicle(term_width: u16, term_height: u16, rng: &mut ThreadRng) -> Vehicle {
    let road_y = term_height - 3;
    let vehicle_styles = [
        ("─=≡(°o°)", Color::Yellow, 5.0),
        ("[\\__\\_]", Color::Green, -3.0),
        ("o-o-o", Color::Cyan, 4.0),
        ("[##-##]", Color::Magenta, -2.5),
        ("<(o.o)>", Color::Red, 2.0),
        ("🚚", Color::Blue, -2.0),
        ("🚓", Color::White, 3.5),
        ("🚑", Color::Red, -4.0),
        ("🚌", Color::Green, 2.8),
    ];

    let (style, color, speed) = vehicle_styles[rng.gen_range(0..vehicle_styles.len())];
    let y = if rng.gen_bool(0.5) { road_y } else { road_y - 1 };
    let x = if speed > 0.0 { 0.0 } else { term_width as f32 };

    Vehicle { x, y, style, color, speed }
}

fn create_stars(term_width: u16, term_height: u16, rng: &mut ThreadRng) -> Vec<Star> {
    let mut stars = Vec::new();
    let star_chars = ['.', '*', '+', '\''];
    for _ in 0..50 {
        stars.push(Star {
            x: rng.gen_range(0..term_width),
            y: rng.gen_range(0..term_height / 2),
            char: star_chars[rng.gen_range(0..star_chars.len())],
        });
    }
    stars
}

fn create_raindrops(term_width: u16, term_height: u16, rng: &mut ThreadRng) -> Vec<RainDrop> {
    let mut raindrops = Vec::new();
    for _ in 0..100 {
        raindrops.push(RainDrop {
            x: rng.gen_range(0..term_width),
            y: rng.gen_range(0..term_height),
            speed: rng.gen_range(1..3),
        });
    }
    raindrops
}

fn update_windows(buildings: &mut [Building], rng: &mut ThreadRng) {
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

fn update_vehicles(vehicles: &mut Vec<Vehicle>, term_width: u16) {
    let mut i = 0;
    while i < vehicles.len() {
        vehicles[i].x += vehicles[i].speed * 0.1;
        
        let vehicle_width = vehicles[i].style.len() as f32; // Assuming ASCII chars have width 1

        // Remove vehicle if it's off-screen
        if (vehicles[i].speed > 0.0 && vehicles[i].x > term_width as f32) || 
           (vehicles[i].speed < 0.0 && vehicles[i].x < -vehicle_width) {
            vehicles.remove(i);
        } else {
            i += 1;
        }
    }
}

fn update_stars(stars: &mut [Star], rng: &mut ThreadRng) {
    let star_chars = ['.', '*', '+', '\''];
    for star in stars {
        if rng.gen_bool(0.05) {
            star.char = star_chars[rng.gen_range(0..star_chars.len())];
        }
    }
}

fn update_raindrops(raindrops: &mut [RainDrop], term_width: u16, term_height: u16, rng: &mut ThreadRng) {
    for drop in raindrops {
        drop.y += drop.speed;
        if drop.y >= term_height {
            drop.y = 0;
            drop.x = rng.gen_range(0..term_width);
        }
    }
}

fn create_snowflakes(term_width: u16, term_height: u16, rng: &mut ThreadRng) -> Vec<Snowflake> {
    let mut snowflakes = Vec::new();
    let snowflake_chars = ['*', '.', 'o'];
    for _ in 0..50 {
        snowflakes.push(Snowflake {
            x: rng.gen_range(0..term_width),
            y: rng.gen_range(0..term_height),
            speed_y: rng.gen_range(1..2),
            speed_x: rng.gen_range(-1..2),
            char: snowflake_chars[rng.gen_range(0..snowflake_chars.len())],
        });
    }
    snowflakes
}

fn update_snowflakes(snowflakes: &mut [Snowflake], term_width: u16, term_height: u16, rng: &mut ThreadRng) {
    for flake in snowflakes {
        flake.y += flake.speed_y;
        if flake.y >= term_height {
            flake.y = 0;
            flake.x = rng.gen_range(0..term_width);
        }

        flake.x = (flake.x as i16 + flake.speed_x as i16) as u16;
        if flake.x >= term_width {
            flake.x = 0;
        } else if flake.x == 0 && flake.speed_x < 0 {
            flake.x = term_width - 1;
        }
    }
}

fn create_clouds(term_width: u16, term_height: u16, rng: &mut ThreadRng) -> Vec<Cloud> {
    let mut clouds = Vec::new();
    let cloud_shapes = ["_.-^-._", " ~~~", "(-.-)"];
    for _ in 0..5 { // Create a few clouds
        clouds.push(Cloud {
            x: rng.gen_range(0..term_width) as f32,
            y: rng.gen_range(0..term_height / 4), // Upper quarter of the screen
            shape: cloud_shapes[rng.gen_range(0..cloud_shapes.len())],
            speed: rng.gen_range(0.5..1.5),
        });
    }
    clouds
}

fn update_clouds(clouds: &mut [Cloud], term_width: u16) {
    for cloud in clouds {
        cloud.x += cloud.speed * 0.1;
        if cloud.x > term_width as f32 {
            cloud.x = -(cloud.shape.len() as f32); // Wrap around
        }
    }
}

fn draw_scene(
    stdout: &mut io::Stdout,
    buildings: &[Building],
    vehicles: &[Vehicle],
    stars: &[Star],
    raindrops: &[RainDrop],
    snowflakes: &[Snowflake],
    clouds: &[Cloud],
    term_width: u16,
    term_height: u16,
) -> io::Result<()> {
    stdout.queue(Clear(ClearType::All))?;

    // Draw clouds
    for cloud in clouds {
        stdout
            .queue(cursor::MoveTo(cloud.x as u16, cloud.y))?
            .queue(style::SetForegroundColor(CLOUD_COLOR))?
            .queue(Print(cloud.shape))?;
    }

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

        if building.has_antenna {
            stdout
                .queue(cursor::MoveTo(building.x + building.width / 2, term_height - building.height - 4))?
                .queue(style::SetForegroundColor(building.color))?
                .queue(Print(building.antenna_char))?;
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

    // Draw raindrops
    for drop in raindrops {
        stdout
            .queue(cursor::MoveTo(drop.x, drop.y))?
            .queue(style::SetForegroundColor(RAIN_COLOR))?
            .queue(Print("|"))?;
    }

    // Draw snowflakes
    for flake in snowflakes {
        stdout
            .queue(cursor::MoveTo(flake.x, flake.y))?
            .queue(style::SetForegroundColor(SNOW_COLOR))?
            .queue(Print(flake.char))?;
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
