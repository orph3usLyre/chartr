use chartr::{image_to_kap, kap_to_image};
use std::path::PathBuf;
use tracing::{info, Level};

use anyhow::{bail, Result};
use clap::{Parser, Subcommand};

#[cfg(not(debug_assertions))]
const DEFAULT_DEBUG_LEVEL: u8 = 1;
#[cfg(debug_assertions)]
const DEFAULT_DEBUG_LEVEL: u8 = 99;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Turn debugging information on
    #[arg(short, long, default_value_t = DEFAULT_DEBUG_LEVEL, action = clap::ArgAction::Count)]
    verbosity: u8,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// converts a BSB/KAP image to a different image format
    #[command(name = "kapimg")]
    BsbToImage {
        /// The kap image
        // #[arg(short, long)]
        bsb_file: PathBuf,

        /// The output file name
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// converts a PNG image to a BSB/KAP file
    #[command(name = "imgkap")]
    ImageToBsb {
        /// The image
        img_file: PathBuf,
        /// The output file name
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}
fn main() -> Result<()> {
    let cli = Cli::parse();
    let level = match cli.verbosity {
        0 => Level::ERROR,
        1 => Level::WARN,
        2 => Level::INFO,
        3 => Level::DEBUG,
        _ => Level::TRACE,
    };
    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_file(true)
        .with_line_number(true)
        .init();

    match cli.command {
        Commands::BsbToImage { bsb_file, output } => {
            let output = match output {
                Some(o) => o,
                None => {
                    let mut output = PathBuf::new();
                    let Some(dir) = bsb_file.parent() else {
                        bail!("Invalid bsb file");
                    };
                    let Some(Some(filename)) = bsb_file.file_stem().map(|os| os.to_str()) else {
                        bail!("Invalid bsb file");
                    };
                    let suffix = "png";
                    output.push(dir);
                    output.push(format!("{}.{}", filename, suffix));
                    info!("output name: {}", output.display());
                    output
                }
            };
            kap_to_image(&bsb_file, &output)?;
        }
        Commands::ImageToBsb { img_file, output } => {
            let output = match output {
                Some(o) => o,
                None => {
                    let mut output = PathBuf::new();
                    let Some(dir) = img_file.parent() else {
                        bail!("Invalid img file");
                    };
                    let Some(Some(filename)) = img_file.file_stem().map(|os| os.to_str()) else {
                        bail!("Invalid img file");
                    };
                    let suffix = "kap";
                    output.push(dir);
                    output.push(format!("{}.{}", filename, suffix));
                    info!("output name: {}", output.display());
                    output
                }
            };
            image_to_kap(&img_file, &output)?;
        }
    }
    Ok(())
}
