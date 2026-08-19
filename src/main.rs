#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;

use docpack::cli;
use docpack::gui;
use docpack::ignore;
use docpack::pack;
use docpack::platform;
use docpack::settings;
use docpack::unpack;

fn main() {
    let cli = cli::Cli::parse_from(std::env::args());
    let app_settings = settings::load_settings();

    match cli.command {
        Some(cli::Commands::Pack {
            paths,
            output,
            exclude,
        }) => {
            let output = pack::resolve_output_name(&paths, &output);
            let rules = ignore::ExcludeRules::load_or_default(
                exclude.as_deref(),
                &app_settings.exclude_patterns,
            );

            let on_progress: pack::ProgressFn = Box::new(move |p| {
                use std::io::{stderr, Write};
                match p.phase {
                    pack::ProgressPhase::Collecting => {
                        eprint!("\rCollecting files... {}/{}", p.current, p.total);
                    }
                    pack::ProgressPhase::Reading => {
                        eprint!("\rReading: {} ({}/{})", p.file, p.current, p.total);
                    }
                    pack::ProgressPhase::Writing => {
                        eprint!("\rWriting DOCX...");
                    }
                    pack::ProgressPhase::Done => {
                        eprint!("\rDone.                    \n");
                    }
                }
                let _ = stderr().flush();
            });

            let result = pack::pack_files(
                &paths,
                &output,
                &rules,
                &app_settings.local_encodings,
                Some(&on_progress),
            );

            match result {
                Ok(res) => {
                    println!(
                        "Packed {} files to {}",
                        res.file_count,
                        res.output_path.display()
                    );
                    if res.binary_skipped > 0 {
                        println!("Binary files skipped: {}", res.binary_skipped);
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Some(cli::Commands::Unpack { input, output }) => {
            let default_output = unpack::default_output_path(&input);
            let output = unpack::resolve_unpack_output(&output.unwrap_or(default_output));

            match unpack::unpack_docx(&input, &output) {
                Ok(res) => {
                    println!(
                        "Extracted {} files to {}",
                        res.file_count,
                        res.output_dir.display()
                    );
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Some(cli::Commands::Install) => {
            let exe = std::env::current_exe()
                .unwrap_or_else(|_| PathBuf::from(std::env::args().next().unwrap_or_default()));
            match platform::install::ContextMenu::install(&exe) {
                Ok(()) => println!("Context menu installed"),
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Some(cli::Commands::Uninstall) => match platform::install::ContextMenu::uninstall() {
            Ok(()) => println!("Context menu uninstalled"),
            Err(e) => eprintln!("Error: {}", e),
        },
        Some(cli::Commands::GuiPack { paths }) => {
            if let Err(e) = gui::run_gui_pack(app_settings, paths) {
                eprintln!("GUI Error: {}", e);
            }
        }
        Some(cli::Commands::GuiUnpack { paths }) => {
            if let Err(e) = gui::run_gui_unpack(app_settings, paths) {
                eprintln!("GUI Error: {}", e);
            }
        }
        None => {
            if let Err(e) = gui::run_gui(app_settings, Vec::new()) {
                eprintln!("GUI Error: {}", e);
            }
        }
    }
}
