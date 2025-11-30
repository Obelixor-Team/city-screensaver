//! A city-themed terminal screensaver with animated buildings, vehicles, and weather effects.
//!
//! This application creates an animated city scene with moving vehicles, animated building windows,
//! and configurable weather effects displayed as a screensaver in the terminal.

use clap::Parser;
use crossterm::{
    cursor::{self, Hide, Show},
    event::{self, Event},
    style::{self, Color, Print},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand, QueueableCommand,
};
use rand::{rngs::ThreadRng, Rng};
use std::io::{self, stdout, Write};
use std::time::{Duration, Instant};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
/// Command-line arguments for configuring the city screensaver
struct Args {
    /// Number of stars to display
    #[arg(long, default_value_t = 50)]
    stars: u16,

    /// Number of raindrops to display
    #[arg(long, default_value_t = 100)]
    raindrops: u16,

    /// Number of snowflakes to display
    #[arg(long, default_value_t = 50)]
    snowflakes: u16,

    /// Number of clouds to display
    #[arg(long, default_value_t = 5)]
    clouds: u16,

    /// Update interval in milliseconds
    #[arg(long, default_value_t = 50)]
    interval: u64,

    /// Enable rain effect
    #[arg(long, default_value_t = true)]
    rain: bool,

    /// Enable snow effect
    #[arg(long, default_value_t = false)]
    snow: bool,
}

/// Color constants for different elements in the city scene
const WINDOW_ON_COLOR: Color = Color::Rgb { r: 255, g: 255, b: 0 };
const WINDOW_OFF_COLOR: Color = Color::Rgb { r: 40, g: 40, b: 40 };
const ROAD_COLOR: Color = Color::Rgb { r: 20, g: 20, b: 20 };
const MOON_COLOR: Color = Color::Rgb { r: 240, g: 240, b: 240 };
const STAR_COLOR: Color = Color::Rgb { r: 255, g: 255, b: 255 };
const RAIN_COLOR: Color = Color::Rgb { r: 100, g: 100, b: 150 };
const SNOW_COLOR: Color = Color::Rgb { r: 200, g: 200, b: 200 };
const CLOUD_COLOR: Color = Color::Rgb { r: 150, g: 150, b: 150 };

const STAR_CHARS: [char; 4] = ['.', '*', '+', '\''];
const SNOWFLAKE_CHARS: [char; 3] = ['*', '.', 'o'];
const CLOUD_SHAPES: [&str; 3] = ["_.-^-._", " ~~~", "(-.-)"];
const ANTENNA_CHARS: [char; 3] = ['|', 'Y', 'i'];
const BUILDING_COLORS: [Color; 4] = [
    Color::Rgb { r: 60, g: 60, b: 60 },
    Color::Rgb { r: 70, g: 70, b: 70 },
    Color::Rgb { r: 80, g: 80, b: 80 },
    Color::Rgb { r: 90, g: 90, b: 90 },
];
const VEHICLE_STYLES: [(&str, Color, f32); 9] = [
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

/// Represents a star in the night sky
struct Star {
    x: u16,
    y: u16,
    char: char,
}

/// Represents a raindrop falling down the screen
struct RainDrop {
    x: u16,
    y: u16,
    speed: u16,
}

/// Represents a snowflake falling with horizontal drift
struct Snowflake {
    x: u16,
    y: u16,
    speed_y: u16,
    speed_x: i8, // For horizontal drift
    char: char,
}

/// Represents a cloud moving across the sky
struct Cloud {
    x: f32,
    y: u16,
    shape: &'static str,
    speed: f32,
}

/// Represents a window in a building that can be on or off
struct Window {
    on: bool,
}

/// Represents a building with windows and optional antenna
struct Building {
    x: u16,
    width: u16,
    height: u16,
    color: Color,
    windows: Vec<Vec<Window>>,
    has_antenna: bool,
    antenna_char: char,
}

/// Represents a vehicle moving along the road
struct Vehicle {
    x: f32,
    y: u16,
    style: &'static str,
    color: Color,
    speed: f32,
}

/// Sets up the terminal for the screensaver by enabling raw mode and switching to alternate screen
fn setup_terminal() -> io::Result<std::io::Stdout> {
    let mut stdout = stdout();
    stdout.execute(EnterAlternateScreen)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to enter alternate screen: {}", e)))?;
    stdout.execute(Hide)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to hide cursor: {}", e)))?;
    terminal::enable_raw_mode()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to enable raw mode: {}", e)))?;
    Ok(stdout)
}

/// Restores the terminal to its original state after the screensaver exits
fn restore_terminal(stdout: &mut std::io::Stdout) -> io::Result<()> {
    terminal::disable_raw_mode()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to disable raw mode: {}", e)))?;
    stdout.execute(Show)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to show cursor: {}", e)))?;
    stdout.execute(LeaveAlternateScreen)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to leave alternate screen: {}", e)))?;
    Ok(())
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    let mut stdout = setup_terminal()?;

    // Ensure terminal is restored on panic or exit
    let (width, height) = terminal::size()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to get terminal size: {}", e)))?;
    let mut rng = ThreadRng::default();
    let mut buildings = create_buildings(width, height, &mut rng);
    let mut vehicles = create_vehicles(height);
    let mut stars = create_stars_with_count(width, height, &mut rng, args.stars);
    let mut raindrops = if args.rain {
        create_raindrops_with_count(width, height, &mut rng, args.raindrops)
    } else {
        Vec::new()
    };
    let mut snowflakes = if args.snow {
        create_snowflakes_with_count(width, height, &mut rng, args.snowflakes)
    } else {
        Vec::new()
    };
    let mut clouds = create_clouds_with_count(width, height, &mut rng, args.clouds);

    // FPS tracking
    let mut frame_count = 0;
    let mut last_fps_update = Instant::now();
    let mut fps = 0.0;

    let result = (|| -> io::Result<()> {
        let mut running = true;
        while running {
            let frame_start = Instant::now();

            if event::poll(Duration::from_millis(args.interval))? {
                if let Event::Key(_) = event::read()? {
                    running = false;
                }
            }

            if rng.random_bool(0.1) {
                vehicles.push(spawn_vehicle(width, height, &mut rng));
            }

            update_windows(&mut buildings, &mut rng);
            update_vehicles(&mut vehicles, width);
            update_stars(&mut stars, &mut rng);
            if args.rain {
                update_raindrops(&mut raindrops, width, height, &mut rng);
            }
            if args.snow {
                update_snowflakes(&mut snowflakes, width, height, &mut rng);
            }
            update_clouds(&mut clouds, width);

            // Calculate and display FPS
            frame_count += 1;
            let elapsed = last_fps_update.elapsed();
            if elapsed.as_secs() >= 1 {
                fps = frame_count as f64 / elapsed.as_secs_f64();
                frame_count = 0;
                last_fps_update = Instant::now();

                // Display FPS for debugging (only when not in terminal)
                // In a terminal screensaver, we typically don't show FPS overlay
            }

            draw_scene(&mut stdout, &buildings, &vehicles, &stars, &raindrops, &snowflakes, &clouds, width, height, args.snow)?;

            // Calculate frame time for FPS display purposes
            let frame_time = frame_start.elapsed();
            let target_frame_time = Duration::from_millis(args.interval);
            if frame_time < target_frame_time {
                std::thread::sleep(target_frame_time - frame_time);
            }
        }
        Ok(())
    })();

    // Always restore terminal
    if let Err(e) = restore_terminal(&mut stdout) {
        eprintln!("Error restoring terminal: {}", e);
    }

    result
}



fn create_buildings(term_width: u16, term_height: u16, rng: &mut ThreadRng) -> Vec<Building> {
    let mut buildings = Vec::new();
    let mut x = 0;

    while x < term_width {
        let width = rng.random_range(5..15);
        let height = rng.random_range(5..(term_height - 5));
        let color = BUILDING_COLORS[rng.random_range(0..BUILDING_COLORS.len())];
        let mut windows = Vec::new();

        for y in 1..height-1 {
            let mut row = Vec::new();
            for wx in 1..width-1 {
                if (y % 2 != 0) && (wx % 2 != 0) {
                    row.push(Window { on: rng.random_bool(0.3) });
                }
            }
            windows.push(row);
        }

        let has_antenna = rng.random_bool(0.3);
        let antenna_char = if has_antenna {
            ANTENNA_CHARS[rng.random_range(0..ANTENNA_CHARS.len())]
        } else {
            ' '
        };

        buildings.push(Building { x, width, height, color, windows, has_antenna, antenna_char });
        x += width + rng.random_range(1..5);
    }
    buildings
}

fn create_vehicles(_term_height: u16) -> Vec<Vehicle> {
    Vec::new()
}

fn spawn_vehicle(term_width: u16, term_height: u16, rng: &mut ThreadRng) -> Vehicle {
    let road_y = term_height - 3;

    let (style, color, speed) = VEHICLE_STYLES[rng.random_range(0..VEHICLE_STYLES.len())];
    let y = if rng.random_bool(0.5) { road_y } else { road_y - 1 };
    let x = if speed > 0.0 { 0.0 } else { term_width as f32 };

    Vehicle { x, y, style, color, speed }
}

/// Creates a specified number of stars with random positions and characters
fn create_stars_with_count(term_width: u16, term_height: u16, rng: &mut ThreadRng, count: u16) -> Vec<Star> {
    let mut stars = Vec::new();
    for _ in 0..count {
        stars.push(Star {
            x: rng.random_range(0..term_width),
            y: rng.random_range(0..term_height / 2),
            char: STAR_CHARS[rng.random_range(0..STAR_CHARS.len())],
        });
    }
    stars
}

/// Creates 50 stars with random positions and characters
fn create_stars(term_width: u16, term_height: u16, rng: &mut ThreadRng) -> Vec<Star> {
    create_stars_with_count(term_width, term_height, rng, 50)  // Default to 50 for backward compatibility
}

fn create_raindrops_with_count(term_width: u16, term_height: u16, rng: &mut ThreadRng, count: u16) -> Vec<RainDrop> {
    let mut raindrops = Vec::new();
    for _ in 0..count {
        raindrops.push(RainDrop {
            x: rng.random_range(0..term_width),
            y: rng.random_range(0..term_height),
            speed: rng.random_range(1..3),
        });
    }
    raindrops
}

fn create_raindrops(term_width: u16, term_height: u16, rng: &mut ThreadRng) -> Vec<RainDrop> {
    create_raindrops_with_count(term_width, term_height, rng, 100)  // Default to 100 for backward compatibility
}

/// Updates the state of windows in all buildings, randomly toggling them on/off
fn update_windows(buildings: &mut [Building], rng: &mut ThreadRng) {
    for building in buildings {
        for row in &mut building.windows {
            for window in row {
                if rng.random_bool(0.01) {
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
    for star in stars {
        if rng.random_bool(0.05) {
            star.char = STAR_CHARS[rng.random_range(0..STAR_CHARS.len())];
        }
    }
}

fn update_raindrops(raindrops: &mut [RainDrop], term_width: u16, term_height: u16, rng: &mut ThreadRng) {
    for drop in raindrops {
        drop.y += drop.speed;
        if drop.y >= term_height {
            drop.y = 0;
            drop.x = rng.random_range(0..term_width);
        }
    }
}

fn create_snowflakes_with_count(term_width: u16, term_height: u16, rng: &mut ThreadRng, count: u16) -> Vec<Snowflake> {
    let mut snowflakes = Vec::new();
    for _ in 0..count {
        snowflakes.push(Snowflake {
            x: rng.random_range(0..term_width),
            y: rng.random_range(0..term_height),
            speed_y: rng.random_range(1..2),
            speed_x: rng.random_range(-1..2),
            char: SNOWFLAKE_CHARS[rng.random_range(0..SNOWFLAKE_CHARS.len())],
        });
    }
    snowflakes
}

fn create_snowflakes(term_width: u16, term_height: u16, rng: &mut ThreadRng) -> Vec<Snowflake> {
    create_snowflakes_with_count(term_width, term_height, rng, 50)  // Default to 50 for backward compatibility
}

fn update_snowflakes(snowflakes: &mut [Snowflake], term_width: u16, term_height: u16, rng: &mut ThreadRng) {
    for flake in snowflakes {
        flake.y += flake.speed_y;
        if flake.y >= term_height {
            flake.y = 0;
            flake.x = rng.random_range(0..term_width);
        }

        flake.x = (flake.x as i16 + flake.speed_x as i16) as u16;
        if flake.x >= term_width {
            flake.x = 0;
        } else if flake.x == 0 && flake.speed_x < 0 {
            flake.x = term_width - 1;
        }
    }
}

fn create_clouds_with_count(term_width: u16, term_height: u16, rng: &mut ThreadRng, count: u16) -> Vec<Cloud> {
    let mut clouds = Vec::new();
    for _ in 0..count { // Create count clouds
        clouds.push(Cloud {
            x: rng.random_range(0..term_width) as f32,
            y: rng.random_range(0..term_height / 4), // Upper quarter of the screen
            shape: CLOUD_SHAPES[rng.random_range(0..CLOUD_SHAPES.len())],
            speed: rng.random_range(0.5..1.5),
        });
    }
    clouds
}

fn create_clouds(term_width: u16, term_height: u16, rng: &mut ThreadRng) -> Vec<Cloud> {
    create_clouds_with_count(term_width, term_height, rng, 5)  // Default to 5 for backward compatibility
}

fn update_clouds(clouds: &mut [Cloud], term_width: u16) {
    for cloud in clouds {
        cloud.x += cloud.speed * 0.1;
        if cloud.x > term_width as f32 {
            cloud.x = -(cloud.shape.len() as f32); // Wrap around
        }
    }
}

/// Draws the entire scene by calling individual drawing functions
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
    is_snow: bool,
) -> io::Result<()> {
    stdout.queue(Clear(ClearType::All))?;

    // Draw background elements first
    draw_clouds(stdout, clouds)?;
    draw_stars(stdout, stars)?;
    draw_moon(stdout, term_width)?;
    draw_buildings(stdout, buildings, term_height)?;
    draw_road(stdout, term_width, term_height)?;
    draw_weather_effects(stdout, raindrops, snowflakes, is_snow)?;
    draw_vehicles(stdout, vehicles)?;

    stdout.flush()
}

/// Draws all clouds in the scene
fn draw_clouds(stdout: &mut io::Stdout, clouds: &[Cloud]) -> io::Result<()> {
    for cloud in clouds {
        stdout
            .queue(cursor::MoveTo(cloud.x as u16, cloud.y))?
            .queue(style::SetForegroundColor(CLOUD_COLOR))?
            .queue(Print(cloud.shape))?;
    }
    Ok(())
}

/// Draws all stars in the scene
fn draw_stars(stdout: &mut io::Stdout, stars: &[Star]) -> io::Result<()> {
    for star in stars {
        stdout
            .queue(cursor::MoveTo(star.x, star.y))?
            .queue(style::SetForegroundColor(STAR_COLOR))?
            .queue(Print(star.char))?;
    }
    Ok(())
}

/// Draws the moon in the scene
fn draw_moon(stdout: &mut io::Stdout, term_width: u16) -> io::Result<()> {
    stdout
        .queue(cursor::MoveTo(term_width - 15, 1))?
        .queue(style::SetForegroundColor(MOON_COLOR))?
        .queue(Print("  ,'.'."))?
        .queue(cursor::MoveTo(term_width - 15, 2))?
        .queue(Print(" ,'. ..'."))?
        .queue(cursor::MoveTo(term_width - 15, 3))?
        .queue(Print(".' .. '. '."))?;
    Ok(())
}

/// Draws all buildings in the scene
fn draw_buildings(stdout: &mut io::Stdout, buildings: &[Building], term_height: u16) -> io::Result<()> {
    for building in buildings {
        // Draw building structure
        for y in 0..building.height {
            for x in 0..building.width {
                stdout
                    .queue(cursor::MoveTo(building.x + x, term_height - building.height - 3 + y))?
                    .queue(style::SetForegroundColor(building.color))?
                    .queue(Print("█"))?;
            }
        }

        // Draw antenna if present
        if building.has_antenna {
            stdout
                .queue(cursor::MoveTo(building.x + building.width / 2, term_height - building.height - 4))?
                .queue(style::SetForegroundColor(building.color))?
                .queue(Print(building.antenna_char))?;
        }

        // Draw windows
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
    Ok(())
}

/// Draws the road at the bottom of the scene
fn draw_road(stdout: &mut io::Stdout, term_width: u16, term_height: u16) -> io::Result<()> {
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
    Ok(())
}

/// Draws weather effects (either rain or snow based on the is_snow flag)
fn draw_weather_effects(
    stdout: &mut io::Stdout,
    raindrops: &[RainDrop],
    snowflakes: &[Snowflake],
    is_snow: bool,
) -> io::Result<()> {
    if is_snow {
        // Draw snowflakes
        for flake in snowflakes {
            stdout
                .queue(cursor::MoveTo(flake.x, flake.y))?
                .queue(style::SetForegroundColor(SNOW_COLOR))?
                .queue(Print(flake.char))?;
        }
    } else {
        // Draw raindrops
        for drop in raindrops {
            stdout
                .queue(cursor::MoveTo(drop.x, drop.y))?
                .queue(style::SetForegroundColor(RAIN_COLOR))?
                .queue(Print("|"))?;
        }
    }
    Ok(())
}

/// Draws all vehicles in the scene
fn draw_vehicles(stdout: &mut io::Stdout, vehicles: &[Vehicle]) -> io::Result<()> {
    for vehicle in vehicles {
        stdout
            .queue(cursor::MoveTo(vehicle.x as u16, vehicle.y))?
            .queue(style::SetForegroundColor(vehicle.color))?
            .queue(Print(vehicle.style))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::ThreadRng;

    /// Test that create_stars_with_count creates the correct number of stars
    #[test]
    fn test_create_stars_with_count() {
        let mut rng = ThreadRng::default();
        let stars = create_stars_with_count(80, 24, &mut rng, 10);
        assert_eq!(stars.len(), 10);

        // Verify all stars are within the specified bounds
        for star in &stars {
            assert!(star.x < 80);
            assert!(star.y < 24 / 2); // Stars only in the top half of the screen
            assert!(STAR_CHARS.contains(&star.char));
        }
    }

    /// Test that create_buildings creates buildings with valid properties
    #[test]
    fn test_create_buildings() {
        let mut rng = ThreadRng::default();
        let buildings = create_buildings(80, 24, &mut rng);

        for building in &buildings {
            assert!(building.width >= 5 && building.width < 15);
            assert!(building.height >= 5 && building.height < 24 - 5);
            assert!(building.x < 80);
            assert!(BUILDING_COLORS.contains(&building.color));
        }
    }

    /// Test that spawn_vehicle creates valid vehicles
    #[test]
    fn test_spawn_vehicle() {
        let mut rng = ThreadRng::default();
        let vehicle = spawn_vehicle(80, 24, &mut rng);

        // Check that the vehicle properties are from our valid set
        let valid_styles: Vec<&str> = VEHICLE_STYLES.iter().map(|(style, _, _)| *style).collect();
        assert!(valid_styles.contains(&vehicle.style));

        let valid_colors: Vec<Color> = VEHICLE_STYLES.iter().map(|(_, color, _)| *color).collect();
        assert!(valid_colors.contains(&vehicle.color));

        let valid_speeds: Vec<f32> = VEHICLE_STYLES.iter().map(|(_, _, speed)| *speed).collect();
        assert!(valid_speeds.contains(&vehicle.speed));
    }

    /// Test that vehicles spawn with appropriate y positions
    #[test]
    fn test_spawn_vehicle_y_position() {
        let mut rng = ThreadRng::default();
        let road_y = 24 - 3; // term_height - 3
        let vehicle1 = spawn_vehicle(80, 24, &mut rng);
        let vehicle2 = spawn_vehicle(80, 24, &mut rng);

        // Vehicle y position should be either road_y or road_y - 1
        assert!(vehicle1.y == road_y || vehicle1.y == road_y - 1);
        assert!(vehicle2.y == road_y || vehicle2.y == road_y - 1);
    }

    /// Test that building windows are created with the right pattern
    #[test]
    fn test_building_windows_pattern() {
        let mut rng = ThreadRng::default();
        let buildings = create_buildings(80, 24, &mut rng);

        // Verify buildings have windows created
        for building in &buildings {
            assert!(!building.windows.is_empty());
        }
    }
}
