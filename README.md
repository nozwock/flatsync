# Flatpak synching between machines

This is a new application participating in GNOME's GSoC '23: https://gsoc.gnome.org/2023/#flatpak-sync-between-machines

Currently, it’s rather bothersome to sync the Flatpak packages installed on multiple systems: One either has to manually keep them in sync or use the CLI to get a list of installed Flatpaks.

As such, it would be useful to have an application that can do this for users. The go-to approach would be a D-Bus daemon that automatically (either on changes or periodically) creates a list of installed Flatpaks and pushes it to a service like GitHub Gists. If a new version is detected (which comes from a different system), the daemon would fetch it and adjust the local installation accordingly.

Additionally, there should be a GUI application that can interface with the daemon for setup and configuration. A CLI application for usage independent of the DE and manual syncing would be a plus.

# Requirements

Since the project should be useable on Silverblue, it’d be good if the binary can run on it by default. Since Silverblue doesn’t have GJS nor Python installed by default, Rust seems like a good choice now that we have zbus.

=> Familiarity with Rust & DBus

# Communication

* Matrix: @cogitri:gnome.org

Mentor(s): [Rasmus Thomsen](https://gitlab.gnome.org/Cogitri)

Project length: 175 hours

More information: https://gitlab.gnome.org/Teams/Engagement/internship-project-ideas/-/issues/34
