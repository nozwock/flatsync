use zbus::ConnectionBuilder;

mod api;
mod dbus;
mod imp;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let daemon = dbus::Daemon::new().await;

    let _con = ConnectionBuilder::session()?
        .name("app.drey.FlatSync.Daemon")?
        .serve_at("/app/drey/FlatSync/Daemon", daemon)?
        .build()
        .await?;

    println!("Started daemon. Press Ctrl+C to exit.");

    let imp = imp::Impl::new().await;
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
    loop {
        interval.tick().await;
        if imp.get_gist_secret().await.is_err() {
            println!("No secret found");
            continue;
        }
        imp.post_gist().await;
    }
}
