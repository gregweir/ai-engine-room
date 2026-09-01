use std::io::{self, BufRead, Write};
use std::net::{Ipv4Addr, Ipv6Addr, TcpListener, TcpStream};

pub(crate) const READY_TOKEN: &str = "fixture_ready";
pub(crate) const STOP_TOKEN: &str = "fixture_stop";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FixtureFamily {
    Ipv4,
    Ipv6,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FixtureError {
    Io,
    InvalidControl,
}

pub(crate) fn run(family: FixtureFamily) -> Result<(), FixtureError> {
    match family {
        FixtureFamily::Ipv4 => {
            let listener =
                TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).map_err(|_| FixtureError::Io)?;
            let address = listener.local_addr().map_err(|_| FixtureError::Io)?;
            let client = TcpStream::connect(address).map_err(|_| FixtureError::Io)?;
            let (server, _) = listener.accept().map_err(|_| FixtureError::Io)?;
            hold(listener, client, server)
        }
        FixtureFamily::Ipv6 => {
            let listener =
                TcpListener::bind((Ipv6Addr::LOCALHOST, 0)).map_err(|_| FixtureError::Io)?;
            let address = listener.local_addr().map_err(|_| FixtureError::Io)?;
            let client = TcpStream::connect(address).map_err(|_| FixtureError::Io)?;
            let (server, _) = listener.accept().map_err(|_| FixtureError::Io)?;
            hold(listener, client, server)
        }
    }
}

fn hold(
    _listener: TcpListener,
    _client: TcpStream,
    _server: TcpStream,
) -> Result<(), FixtureError> {
    let mut stdout = io::stdout().lock();
    stdout
        .write_all(format!("{READY_TOKEN}\n").as_bytes())
        .and_then(|_| stdout.flush())
        .map_err(|_| FixtureError::Io)?;

    let mut control = String::new();
    io::stdin()
        .lock()
        .read_line(&mut control)
        .map_err(|_| FixtureError::Io)?;
    if control != format!("{STOP_TOKEN}\n") {
        return Err(FixtureError::InvalidControl);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixture_control_tokens_are_fixed_and_non_numeric() {
        assert_eq!(READY_TOKEN, "fixture_ready");
        assert_eq!(STOP_TOKEN, "fixture_stop");
        assert!(READY_TOKEN
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte == b'_'));
        assert!(STOP_TOKEN
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte == b'_'));
    }
}
