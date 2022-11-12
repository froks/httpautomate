use clap::{arg, ArgAction, Command, command, value_parser};
use httpautomate::errors::AutomateError;
use httpautomate::execute::execute_http_files;

fn main() -> Result<(), AutomateError> {
    let cmd = Command::new("httpmate")
        .author(clap::crate_authors!())
        .version(clap::crate_version!())
        .about(clap::crate_description!())
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            command!("run")
            .about("run one or multiple http files")
            .arg(arg!(<FILES> "files")
                .help("http files that shall be executed")
                .required(true)
                .value_parser(value_parser!(std::path::PathBuf))
                .action(ArgAction::Append))
        );

    let matches = cmd.get_matches();
    let result = match matches.subcommand() {
        Some(("run", matches)) =>
            execute_http_files(matches.get_many::<std::path::PathBuf>("FILES").unwrap().collect()),
        _ => unreachable!("this should've been prevented")
    };
    return result;
}
