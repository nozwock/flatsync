global_conf = configuration_data()
global_conf.set_quoted('APP_ID', application_id)
global_conf.set_quoted('PKGDATADIR', pkgdatadir)
global_conf.set_quoted('PROFILE', profile)
global_conf.set_quoted('VERSION', version + version_suffix)
global_conf.set_quoted('GETTEXT_PACKAGE', gettext_package)
global_conf.set_quoted('LOCALEDIR', localedir)
global_conf.set_quoted('AUTOSTART_DESKTOP_FILE_PATH', get_option('prefix') / get_option('datadir') / 'applications' / '@0@.Autostart.desktop'.format(application_id))

config = configure_file(
  input: 'config.rs.in',
  output: 'config.rs',
  configuration: global_conf
)

# Copy the config.rs output to the source directory.
run_command(
  'cp',
  meson.project_build_root() / 'libflatsync-common' / 'src' / 'config.rs',
  meson.project_source_root() / 'libflatsync-common' / 'src' / 'config.rs',
  check: true
)
