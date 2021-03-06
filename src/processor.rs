extern crate lodepng;

use std::fs;
use std::path::Path;
use std::time;

pub const REGION_WIDTH: usize = 256;
pub const REGION_HEIGHT: usize = 256;
pub const REGION_BLOCKS: usize = REGION_WIDTH * REGION_HEIGHT;

pub type RegionPos = (i32, i32);
pub type RegionPixels = [u32; REGION_BLOCKS];

pub trait Processor {
    fn process_region(&mut self, region_pos: RegionPos, region_pixels: Box<RegionPixels>);
    fn pre_process(&mut self) {}
    fn post_process(&mut self) {}
}

pub struct TilesProcessor {
    pub tiles_pattern: String,
}

impl Processor for TilesProcessor {
    fn process_region(&mut self, region_pos: RegionPos, region_pixels: Box<RegionPixels>) {
        let (rx, rz) = region_pos;
        let img_path = self.tiles_pattern
            .replace("{tile}", &(format!("{},{}", rx, rz)))
            .replace("{x}", &rx.to_string())
            .replace("{z}", &rz.to_string());
        let dir = Path::new(&img_path).parent()
            .expect(&format!("Getting containing directory of tile {}", img_path));
        fs::create_dir_all(dir)
            .expect(&format!("Creating containing directory for tile {}", img_path));
        lodepng::encode32_file(&img_path, &region_pixels[..], REGION_WIDTH, REGION_HEIGHT)
            .expect(&format!("Encoding tile {}", img_path));
    }
}

pub struct SingleImageProcessor {
    pixbuf: Box<[u32]>,
    img_path: String,
}

const IMG_WIDTH: usize = 40 * REGION_WIDTH;
const IMG_HEIGHT: usize = 40 * REGION_HEIGHT;
const IMG_WEST: i32 = -20 * REGION_WIDTH as i32;
const IMG_NORTH: i32 = -20 * REGION_HEIGHT as i32;

impl SingleImageProcessor {
    pub fn new(img_pattern: &String) -> SingleImageProcessor {
        SingleImageProcessor {
            pixbuf: vec![0_u32; IMG_WIDTH * IMG_HEIGHT].into_boxed_slice(),
            img_path: SingleImageProcessor::replace_timestamp(img_pattern),
        }
    }

    pub fn replace_timestamp(img_pattern: &String) -> String {
        let unix_time = time::SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap().as_secs();
        img_pattern.replace("{t}", &unix_time.to_string())
    }
}

impl Processor for SingleImageProcessor {
    fn process_region(&mut self, region_pos: RegionPos, region_pixels: Box<RegionPixels>) {
        let (rx, rz) = region_pos;
        let x_off: i32 = rx * REGION_WIDTH as i32 - IMG_WEST;
        let z_off: i32 = rz * REGION_HEIGHT as i32 - IMG_NORTH;
        for (line_z, region_line) in region_pixels.chunks(REGION_WIDTH).enumerate() {
            let img_line = (x_off + IMG_WIDTH as i32 * (z_off + line_z as i32)) as usize;
            let img_slice = &mut self.pixbuf[img_line..img_line + REGION_WIDTH];
            img_slice.copy_from_slice(region_line);
        }
    }

    fn pre_process(&mut self) {
        let dir = Path::new(&self.img_path).parent()
            .expect(&format!("Getting containing directory of {}", self.img_path));
        fs::create_dir_all(dir)
            .expect(&format!("Creating containing directory for {}", self.img_path));
    }

    fn post_process(&mut self) {
        println!("Saving image as {}", self.img_path);
        lodepng::encode32_file(&self.img_path, &self.pixbuf[..], IMG_WIDTH, IMG_HEIGHT)
            .expect(&format!("Encoding image {}", self.img_path));
    }
}

pub fn get_processor(arg_output: &String) -> Box<Processor> {
    if arg_output.contains("{x}") && arg_output.contains("{z}")
        || arg_output.contains("{tile}") {
        Box::new(TilesProcessor { tiles_pattern: arg_output.clone() })
    } else {
        Box::new(SingleImageProcessor::new(&arg_output))
    }
}
