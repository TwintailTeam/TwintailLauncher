# Nix package for Twintail Launcher

This repository is a fork of [TwintailLauncher](https://github.com/TwintailTeam/TwintailLauncher) with a Nix flake added, so the launcher can be built and run on NixOS (or any system with Nix flakes) straight from the sources.

Tested on NixOS (unstable, `x86_64-linux`) with Genshin Impact and Honkai: Star Rail.

## Try it

```sh
nix run github:Yuna404/TwintailFlake
```

The first run compiles the launcher from source, which takes a few minutes.

## Install it

Add the repository as an input of your flake:

```nix
inputs.twintail = {
  url = "github:Yuna404/TwintailFlake";
  inputs.nixpkgs.follows = "nixpkgs";
};
```

Then add the package, for example in `home.packages` or `environment.systemPackages`:

```nix
inputs.twintail.packages.x86_64-linux.default
```

An overlay is also available (`overlays.default`) and adds `pkgs.twintaillauncher`.
If you use the overlay, your `pkgs` needs `allowUnfree = true` because `steam-run` is unfree.

## Requirements

- Nix with flakes enabled
- A recent nixpkgs (the package uses `fetchPnpmDeps` with `fetcherVersion = 4`)
- `x86_64-linux`

## How it works

- Built with `rustPlatform.buildRustPackage` and `cargo-tauri.hook`, with the frontend dependencies fetched through pnpm.
- The launcher is wrapped in `steam-run`, because pressure-vessel and the game runners expect a regular FHS environment, which NixOS does not provide.
- The wrapper sets `GDK_BACKEND=x11` and `WEBKIT_DISABLE_DMABUF_RENDERER=1` (Wayland and DMABUF rendering caused problems with WebKit on NVIDIA), and adds `libayatana-appindicator` to `LD_LIBRARY_PATH` for the tray icon.
- The version is read from `package.json`.

### Why there is a `postPatch`

The Sparkle patch (`apply_patch` in `src-tauri/src/utils/mod.rs`) copies `hkrpg_patch.dll` to `jsproxy.dll` with `fs::copy`, which also copies the permissions of the source file. On Nix the source lives in the read-only store, so the copy ends up read-only too. The next launch then fails to overwrite it and the launcher panics with `PermissionDenied`.

The `postPatch` removes the old `jsproxy.dll` before copying. It can be dropped once this is fixed in the launcher itself.

## Maintenance

`cargoHash` and the `pnpmDeps` hash in `nix/package.nix` depend on `Cargo.lock` and `pnpm-lock.yaml`. When those files change, the build fails with a `hash mismatch` error: copy the hash shown after `got:` into `nix/package.nix` and build again.

## Troubleshooting

- **`bwrap: Can't chdir to ...`**: `steam-run` has its own private `/tmp`. Run the launcher from your home directory, not from a directory under `/tmp`.
- **Build fails on `fetcherVersion`**: your nixpkgs is too old, update it.
