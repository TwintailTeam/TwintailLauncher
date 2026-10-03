{
  lib,
  rustPlatform,
  cargo-tauri,
  fetchPnpmDeps,
  pnpmConfigHook,
  pnpm,
  nodejs,
  pkg-config,
  protobuf,
  perl,
  makeWrapper,
  wrapGAppsHook4,
  steam-run,
  openssl,
  webkitgtk_4_1,
  libsoup_3,
  glib-networking,
  librsvg,
  libayatana-appindicator,
  cacert,
}:

rustPlatform.buildRustPackage (finalAttrs: {
  pname = "twintaillauncher";
  version = (lib.importJSON ../package.json).version;

  src = lib.cleanSource ../.;

  cargoRoot = "src-tauri";
  buildAndTestSubdir = finalAttrs.cargoRoot;
  cargoHash = "sha256-Nm9jiHaI5frTBIGKe/gagZuAPkAeci0KP0eycU91anM=";

  pnpmDeps = fetchPnpmDeps {
    inherit (finalAttrs) pname version src;
    fetcherVersion = 4;
    hash = "sha256-lpqberANUFYlPKn83WeK2H5owlODjxEKxgU8aw1q6to=";
  };

  nativeBuildInputs = [
    cargo-tauri.hook
    pnpmConfigHook
    pnpm
    nodejs
    pkg-config
    protobuf
    perl
    makeWrapper
    wrapGAppsHook4
  ];

  buildInputs = [
    openssl
    webkitgtk_4_1
    libsoup_3
    glib-networking
    librsvg
    libayatana-appindicator
  ];

  postPatch = ''
    substituteInPlace src-tauri/src/utils/mod.rs \
      --replace-fail "fs::copy(&patch, &target).unwrap();" \
                     "let _ = fs::remove_file(&target); fs::copy(&patch, &target).unwrap();"
  '';

  preFixup = ''
    gappsWrapperArgs+=(
      --prefix LD_LIBRARY_PATH : ${lib.makeLibraryPath [ libayatana-appindicator ]}
      --set WEBKIT_DISABLE_DMABUF_RENDERER 1
      --set GDK_BACKEND x11
      --set WEBKIT_DISABLE_COMPOSITING_MODE 1
      --set SSL_CERT_FILE "${cacert}/etc/ssl/certs/ca-bundle.crt"
      --set NIX_SSL_CERT_FILE "${cacert}/etc/ssl/certs/ca-bundle.crt"
      --set GTK_IM_MODULE simple
    )
  '';

  postFixup = ''
    mv $out/bin/twintaillauncher $out/bin/.twintaillauncher-real
    
    cat > $out/bin/twintaillauncher <<EOF
    #!/bin/sh
    exec ${steam-run}/bin/steam-run $out/bin/.twintaillauncher-real "\$@"
    EOF
    
    chmod +x $out/bin/twintaillauncher
  '';

  meta = {
    description = "Multi-platform launcher for anime games";
    homepage = "https://github.com/TwintailTeam/TwintailLauncher";
    license = lib.licenses.gpl3Only;
    platforms = [ "x86_64-linux" ];
    mainProgram = "twintaillauncher";
  };
})
