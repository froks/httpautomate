use anyhow::Result;
use clap::{arg, command, value_parser, ArgAction, Command};
use httpautomate::execute::execute_http_files;

fn main() -> Result<()> {
    let cmd = Command::new("httpmate")
        .author(clap::crate_authors!())
        .version(clap::crate_version!())
        .about(clap::crate_description!())
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            command!("run")
                .about("run one or multiple http files")
                .arg(
                    arg!(<FILES> "files")
                        .help("http files that shall be executed")
                        .required(true)
                        .value_parser(value_parser!(std::path::PathBuf))
                        .action(ArgAction::Append),
                )
                .arg(
                    arg!(-e - -"env-file")
                        .help("Environment file - you can specify this argument multiple times")
                        .value_parser(value_parser!(std::path::PathBuf))
                        .action(ArgAction::Append),
                )
                .arg(
                    arg!(-E - -env)
                        .help("Environment to use")
                        .value_parser(value_parser!(String))
                        .action(ArgAction::Set),
                ),
        );

    let matches = cmd.get_matches();
    let result = match matches.subcommand() {
        Some(("run", matches)) => execute_http_files(
            matches
                .get_many::<std::path::PathBuf>("FILES")
                .unwrap()
                .collect(),
            matches
                .get_many::<std::path::PathBuf>("env-file")
                .unwrap()
                .collect(),
            matches.get_one::<String>("env").unwrap(),
        ),
        _ => unreachable!("this should've been prevented"),
    };
    return result;
}
