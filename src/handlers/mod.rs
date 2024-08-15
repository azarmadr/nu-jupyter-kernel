pub mod iopub {
    use tokio::sync::{broadcast, mpsc};

    use super::Shutdown;
    use crate::jupyter::messages::multipart::Multipart;
    use crate::util::Select;
    use crate::IopubSocket;

    pub async fn handle(
        mut socket: IopubSocket,
        mut shutdown: broadcast::Receiver<Shutdown>,
        mut iopub_rx: mpsc::Receiver<Multipart>,
    ) {
        loop {
            let next = tokio::select! {
                biased;
                v = shutdown.recv() => Select::Left(v),
                v = iopub_rx.recv() => Select::Right(v.unwrap()),
            };

            let multipart = match next {
                Select::Left(Ok(Shutdown { restart: false })) => break,
                Select::Left(Ok(Shutdown { restart: true })) => continue,
                Select::Left(Err(_)) => break,
                Select::Right(multipart) => multipart,
            };
            multipart.send(&mut socket).await.unwrap();
        }
    }
}

pub mod heartbeat {
    use tokio::sync::broadcast;
    use zeromq::{SocketRecv, SocketSend};

    use super::Shutdown;
    use crate::util::Select;
    use crate::HeartbeatSocket;

    pub async fn handle(mut socket: HeartbeatSocket, mut shutdown: broadcast::Receiver<Shutdown>) {
        loop {
            let next = tokio::select! {
                biased;
                v = shutdown.recv() => Select::Left(v),
                v = socket.recv() => Select::Right(v.unwrap()),
            };

            let msg = match next {
                Select::Left(Ok(Shutdown { restart: false })) => break,
                Select::Left(Ok(Shutdown { restart: true })) => continue,
                Select::Left(Err(_)) => break,
                Select::Right(msg) => msg,
            };
            socket.send(msg).await.unwrap();
        }
    }
}

pub mod shell;
pub mod stream;

#[derive(Debug, Clone, Copy)]
pub struct Shutdown {
    pub restart: bool,
}
