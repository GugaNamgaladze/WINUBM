A Windows USB creator for Linux, written in Rust. Create bootable Windows installation USBs from ISO files.
Features

GPT partition table creation
Dual partition setup (NTFS + FAT32) for full UEFI boot support
UEFI:NTFS bootloader embedded — no external downloads needed
Secure Boot compatible (signed binaries)
Real-time progress bar with speed and ETA
Automatic ISO mounting via loop device
No 4GB file size limit — supports modern Windows 11 ISOs
Minimal binary size (~1.3MB)

# Linux kernel 5.15+ (for ntfs3 driver) but works with ntfs-3g 
# works on wsl but since it does not supports ntfs3 kernel drvier you have to use ntfs-3g and wsl requires usbipd-win to attach usb to your wsl2 instance 

Dependencies
# ntfs-3g   
Install on your distro:
# Arch
sudo pacman -S ntfs-3g 

# Ubuntu/Debian
sudo apt install ntfs-3g 

# Fedora
sudo dnf install ntfs-3g

# build from source 
# https://github.com/GugaNamgaladze/WINUBM.git
# cd WINUBM
# cargo build --release
# run binary: 
# sudo target/release/winubm

# Acknowledgements
# [UEFI:NTFS](https://github.com/pbatard/uefi-ntfs)
# This tool uses this amazing project! Thanks for Contributors.
