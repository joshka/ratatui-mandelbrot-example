use std::iter::zip;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use num_complex::Complex;
use ratatui::{buffer::Buffer, layout::Rect, style::Color, widgets::Widget};

use ratatui::{DefaultTerminal, Frame};

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new().run(terminal);
    ratatui::restore();
    result
}

struct App {
    mandelbrot: Mandelbrot,
    exit: bool,
}

impl App {
    fn new() -> App {
        App {
            mandelbrot: Mandelbrot::new(10000, -2.0, 1.0, -1.0, 1.0),
            exit: false,
        }
    }

    fn run(&mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.render(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn render(&self, frame: &mut Frame) {
        frame.render_widget(&self.mandelbrot, frame.area());
    }

    fn handle_events(&mut self) -> color_eyre::Result<()> {
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                return Ok(());
            }
            let mandelbrot = &mut self.mandelbrot;
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => self.exit = true,
                KeyCode::Char('+') => mandelbrot.increase_max_iterations(),
                KeyCode::Char('-') => mandelbrot.decrease_max_iterations(),
                KeyCode::Char('k') | KeyCode::Up => mandelbrot.pan_up(),
                KeyCode::Char('j') | KeyCode::Down => mandelbrot.pan_down(),
                KeyCode::Char('h') | KeyCode::Left => mandelbrot.pan_left(),
                KeyCode::Char('l') | KeyCode::Right => mandelbrot.pan_right(),
                KeyCode::Char('z') | KeyCode::PageUp => mandelbrot.zoom_in(),
                KeyCode::Char('x') | KeyCode::PageDown => mandelbrot.zoom_out(),
                _ => {}
            }
        }
        Ok(())
    }
}

struct Mandelbrot {
    max_iterations: u16,
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
}

impl Mandelbrot {
    fn new(max_iterations: u16, x_min: f64, x_max: f64, y_min: f64, y_max: f64) -> Mandelbrot {
        Mandelbrot {
            max_iterations,
            x_min,
            x_max,
            y_min,
            y_max,
        }
    }

    fn increase_max_iterations(&mut self) {
        self.max_iterations += 100;
    }

    fn decrease_max_iterations(&mut self) {
        self.max_iterations = self.max_iterations.checked_sub(100).unwrap_or(100)
    }

    fn pan_left(&mut self) {
        let pan = (self.x_max - self.x_min) * 0.1;
        self.x_min -= pan;
        self.x_max -= pan;
    }

    fn pan_right(&mut self) {
        let pan = (self.x_max - self.x_min) * 0.1;
        self.x_min += pan;
        self.x_max += pan;
    }

    fn pan_up(&mut self) {
        let pan = (self.y_max - self.y_min) * 0.1;
        self.y_min -= pan;
        self.y_max -= pan;
    }

    fn pan_down(&mut self) {
        let pan = (self.y_max - self.y_min) * 0.1;
        self.y_min += pan;
        self.y_max += pan;
    }

    fn zoom_in(&mut self) {
        let x_center = (self.x_min + self.x_max) / 2.0;
        let y_center = (self.y_min + self.y_max) / 2.0;
        let x_range = (self.x_max - self.x_min) * 0.9;
        let y_range = (self.y_max - self.y_min) * 0.9;
        self.x_min = x_center - x_range / 2.0;
        self.x_max = x_center + x_range / 2.0;
        self.y_min = y_center - y_range / 2.0;
        self.y_max = y_center + y_range / 2.0;
    }

    fn zoom_out(&mut self) {
        let x_center = (self.x_min + self.x_max) / 2.0;
        let y_center = (self.y_min + self.y_max) / 2.0;
        let x_range = (self.x_max - self.x_min) * 1.1;
        let y_range = (self.y_max - self.y_min) * 1.1;
        self.x_min = x_center - x_range / 2.0;
        self.x_max = x_center + x_range / 2.0;
        self.y_min = y_center - y_range / 2.0;
        self.y_max = y_center + y_range / 2.0;
    }
}

impl Widget for &Mandelbrot {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let x_step = (self.x_max - self.x_min) / area.width as f64;
        let y_step = (self.y_max - self.y_min) / area.height as f64 / 2.0;
        let mut pixels = vec![0; (area.width * area.height * 2) as usize];
        for y in 0..area.height * 2 {
            for x in 0..area.width {
                let c = Complex::new(
                    self.x_min + x as f64 * x_step,
                    self.y_min + y as f64 * y_step,
                );
                let mut z = Complex::new(0.0, 0.0);
                let mut n = 0;

                while z.norm_sqr() <= 4.0 && n < self.max_iterations {
                    z = z * z + c;
                    n += 1;
                }

                pixels[(y * area.width + x) as usize] = n;
            }
        }

        // coloring https://en.wikipedia.org/wiki/Plotting_algorithms_for_the_Mandelbrot_set#Histogram_coloring

        let mut histogram = vec![0; self.max_iterations as usize + 1];
        for &count in &pixels {
            if count < self.max_iterations {
                histogram[count as usize] += 1;
            }
        }

        let mut total = 0;
        for i in 0..histogram.len() {
            total += histogram[i];
        }

        let mut brightness = vec![0.0; pixels.len()];
        for (count, brightness) in zip(pixels, &mut brightness) {
            if count == self.max_iterations {
                continue;
            }
            for i in 0..count {
                *brightness += histogram[i as usize] as f64 / total as f64;
            }
        }

        // iterate to draw a half block on each buffer cell
        for y in 0..area.height {
            for x in 0..area.width {
                let top = brightness[(y * 2 * area.width + x) as usize];
                let bottom = brightness[((y * 2 + 1) * area.width + x) as usize];

                let fg = Color::Rgb(0, 0, (top * 255.0).floor() as u8);
                let bg = Color::Rgb(0, 0, (bottom * 255.0).floor() as u8);

                buf[(x, y)].set_fg(fg).set_bg(bg).set_symbol("▀");
            }
        }
    }
}
