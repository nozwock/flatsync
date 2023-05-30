build_targets = [ 'flatsync', 'flatsync-cli', 'flatsync-daemon' ]

cargo_options = [ '--manifest-path', meson.project_source_root() / 'Cargo.toml' ]
cargo_options += [ '--target-dir', meson.project_build_root() / 'src' ]

if get_option('profile') == 'default'
  cargo_options += [ '--release' ]
  rust_target = 'release'
  message('Building in release mode')
else
  rust_target = 'debug'
  message('Building in debug mode')
endif

cargo_env = [ 'CARGO_HOME=' + meson.project_build_root() / 'cargo-home' ]

cargo_options_mut = cargo_options

base_command = [
    'env',
    cargo_env,
    cargo, 'build',
]
foreach i : build_targets
  cargo_options_mut = cargo_options + [ '-p', i ]
  cmd = base_command + cargo_options_mut +  ['&&', 'cp', join_paths('src', rust_target, i), '@OUTPUT@']

  cargo_build = custom_target(
    'cargo-build-' + i,
    build_by_default: true,
    build_always_stale: true,
    output: i,
    console: true,
    install: true,
    install_dir: bindir,
    depends: resources,
    command: cmd,
  )
endforeach
