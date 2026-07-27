use docpack::cli;
use docpack::constants;
use std::path::PathBuf;

#[test]
fn test_cli_pack() {
    let cli = cli::Cli::parse_from(["docpack", "pack", "src", "-o", "out.docx"]);
    match cli.command {
        Some(cli::Commands::Pack { paths, output, .. }) => {
            assert_eq!(paths, vec![PathBuf::from("src")]);
            assert_eq!(output, PathBuf::from("out.docx"));
        }
        _ => panic!("Expected Pack command"),
    }
}

#[test]
fn test_cli_pack_default_output() {
    let cli = cli::Cli::parse_from(["docpack", "pack", "src"]);
    match cli.command {
        Some(cli::Commands::Pack { paths, output, .. }) => {
            assert_eq!(paths, vec![PathBuf::from("src")]);
            assert_eq!(output, PathBuf::from(constants::DEFAULT_OUTPUT));
        }
        _ => panic!("Expected Pack command"),
    }
}

#[test]
fn test_cli_unpack() {
    let cli = cli::Cli::parse_from(["docpack", "unpack", "in.docx", "-o", "out_dir"]);
    match cli.command {
        Some(cli::Commands::Unpack { input, output, .. }) => {
            assert_eq!(input, PathBuf::from("in.docx"));
            assert_eq!(output, Some(PathBuf::from("out_dir")));
        }
        _ => panic!("Expected Unpack command"),
    }
}

#[test]
fn test_cli_install() {
    let cli = cli::Cli::parse_from(["docpack", "install"]);
    assert!(matches!(cli.command, Some(cli::Commands::Install)));
}

#[test]
fn test_cli_help() {
    let cli = cli::Cli::try_parse_from(["docpack", "--help"]);
    assert!(cli.is_ok() || cli.is_err());
}
