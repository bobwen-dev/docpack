use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "DocPack", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(about = "Pack files into DOCX")]
    Pack {
        #[arg(required = true)]
        paths: Vec<PathBuf>,
        #[arg(short, long, default_value = "output.docx")]
        output: PathBuf,
        #[arg(long)]
        exclude: Option<PathBuf>,
    },
    #[command(about = "Unpack DOCX to files")]
    Unpack {
        input: PathBuf,
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    #[command(about = "Install context menu")]
    Install,
    #[command(about = "Uninstall context menu")]
    Uninstall,
    #[command(name = "gui-pack", hide = true)]
    GuiPack { paths: Vec<PathBuf> },
    #[command(name = "gui-unpack", hide = true)]
    GuiUnpack { paths: Vec<PathBuf> },
}

impl Cli {
    pub fn parse_from<I, T>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<String> + Clone,
    {
        <Cli as Parser>::parse_from(iter.into_iter().map(|s| s.into()).collect::<Vec<_>>())
    }

    pub fn try_parse_from<I, T>(iter: I) -> Result<Self, clap::error::Error>
    where
        I: IntoIterator<Item = T>,
        T: Into<String> + Clone,
    {
        <Cli as Parser>::try_parse_from(iter.into_iter().map(|s| s.into()).collect::<Vec<_>>())
    }
}
