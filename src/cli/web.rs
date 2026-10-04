//! `web` command handlers.

use super::{opt_u16_arg, CliError};
use springsharp::*;

pub(crate) fn run(cmd: &str, args: &[String]) -> Result<(), CliError> {
    match cmd {
        "web" => {
            let port = opt_u16_arg(args, 1, "port", 8080)?;
            run_server(port).map_err(|e| CliError::runtime(format!("web server failed: {e}")))?;
        }
        _ => unreachable!("web::run called with unexpected command {cmd}"),
    }
    Ok(())
}
