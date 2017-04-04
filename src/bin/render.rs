#![feature(slice_patterns)]

extern crate docopt;
extern crate rustc_serialize;
extern crate voxelmap_cache;

use docopt::Docopt;
use voxelmap_cache::render_parallelized;
use voxelmap_cache::*;
use voxelmap_cache::colorizer::*;
use voxelmap_cache::processor::*;

const USAGE: &'static str = "
Usage: rustmap [-q] [-t threads] <cache> <output> (simple | light | biome | height | terrain)

Options:
    -q, --quiet  Do not output info messages.
";

#[derive(Debug, RustcDecodable)]
struct Args {
    flag_quiet: bool,
    arg_cache: String,
    arg_output: String,
    cmd_simple: bool,
    cmd_light: bool,
    cmd_biome: bool,
    cmd_height: bool,
    cmd_terrain: bool,
    arg_threads: Option<usize>,
}

impl Args {
    fn get_colorizer(&self) -> Colorizer {
        if self.cmd_simple {
            Colorizer::Simple
        } else if self.cmd_light {
            Colorizer::Light
        } else if self.cmd_biome {
            Colorizer::Biome
        } else if self.cmd_height {
            Colorizer::Height
        } else if self.cmd_terrain {
            Colorizer::Terrain
        } else {
            Colorizer::Unknown
        }
    }

    fn get_processor(&self) -> Box<Processor> {
        if self.arg_output.contains("{x}") && self.arg_output.contains("{z}")
            || self.arg_output.contains("{tile}") {
            Box::new(TilesProcessor { tiles_pattern: self.arg_output.clone() })
        } else {
            Box::new(SingleImageProcessor::new(&self.arg_output))
        }
    }
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());
    let verbose = !args.flag_quiet;

    match get_regions(args.arg_cache.as_ref()) {
        Err(e) => print!("{:?}", e),
        Ok(regions) => {
            let colorizer = args.get_colorizer();
            let processor = args.get_processor();

            if verbose {
                println!("Rendering {} regions from {} to {} with {:?}",
                         regions.len(), args.arg_cache, args.arg_output, colorizer);
            }

            render_parallelized(
                processor,
                colorizer,
                regions,
                args.arg_threads.unwrap_or(4),
                verbose,
            );
        },
    }
}
