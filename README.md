[![Build Status](https://travis-ci.org/gdamjan/fb-ask-pass-rs.svg?branch=master)](https://travis-ci.org/gdamjan/fb-ask-pass-rs)


Primary usage: run it from a initcpio hook (on archlinux) to ask for the LUKS passphrase, while showing the firmware picture.
The passphrase is saved in a file (`/crypto_keyfile.bin`) which the `encrypt` hook uses to unlock LUKS volumes.


The kernel parameters I use are:

```
quiet udev.log_priority=3 loglevel=3 vt.global_cursor_default=0 i915.fastboot=1
```

Arch initcpio configuration (`/etc/mkinitcpio.conf`):

```
HOOKS=(base udev autodetect block fb-ask-pass encrypt filesystems)
```

For testing, in a console you can just run:

```
sudo chown `id -u` /dev/fb0
cargo run
```

and it'll show whatever you type on the terminal. It won't work in X11 or wayland.

see also:
- [Arch wiki page for silent boot](https://wiki.archlinux.org/index.php/silent_boot)

# FAQ

## Why Rust?

- It's a learning experience
- Rust compiles to a single binary, depends only on glibc


## How does it work?

- the program reads the firmware image provided by ACPI 5.0 from `/sys/firmware/acpi/bgrt/*` and displays it on the
  framebuffer at the same position (`xoffset`, `yoffset`).
  - afaik, UEFI needs to be enabled. probably quick boot, and full resolution booting too.
- then waits for the user to enter its password, and writes it to a file.
- look in `arch/` to see how to integrate with archlinux's initcpio system.

## What is BGRT?

the TL;DR; it's the image/logo that the BIOS shows when powering up the PC. The ACPI 5.0 standard
allows access to the image by OS. Please read the [kernel documentation about the /sys/firmware/acpi/bgrt interface](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/Documentation/ABI/testing/sysfs-firmware-acpi).


## Why not plymouth?

- Yes, [Plymouth](https://www.freedesktop.org/wiki/Software/Plymouth/) is probably the better solution. It has much
  more features and is much better tested.
- For example, plymouth stays in control for the whole boot process, and gives out control only before X11/lgoin manager takes over. This program instead, leaves the console in static graphic mode with the image, making it fairly useless.
- This is just a very simple experiment.


# TODO

- some kind of password prompt, and some feedback
- see how to integrate with Dracut and/or other distros
- probably by using the [keyring interface](https://gitlab.com/cryptsetup/cryptsetup/blob/master/docs/Keyring.txt)
