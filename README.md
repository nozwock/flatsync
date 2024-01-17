# FlatSync

Synchronise your Flatpak packages across machines

Currently, it’s rather bothersome to sync the Flatpak packages installed on multiple systems: One either has to manually keep them in sync or use the CLI to get a list of installed Flatpaks. To improve this situation, FlatSync was created. It creates a D-Bus daemon that automatically collects a list of installed Flatpaks and pushes it to a service like GitHub Gists. It also pulls configuration from this service and installs Flatpaks that are missing remotely.

To interact with this daemon, FlatSync provides both a CLI and a GUI component, see [Architecture](#architecture).

# Getting Started

0. **Setup Rust**: See [rustup.rs](https://rustup.rs) for more information

1. **Generate `config.rs`**: Run `meson setup build` to create essential configuration files.

2. **Install Dependencies** (For Red Hat-based systems like RHEL, CentOS, Fedora):

   ```bash
   sudo dnf install libadwaita-devel flatpak-devel gtk4-devel
    ```

Now that all the basics are setup, you should be able to build the project with `cargo build`. To learn the basics, check out [The Rust Programming Language](https://doc.rust-lang.org/book/) and [GUI development with Rust and GTK4](https://gtk-rs.org/gtk4-rs/stable/latest/book/), two excellent books for beginners.

# Architecture

This project is split into multiple parts:

* libflatsync-common: A library for (utility) functions that are shared across the projects
* flatsync-daemon: A D-Bus daemon that periodically fetches installed flatpaks via `libflatpak` and pushes them to a gist provider. It provides a D-Bus API that both flatsync-cli and flatsync can use for setting things like the gist secret token or manually triggering a push to the gist provider.
* flatsync-cli: A CLI application for interfacing with flatsync-daemon
* flatsync: A GUI application for interfacing with flatsync-daemon

# Communication

* Matrix: You can join the chat room [here](https://matrix.to/#/#flatsync:gnome.org)
