# jack-autoconnect

Automatically connect jack clients when they appear.

## Installation

### Arch Linux

Run:
```
cd pkgbuild
makepkg -sif
```

Or simply download the PKGBUILD directly and build build.

## Usage

Simply run:
```
/usr/bin/jack-autoconnect
```

It is recommended to run the application in the background in order to keep it
running all the time.

## Configuration

After running the application once the mappings can be configured in the
configuration file located in `~/.config/jack-autoconnect/config.json`.
