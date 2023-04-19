/// Defines helper functions for raw operations like fetching a list of available
/// installations, transactions such as installation, removal and updates.
use crate::error::Error;
use libflatpak::gio;
use libflatpak::Installation;

pub fn installations() -> Result<Vec<Installation>, Error> {
    let mut system = libflatpak::system_installations(gio::Cancellable::NONE)
        .map_err(Error::FlatpakInstallationQueryFailure)?;
    let user = Installation::new_user(gio::Cancellable::NONE)
        .map_err(Error::FlatpakInstallationQueryFailure)?;
    system.push(user);
    Ok(system)
}
