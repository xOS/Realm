use std::io::Result;
use futures::future::join_all;

mod tcp;
use tcp::TcpListener;
use crate::utils::Endpoint;

pub async fn run(eps: Vec<Endpoint>) {
    let mut workers = vec![];
    for ep in eps.into_iter() {
        #[cfg(feature = "udp")]
        if ep.udp {
            workers.push(tokio::spawn(proxy_udp(ep.clone())))
        }
        workers.push(tokio::spawn(proxy_tcp(ep)));
    }
    join_all(workers).await;
}

async fn proxy_tcp(ep: Endpoint) -> Result<()> {
    let Endpoint {
        local,
        remote,
        conn_opts,
        ..
    } = ep;
    let lis = TcpListener::bind(local)
        .await
        .unwrap_or_else(|_| panic!("unable to bind {}", &local));
    while let Ok((stream, _)) = lis.accept().await {
        tokio::spawn(tcp::proxy(stream, remote.clone(), conn_opts));
    }
    Ok(())
}

#[cfg(feature = "udp")]
mod udp;

#[cfg(feature = "udp")]
async fn proxy_udp(ep: Endpoint) -> Result<()> {
    let Endpoint {
        local,
        remote,
        conn_opts,
        ..
    } = ep;
    udp::proxy(local, remote.clone(), conn_opts).await
}
