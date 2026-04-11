use console;
use dialoguer::{Confirm, Select};
use fatfs;
use gpt;
use indicatif::{ProgressBar, ProgressStyle};
use nix::libc::ioctl;
use nix::mount::MsFlags;
use std::io::Read;
use std::io::Seek;
use std::io::Write;
use std::os::unix::io::AsRawFd;
use std::process::Command;
use std::{
    fs,
    path::{Path, PathBuf},
};

// embeding assests to our executable
static BOOTX64: &[u8] = include_bytes!("assets/bootx64_signed.efi");
static NTFS_DRIVER: &[u8] = include_bytes!("assets/ntfs_x64_signed.efi");
// constants for speaking to ioctl-s

const SIMB: &str = r#"
██╗    ██╗██╗███╗   ██╗██╗   ██╗██████╗ ███╗   ███╗
██║    ██║██║████╗  ██║██║   ██║██╔══██╗████╗ ████║
██║ █╗ ██║██║██╔██╗ ██║██║   ██║██████╔╝██╔████╔██║
██║███╗██║██║██║╚██╗██║██║   ██║██╔══██╗██║╚██╔╝██║
╚███╔███╔╝██║██║ ╚████║╚██████╔╝██████╔╝██║ ╚═╝ ██║
 ╚══╝╚══╝ ╚═╝╚═╝  ╚═══╝ ╚═════╝ ╚═════╝ ╚═╝     ╚═╝
===================================================
"#;

const LOOP_CTL_GET_FREE: u64 = 0x4C82;
const LOOP_SET_FD: u64 = 0x4C00;
const LOOP_CLR_FD: u64 = 0x4C01;

struct MountIso {
    mountpoint: std::path::PathBuf,
    loop_device: String,
}

impl MountIso {
    fn new(iso_path: &Path) -> Result<MountIso, Box<dyn std::error::Error>> {
        let loop_ctl = fs::File::open("/dev/loop-control")?;

        let iso_file = fs::File::open(iso_path)?;

        let free_loop = unsafe { ioctl(loop_ctl.as_raw_fd(), LOOP_CTL_GET_FREE) };

        let loop_device = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(format!("/dev/loop{}", free_loop))?;

        unsafe { ioctl(loop_device.as_raw_fd(), LOOP_SET_FD, iso_file.as_raw_fd()) };

        fs::create_dir_all("/mnt/winubm/iso")?;
        nix::mount::mount(
            Some(format!("/dev/loop{}", free_loop).as_str()),
            "/mnt/winubm/iso",
            Some("udf"),
            MsFlags::MS_RDONLY,
            None::<&str>,
        )?;

        Ok(MountIso {
            mountpoint: PathBuf::from("/mnt/winubm/iso"),
            loop_device: format!("/dev/loop{}", free_loop),
        })
    }
}

impl Drop for MountIso {
    fn drop(&mut self) {
        nix::mount::umount(&self.mountpoint).ok();
        let loop_file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&self.loop_device)
            .ok();
        if let Some(f) = loop_file {
            unsafe { ioctl(f.as_raw_fd(), LOOP_CLR_FD) };
        }
        fs::remove_dir_all(&self.mountpoint).ok();
    }
}

struct MountedNtfs {
    mountpoint: PathBuf,
}

impl MountedNtfs {
    fn new(partition: &str) -> Result<MountedNtfs, Box<dyn std::error::Error>> {
        let mountpoint = PathBuf::from("/mnt/winubm/ntfs");
        fs::create_dir_all(&mountpoint)?;

        // nix::mount::mount(
        //     Some(partition),
        //     &mountpoint,
        //     Some("ntfs"),
        //     MsFlags::empty(),
        //     None::<&str>,
        // )?;

        std::process::Command::new("ntfs-3g")
            .args([partition, "/mnt/winubm/ntfs"])
            .output()?;
        Ok(MountedNtfs {
            mountpoint: mountpoint,
        })
    }
}
impl Drop for MountedNtfs {
    fn drop(&mut self) {
        nix::mount::umount(&self.mountpoint).ok();
        fs::remove_dir_all(&self.mountpoint).ok();
    }
}

fn total_size(path: &Path) -> u64 {
    let mut total_count: u64 = 0;
    for entry in fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let file_type = entry.file_type().unwrap();
        if file_type.is_file() {
            total_count += entry.metadata().unwrap().len();
        } else if file_type.is_dir() {
            total_count += total_size(&entry.path());
        }
    }
    total_count
}

fn copy_file(src: &Path, dst: &Path, bar: &ProgressBar) -> Result<(), Box<dyn std::error::Error>> {
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_file() {
            let mut src_file = fs::File::open(entry.path())?;
            let mut dst_file = fs::File::create(dst.join(entry.file_name()))?;
            let mut buf = vec![0u8; 1024 * 1024];
            let mut accumulated = 0u64;
            loop {
                let n = src_file.read(&mut buf)?;
                if n == 0 {
                    break;
                }
                dst_file.write_all(&buf[..n])?;
                accumulated += n as u64;
                if accumulated >= 8 * 1024 * 1024 {
                    bar.inc(accumulated);
                    accumulated = 0;
                }
            }
            bar.inc(accumulated);
        } else if file_type.is_dir() {
            let new_dst = dst.join(entry.file_name());
            fs::create_dir_all(&new_dst)?;
            copy_file(&entry.path(), &new_dst, bar)?;
        }
    }
    Ok(())
}

fn main() {
    println!("{}", console::style(SIMB).green().bold());
    let mut removable_devices: Vec<String> = Vec::new();
    for device in fs::read_dir("/sys/block/").unwrap() {
        let device = device.unwrap();
        let device_name = device.file_name();
        let removable_device = format!("/sys/block/{}/removable", device_name.to_str().unwrap());

        if fs::read_to_string(&removable_device).unwrap().trim() == "1" {
            removable_devices.push(device_name.into_string().expect("failed to read line"));
        }
    }

    let selection = match Select::new()
        .with_prompt("List of available Usb devices")
        .items(&removable_devices)
        .interact()
    {
        Ok(s) => s,
        Err(_) => {
            eprintln!("{}", console::style("cant find drive").red().bold());
            std::process::exit(1);
        }
    };

    let selected_device = format!("/dev/{}", removable_devices[selection]);
    let win_partition = format!("/dev/{}1", removable_devices[selection]);
    let efi_partition = format!("/dev/{}2", removable_devices[selection]);
    let size_selected_device = format!("/sys/block/{}/size", removable_devices[selection]);
    let blocks: u64 = fs::read_to_string(&size_selected_device)
        .unwrap()
        .trim()
        .parse()
        .unwrap();

    let size_gb = (blocks * 512) as f64 / (1024.0 * 1024.0 * 1024.0);

    println!("{:.1} GB", size_gb);

    println!(
        "{}",
        console::style("Attention! Selected Drive will be Formatted. Are you Sure?")
            .red()
            .bold()
    );
    let confirmation = Confirm::new()
        .with_prompt("Do you want to continue?")
        .interact()
        .unwrap();

    if confirmation {
        println!(
            "{}",
            console::style("Attention! starting  creation  of gpt table")
                .red()
                .bold()
        );
        let iso = dialoguer::Input::<String>::new()
            .with_prompt("Provide absolute path for ISO")
            .interact()
            .unwrap();
        let _iso = MountIso::new(Path::new(&iso)).unwrap();

        let mut file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&selected_device)
            .expect("failed ");
        println!("writing metadata for gpt");

        let mbr = gpt::mbr::ProtectiveMBR::with_lb_size(
            u32::try_from(blocks - 1).unwrap_or(0xFF_FF_FF_FF),
        );
        mbr.overwrite_lba0(&mut file).expect("failed to write MBR");

        let mut gdisk = gpt::GptConfig::default()
            .writable(true)
            .logical_block_size(gpt::disk::LogicalBlockSize::Lb512)
            .create_from_device(file, None)
            .expect("failed to read line");
        println!(
            "{}",
            console::style("gpt metadata written successfully")
                .green()
                .bold()
        );

        println!("start partitiong process:");

        gdisk
            .add_partition(
                "windows",
                (blocks - 2048 - 33) * 512 - 64 * 1024 * 1024,
                gpt::partition_types::BASIC,
                0,
                None,
            )
            .unwrap();
        gdisk
            .add_partition("EFI", 64 * 1024 * 1024, gpt::partition_types::EFI, 0, None)
            .unwrap();
        gdisk.write().expect("failed to write data");
        println!("{}", console::style("partitins created ").green().bold());

        std::thread::sleep(std::time::Duration::from_millis(500));

        let mut file_partition = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&efi_partition)
            .expect("failed to load device");

        println!(
            "{}",
            console::style("writing fat32 filesystem to create partition for bootloader ")
                .green()
                .bold()
        );

        fatfs::format_volume(
            &mut file_partition,
            fatfs::FormatVolumeOptions::new().fat_type(fatfs::FatType::Fat32),
        )
        .expect("failed to format partition");
        println!("{}", console::style("done").green().bold());

        println!(
            "{}",
            console::style("writing ntfs filesystem to create iso compatible partition")
                .green()
                .bold()
        );

        Command::new("mkfs.ntfs")
            .args(["-f", win_partition.as_str()])
            .output()
            .expect("failed");
        match file_partition.seek(std::io::SeekFrom::Start(0)) {
            Ok(_) => {}
            Err(err) => {
                eprintln!("could not seek {}", err)
            }
        };
        println!("{}", console::style("done").green().bold());

        println!(
            "{}",
            console::style("writing uefi bootloader to fat32 paritition")
                .green()
                .bold()
        );
        println!("partitioning done ");

        let fs = fatfs::FileSystem::new(&mut file_partition, fatfs::FsOptions::new()).unwrap();
        let root_dir = fs.root_dir();
        match root_dir.create_dir("EFI") {
            Ok(_) => {}
            Err(err) => eprintln!("could not create directory {}", err),
        };

        match root_dir.create_dir("EFI/BOOT") {
            Ok(_) => {}
            Err(err) => eprintln!("could not create directory {}", err),
        };
        let mut file = root_dir.create_file("EFI/BOOT/bootx64.efi").unwrap();
        match file.write_all(BOOTX64) {
            Ok(_) => {}
            Err(err) => eprint!("could not write {}", err),
        };

        match root_dir.create_dir("efi") {
            Ok(_) => {}
            Err(err) => eprintln!("could not create dir {}", err),
        };
        match root_dir.create_dir("efi/rufus") {
            Ok(_) => {}
            Err(err) => eprintln!("could not create dir {}", err),
        }
        let mut ntfs_file = root_dir.create_file("efi/rufus/ntfs_x64.efi").unwrap();
        match ntfs_file.write_all(NTFS_DRIVER) {
            Ok(_) => {}
            Err(err) => eprintln!("could not write data {}", err),
        }
        println!("{}", console::style("done").green().bold());

        let _ntfs = MountedNtfs::new(&win_partition).unwrap();

        let total = total_size(Path::new("/mnt/winubm/iso"));
        let bar = ProgressBar::new(total);
        bar.set_style(
            ProgressStyle::default_bar()
                .template("{wide_bar} {bytes}/{total_bytes} {binary_bytes_per_sec} ({eta})")
                .unwrap()
                .progress_chars("█░"),
        );

        println!(
            "{}",
            console::style("consStarting Iso writing this process might take a while)")
                .green()
                .bold()
        );
        match copy_file(
            Path::new("/mnt/winubm/iso"),
            Path::new("/mnt/winubm/ntfs"),
            &bar,
        ) {
            Ok(_) => {}
            Err(err) => eprintln!("could not copy files {}", err),
        };
        println!("{}", console::style("Done").green().bold());
        println!(
            "{}",
            console::style("Unmounting Please wait ...").green().bold()
        );
    } else {
        println!(
            "{}",
            console::style("As per your  request program will be Terminated")
                .red()
                .bold()
        );
    }
}
