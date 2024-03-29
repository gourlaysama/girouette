pub mod client;
pub mod location;
pub mod manager;

use crate::geoclue::client::*;
use crate::geoclue::location::*;
use crate::{geoclue::manager::*, Location};
use anyhow::*;
use dbus::message::SignalArgs;
use dbus::nonblock;
use dbus_tokio::connection;
use futures_util::*;
use log::*;
use std::time::Duration;
use tokio::time::timeout;

pub async fn get_location(timeout_duration: Duration) -> Result<Location> {
    let (resource, conn) = connection::new_system_sync()?;

    tokio::spawn(async {
        let err = resource.await;
        panic!("Lost connection to D-Bus: {}", err);
    });

    let manager = nonblock::Proxy::new(
        "org.freedesktop.GeoClue2",
        "/org/freedesktop/GeoClue2/Manager",
        Duration::from_secs(1),
        conn.clone(),
    );
    let client_path = manager.get_client().await.context("D-bus error")?;

    trace!("client path: {}", client_path);

    let client = nonblock::Proxy::new(
        "org.freedesktop.GeoClue2",
        &client_path,
        Duration::from_secs(1),
        conn.clone(),
    );

    let (incoming, mut stream) = conn
        .add_match(OrgFreedesktopGeoClue2ClientLocationUpdated::match_rule(
            None, None,
        ))
        .await
        .context("D-bus error")?
        .stream();

    // required to be able to query geoclue
    client
        .set_desktop_id("girouette".to_string())
        .await
        .context("D-bus error")?;

    client.start().await.context("D-bus error")?;

    let res: (_, OrgFreedesktopGeoClue2ClientLocationUpdated) =
        timeout(timeout_duration, stream.next())
            .await
            .map_err(|_| anyhow!("geoclue timed-out trying to find your location"))?
            .ok_or_else(|| anyhow!("no location"))?;

    conn.remove_match(incoming.token())
        .await
        .context("D-bus error")?;

    let location_path = res.1.new;

    trace!("location path: {}", location_path);

    let location = nonblock::Proxy::new(
        "org.freedesktop.GeoClue2",
        &location_path,
        Duration::from_secs(1),
        conn.clone(),
    );

    let lat = location.latitude().await.context("D-bus error")?;
    let lon = location.longitude().await.context("D-bus error")?;

    Ok(Location::LatLon(lat, lon))
}
