# Memoria del proyecto - OpenSCQ30 en postmarketOS (epo2)

## Compilación

El binario se compila nativamente en epo2 con musl:

```bash
ssh epo2 "cd /home/edi/openscq30 && cargo build --package openscq30-gui --profile release-fast"
```

El binario resultante va a `/usr/local/bin/openscq30-gui`.

## Instalación en epo2

- **Binario**: `/usr/local/bin/openscq30-gui` (musl aarch64, 38M)
- **Icono scalable**: `/usr/share/icons/hicolor/scalable/apps/com.oppzippy.OpenSCQ30.svg`
- **Iconos por tamaño**: `16x16`, `24x24`, `32x32`, `48x48`, `64x64`, `128x128`, `192x192`, `256x256`
- **Desktop entry**: `/usr/share/applications/com.oppzippy.OpenSCQ30.desktop`

Para actualizar iconos tras cambios:
```bash
ssh epo2 "sudo gtk-update-icon-cache /usr/share/icons/hicolor/"
```

## Lanzar en phosh

```bash
ssh epo2 "WAYLAND_DISPLAY=wayland-0 openscq30-gui"
```

## Cambios realizados (en fork woodyst/OpenSCQ30)

### PR #1: https://github.com/woodyst/OpenSCQ30/pull/1
- **A3135** (Soundcore Motion 300): soporte completo (EQ, adaptive direction, LED brightness, volume, LDAC, voice prompt, auto power-off)
- **A3961** (Soundcore Sport X10): ambient sound modes, equalizer, BassUp preset
- **GUI responsive**: device selection y quick presets se apilan en pantallas <450px
- **Docs postmarketOS**: añadido a docs/build-linux.md

## Notas técnicas

- epo2 usa Alpine Linux con musl, NO glibc. El binario compilado en ia (glibc) no funciona aquí.
- Se necesitan los packages: `rust cargo pkgconfig dbus-dev libxkbcommon-dev git`
- phosh es el compositor Wayland en epo2 (`phoc`)
- El proyecto está clonado en `/home/edi/openscq30/` en epo2
- Repositorio remoto: `origin` -> Oppzippy/OpenSCQ30 (solo lectura para woodyst)
