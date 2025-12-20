use clap::Command;
use gpui::private::serde_json;
use smol::io::{AsyncReadExt, AsyncWriteExt};

use super::server::{CommandPayload, TopLevelCommand, get_command};
use crate::{commands::RootCommands, ipc::SOCKET_PATH};

pub async fn client_connect() -> anyhow::Result<()> {
    #[cfg(unix)]
    let mut stream = {
        use smol::net::unix::UnixStream;
        UnixStream::connect(Path::new(SOCKET_PATH)).await?
    };

    #[cfg(windows)]
    let mut stream = {
        use smol::net::TcpStream;
        // Windows 上使用 TCP 替代 Unix 套接字
        let port = SOCKET_PATH.trim_start_matches('/');
        TcpStream::connect(format!("127.0.0.1:{}", port)).await?
    };
    let mut buf = vec![0; 8096];
    let n = stream.read(&mut buf).await?;
    let root_commands: RootCommands = serde_json::from_slice(&buf[..n])?;

    let command: Command = get_command(&root_commands);

    let matches = command.get_matches();

    let payload: CommandPayload = CommandPayload {
        action: matches
            .get_one::<TopLevelCommand>("Action")
            .ok_or(anyhow::anyhow!("Action not found"))?
            .clone(),
        command: matches.get_one::<String>("Command").cloned(),
    };

    let bytes = serde_json::to_vec(&payload)?;

    stream.write_all(&bytes).await?;

    Ok(())
}
