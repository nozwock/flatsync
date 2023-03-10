This project is split into multiple parts:

* libflatsync-common: A library for (utility) functions that are shared across the projects
* flatsync-daemon: A D-Bus daemon that periodically fetches installed flatpaks via `libflatpak` and pushes them to a gist provider. It provides a D-Bus API that both flatsync-cli and flatsync can use for setting things like the gist secret token or manually triggering a push to the gist provider.
* flatsync-cli: A CLI application for interfacing with flatsync-daemon
* flatsync: A GUI application for interfacing with flatsync-daemon
