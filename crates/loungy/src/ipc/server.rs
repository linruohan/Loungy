use std::{
    net::{Ipv4Addr, SocketAddr},
    pin::Pin,
    task::{Context, Poll},
};

use anyhow::{Error, anyhow};
use clap::{Arg, ValueEnum, command};
use gpui::{AsyncApp, private::serde_json};
use serde::{Deserialize, Serialize};
use smol::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use super::{SOCKET_PATH, SOCKET_PORT};
use crate::{
    commands::RootCommands,
    state::{LActions, StateModel},
    window::LWindow,
};

// 平台无关的监听器枚举
pub enum PlatformListener {
    #[cfg(unix)]
    Unix(smol::net::unix::UnixListener),
    #[cfg(windows)]
    Tcp(smol::net::TcpListener),
}

impl PlatformListener {
    pub async fn accept(&self) -> Result<(PlatformStream, SocketAddr), Error> {
        match self {
            #[cfg(unix)]
            PlatformListener::Unix(listener) => {
                let (stream, _) = listener.accept().await?;
                Ok((
                    PlatformStream::Unix(stream),
                    SocketAddr::from(([127, 0, 0, 1], 0)),
                ))
            }
            #[cfg(windows)]
            PlatformListener::Tcp(listener) => {
                let (stream, addr) = listener.accept().await?;
                Ok((PlatformStream::Tcp(stream), addr))
            }
        }
    }
}

// 平台无关的流枚举
pub enum PlatformStream {
    #[cfg(unix)]
    Unix(smol::net::unix::UnixStream),
    #[cfg(windows)]
    Tcp(smol::net::TcpStream),
}
// 为 PlatformStream 实现 AsyncRead
impl AsyncRead for PlatformStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<std::io::Result<usize>> {
        match &mut *self {
            #[cfg(unix)]
            PlatformStream::Unix(stream) => Pin::new(stream).poll_read(cx, buf),
            #[cfg(windows)]
            PlatformStream::Tcp(stream) => Pin::new(stream).poll_read(cx, buf),
        }
    }
}

// 为 PlatformStream 实现 AsyncWrite
impl AsyncWrite for PlatformStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        match &mut *self {
            #[cfg(unix)]
            PlatformStream::Unix(stream) => Pin::new(stream).poll_write(cx, buf),
            #[cfg(windows)]
            PlatformStream::Tcp(stream) => Pin::new(stream).poll_write(cx, buf),
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        match &mut *self {
            #[cfg(unix)]
            PlatformStream::Unix(stream) => Pin::new(stream).poll_flush(cx),
            #[cfg(windows)]
            PlatformStream::Tcp(stream) => Pin::new(stream).poll_flush(cx),
        }
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        match &mut *self {
            #[cfg(unix)]
            PlatformStream::Unix(stream) => Pin::new(stream).poll_close(cx),
            #[cfg(windows)]
            PlatformStream::Tcp(stream) => Pin::new(stream).poll_close(cx),
        }
    }
}

impl PlatformStream {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        AsyncReadExt::read(self, buf).await.map_err(Into::into)
    }

    async fn write_all(&mut self, buf: &[u8]) -> Result<(), Error> {
        AsyncWriteExt::write_all(self, buf)
            .await
            .map_err(Into::into)
    }
}

fn extract_port_from_path(path: &str) -> Option<u16> {
    if path.starts_with("/") {
        path.split('/').next_back()?.parse().ok()
    } else {
        path.parse().ok()
    }
}
pub async fn setup_socket() -> anyhow::Result<PlatformListener> {
    #[cfg(unix)]
    {
        use smol::net::unix::{UnixListener, UnixStream};

        if Path::new(SOCKET_PATH).exists() {
            if UnixStream::connect(Path::new(SOCKET_PATH)).await.is_ok() {
                return Err(anyhow::anyhow!("Server already running"));
            }
            std::fs::remove_file(SOCKET_PATH)?;
        };

        let listener = UnixListener::bind(Path::new(SOCKET_PATH))?;
        log::info!("Listening on Unix socket: {}", SOCKET_PATH);

        Ok(PlatformListener::Unix(listener))
    }

    #[cfg(windows)]
    {
        let port = extract_port_from_path(SOCKET_PATH).unwrap_or(SOCKET_PORT);
        let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, port));

        // 检查端口是否被占用
        if smol::net::TcpStream::connect(addr).await.is_ok() {
            return Err(anyhow::anyhow!("Server already running on port {}", port));
        }

        let listener = smol::net::TcpListener::bind(addr).await?;
        log::info!("Listening on TCP socket: {}", addr);

        Ok(PlatformListener::Tcp(listener))
    }
}
pub async fn start_server(listener: PlatformListener, mut cx: AsyncApp) -> anyhow::Result<()> {
    let commands = cx.read_global::<RootCommands, _>(|commands, _| commands.clone())?;
    loop {
        let (stream, _) = listener.accept().await?;
        let cx = cx.clone();
        let commands = commands.clone();

        cx.spawn(async move |cx| {
            if let Err(e) = handle_client(stream, commands, cx).await {
                log::error!("Client handling error: {}", e);
            }
        })
        .detach();
    }
}

async fn handle_client(
    mut stream: PlatformStream,
    commands: RootCommands,
    mut cx: AsyncApp,
) -> anyhow::Result<()> {
    // Send available commands to the client
    let bytes = serde_json::to_vec(&commands)?;
    stream.write_all(&bytes).await?;

    let mut buf = vec![0; 1024];
    let n = stream.read(&mut buf).await?;

    let matches: CommandPayload = serde_json::from_slice(&buf[..n])?;

    let _ = cx.update::<anyhow::Result<()>>(|cx| {
        match matches.action {
            TopLevelCommand::Toggle => {
                LWindow::toggle(cx);
            }
            TopLevelCommand::Show => {
                LWindow::open(cx);
            }
            TopLevelCommand::Hide => {
                LWindow::close(cx);
            }
            TopLevelCommand::Quit => {
                cx.quit();
            }
            TopLevelCommand::Command => {
                let Some(c) = matches.command else {
                    return Err(anyhow!("No command provided"));
                };
                let Some((_, command)) = commands.commands.iter().find(|(k, _)| {
                    let split = k.split("::").collect::<Vec<_>>();
                    c.eq(split[2])
                }) else {
                    return Err(anyhow!("Command not found"));
                };

                let state = cx.global::<StateModel>();
                let state = state.inner.read(cx);
                let mut is_active = false;
                if let Some(active) = state.stack.last() {
                    is_active = active.id.eq(&command.id);
                };
                if !is_active {
                    StateModel::update(
                        |this, cx| {
                            this.reset(cx);
                        },
                        cx,
                    );
                    (command.action)(&mut LActions::default(cx), cx);
                    LWindow::open(cx);
                } else {
                    LWindow::toggle(cx);
                }
            }
            TopLevelCommand::Pipe => {}
        }
        Ok(())
    });
    Ok(())
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommandPayload {
    pub action: TopLevelCommand,
    pub command: Option<String>,
}

#[derive(Clone, Debug, ValueEnum, Serialize, Deserialize)]
pub enum TopLevelCommand {
    Toggle,
    Show,
    Hide,
    Quit,
    Command,
    Pipe,
}

impl From<TopLevelCommand> for clap::builder::OsStr {
    fn from(cmd: TopLevelCommand) -> Self {
        match cmd {
            TopLevelCommand::Toggle => "toggle".into(),
            TopLevelCommand::Show => "show".into(),
            TopLevelCommand::Hide => "hide".into(),
            TopLevelCommand::Quit => "quit".into(),
            TopLevelCommand::Command => "command".into(),
            TopLevelCommand::Pipe => "pipe".into(),
        }
    }
}

pub fn get_command(commands: &RootCommands) -> clap::Command {
    command!()
        .arg(
            Arg::new("Action")
                .value_parser(clap::builder::EnumValueParser::<TopLevelCommand>::new())
                .required(true),
        )
        .arg(
            Arg::new("Command")
                .required_if_eq("Action", TopLevelCommand::Command)
                .value_parser(
                    commands
                        .commands
                        .keys()
                        .map(|key| {
                            let split = key.split("::").collect::<Vec<_>>();
                            split[2].to_string()
                        })
                        .collect::<Vec<_>>(),
                ),
        )
        .arg(
            Arg::new("Delimeter")
                .long("Delimeter")
                .short('d')
                .required_if_eq("Action", TopLevelCommand::Pipe)
                .default_value(" "),
        )
}
