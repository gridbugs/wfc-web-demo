use image::DynamicImage;
use rand::SeedableRng;
use rand_xorshift::XorShiftRng;
use std::num::NonZeroU32;
use wasm_bindgen::prelude::*;
use web_sys::CanvasRenderingContext2d;
use wfc::{orientation, Observe, Orientation, PropagateError, RunOwnAll, Size, Wave};
use wfc_image::ImagePatterns;

// When the `wee_alloc` feature is enabled, this uses `wee_alloc` as the global
// allocator.
//
// If you don't want to use `wee_alloc`, you can safely delete this.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// This is like the `main` function, except for JavaScript.
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    // This provides better error messages in debug mode.
    // It's disabled in release mode so it doesn't bloat up the file size.
    #[cfg(debug_assertions)]
    {
        console_error_panic_hook::set_once();
        wasm_logger::init(wasm_logger::Config::new(log::Level::Info));
    }

    Ok(())
}

#[wasm_bindgen]
pub struct Wfc {
    rng: XorShiftRng,
    wave: Wave,
    run: RunOwnAll,
    image_patterns: ImagePatterns,
}

#[wasm_bindgen]
impl Wfc {
    fn load_image(name: &str) -> DynamicImage {
        match name {
            "cat" => image::load_from_memory(include_bytes!("./cat.png")).unwrap(),
            _ => image::load_from_memory(include_bytes!("./flowers.png")).unwrap(),
        }
    }

    #[wasm_bindgen(constructor)]
    pub fn new(width: u32, height: u32, rng_seed: u32, image: String, rotate: bool) -> Self {
        let grid_size = Size::new(width, height);
        let wave = Wave::new(grid_size);
        let mut rng = XorShiftRng::seed_from_u64(rng_seed as u64);
        let image = Self::load_image(image.as_str());
        let pattern_size = NonZeroU32::new(3).unwrap();
        let orientation: &[Orientation] = if rotate {
            &orientation::ALL
        } else {
            &[Orientation::Original]
        };
        let image_patterns = ImagePatterns::new(&image, pattern_size, orientation);
        let global_stats = image_patterns.global_stats();
        let run = RunOwnAll::new(grid_size, global_stats, &mut rng);
        Self {
            rng,
            wave,
            run,
            image_patterns,
        }
    }

    pub fn tick(&mut self, ctx: &CanvasRenderingContext2d, width: u32, height: u32) -> bool {
        let mut finished = false;
        const N: usize = 4;
        for _ in 0..N {
            finished = self.update() || finished;
        }
        self.draw(ctx, width, height);
        finished
    }

    fn update(&mut self) -> bool {
        let finished = match self.run.step(&mut self.rng) {
            Ok(observe) => match observe {
                Observe::Complete => true,
                Observe::Incomplete => false,
            },
            Err(PropagateError::Contradiction) => {
                self.reset();
                false
            }
        };
        finished
    }

    pub fn reset(&mut self) {
        self.run.borrow_mut().reset(&mut self.rng);
    }

    fn draw(&mut self, ctx: &CanvasRenderingContext2d, width: u32, height: u32) {
        let grid_size = self.wave.grid().size();
        let cell_width = width as f64 / grid_size.width() as f64;
        let cell_height = height as f64 / grid_size.height() as f64;
        for (cell, coord) in self
            .run
            .wave_cell_ref_iter()
            .zip(grid_size.coord_iter_row_major())
        {
            let [r, g, b, a] = self.image_patterns.weighted_average_colour(&cell).0;
            let colour_string = format!("rgba({},{},{},{})", r, g, b, a);
            ctx.set_fill_style(&JsValue::from_str(colour_string.as_str()));
            ctx.fill_rect(
                coord.x as f64 * cell_width,
                coord.y as f64 * cell_height,
                cell_width,
                cell_height,
            );
        }
    }
}
