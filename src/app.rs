use crate::{cache::Cache, commands::Command, resp::RESPType};
use anyhow::Result;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::{mpsc, oneshot},
};

const MESSAGE_SIZE: usize = 512;

struct ExectionRequest {
    pub cmd: Command,
    pub response_chan: oneshot::Sender<Result<Option<RESPType>, String>>,
}

pub struct App {
    listener: TcpListener,
}

impl App {
    pub fn new() -> Result<Self> {
        Self::with_port("6379")
    }

    pub fn with_port(port: &str) -> Result<Self> {
        let std_listener = match std::net::TcpListener::bind(format!("0.0.0.0:{port}")) {
            Ok(listener) => listener,
            Err(e) => {
                return Err(e.into());
            }
        };

        let listener = TcpListener::from_std(std_listener)?;
        println!("Server listening on port {port}");

        Ok(Self { listener })
    }

    pub async fn run(&self) -> Result<()> {
        let (send_cmd, mut recv_wait_handle): (
            mpsc::Sender<ExectionRequest>,
            mpsc::Receiver<ExectionRequest>,
        ) = mpsc::channel(256);

        tokio::spawn(async move {
            let mut cache = Cache::new();

            while let Some(req) = recv_wait_handle.recv().await {
                req.response_chan
                    .send(Ok(Some(req.cmd.run(&mut cache))))
                    .unwrap();
            }
        });

        loop {
            let incoming = self.listener.accept().await;

            match incoming {
                Ok((mut stream, addr)) => {
                    println!("Handling connection from {addr:?}");
                    let send_ptr = send_cmd.clone();

                    tokio::spawn(async move {
                        Self::handle_connection(&mut stream, send_ptr)
                            .await
                            .unwrap();
                    });
                }
                Err(e) => {
                    println!("error: {e}");
                }
            }
        }
    }

    async fn handle_connection(
        stream: &mut TcpStream,
        send_req: mpsc::Sender<ExectionRequest>,
    ) -> Result<()> {
        loop {
            let mut buf = [0; MESSAGE_SIZE];
            _ = stream.read(&mut buf).await?;

            let (command_buf, _) = RESPType::unpack(&buf);

            match command_buf {
                RESPType::Array(_) => {
                    if let Some(resp) =
                        Self::handle_command(command_buf, &mut send_req.clone()).await
                    {
                        stream.write_all(&resp.pack()).await?;
                    }
                }
                _ => break,
            }
        }

        Ok(())
    }

    async fn handle_command(
        args: RESPType,
        send_req: &mut mpsc::Sender<ExectionRequest>,
    ) -> Option<RESPType> {
        let (sender, mut receiver) = oneshot::channel();

        let command = match args.to_command() {
            Ok(command) => command,
            Err(err) => {
                return Some(RESPType::Error(err.to_string()));
            }
        };
        println!("{command}");

        let process = send_req
            .send(ExectionRequest {
                cmd: command,
                response_chan: sender,
            })
            .await;

        if process.is_err() {
            receiver.close();
            _ = receiver.try_recv();

            return Some(RESPType::Error("Busy".to_string()));
        }

        return match receiver.await {
            Ok(Ok(Some(v))) => Some(v),
            Ok(Err(e)) => Some(RESPType::Error(e)),
            Err(e) => Some(RESPType::Error(format!(
                "Unexpected error receiving result: {e:?}"
            ))),
            _ => None,
        };
    }
}
