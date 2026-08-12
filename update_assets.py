import subprocess
import sys
from tabnanny import check
#def download_latest_bootx64 ():
    #subprocess.run(["sh", "-c", "curl -L https://github.com/pbatard/uefi-ntfs/releases/download/v2.8/bootx64_signed.efi -o bootx64_signed_v2.8.efi --output-dir /home/pc/proj/WINUBM/src/assets"])

#def download_latest_ntfs64 ():
    #subprocess.run(["sh", "-c", "curl -L https://github.com/pbatard/ntfs-3g/releases/download/1.9/ntfs_x64_signed.efi -o ntfs_x64_signed_v1.9.efi --output-dir /home/pc/proj/WINUBM/src/assets"])

result = subprocess.run(
    ["sh", "-c", "curl -s https://api.github.com/repos/pbatard/uefi-ntfs/releases/latest | jq -r '.tag_name'"],
    capture_output=True,
    text=True,
    check=True
)

result_3g = subprocess.run(

    ["sh", "-c", "curl -s https://api.github.com/repos/pbatard/ntfs-3g/releases/latest | jq -r '.tag_name'"],
    capture_output=True,
    text=True,
    check=True
)

latest_version = result.stdout.strip()
latest_version_ntfs_x64 = float(result_3g.stdout.strip())

def download_latest_bootx64 ():
    subprocess.run(["sh", "-c", f"curl -L https://github.com/pbatard/uefi-ntfs/releases/download/v{latest_version}/bootx64_signed.efi -o bootx64_signed_v{latest_version}.efi --output-dir /home/pc/proj/WINUBM/src/assets"])

def download_latest_ntfs64 ():
    subprocess.run(["sh", "-c", f"curl -L https://github.com/pbatard/ntfs-3g/releases/download/{latest_version_ntfs_x64}/ntfs_x64_signed.efi -o ntfs_x64_signed_v{latest_version_ntfs_x64}.efi --output-dir /home/pc/proj/WINUBM/src/assets"])

latest_version = float(latest_version[1:4])
#print(latest_version_ntfs_x64)
#print(latest_version)
result2 = subprocess.run(
    ["ls"],
    cwd="/home/pc/WINUBM/src/assets",
    capture_output=True,
    text=True,
    check=True
)

curr_version = result2.stdout.strip();

curr_version_bootx64 = float((curr_version[16:19]))
curr_version_ntfs_x64 = float((curr_version[41:44]))


def check_version_ntfs():
     if curr_version_ntfs_x64 == latest_version_ntfs_x64:
         print("latest version")

     else:
        download_latest_ntfs64()


def check_version_bootx64():
    if curr_version_bootx64 == latest_version:
        print("latest version")
    else:
        download_latest_bootx64()


print("==============================================================================================")
print("checking status:")
check_version_ntfs()
check_version_bootx64()
