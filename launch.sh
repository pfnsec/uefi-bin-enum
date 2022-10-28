rm -rf efi/
mkdir -p efi/EFI/BOOT/
cp target/x86_64-unknown-uefi/debug/uefi_app.efi efi/EFI/BOOT/BOOTX64.EFI
cp /usr/share/OVMF/OVMF_VARS.fd .
qemu-system-x86_64 \
    -nodefaults \
    -vga std \
    -machine q35,accel=kvm:tcg \
    -m 128M \
    -drive if=pflash,format=raw,readonly,file=/usr/share/OVMF/OVMF_CODE.fd \
    -drive if=pflash,format=raw,file=OVMF_VARS.fd \
    -drive format=raw,file=fat:rw:efi/ \
    -drive format=raw,file=fat:rw:efi2/ \
    -drive format=raw,file=fat:rw:efi3/ \
    -serial stdio \
    -monitor vc:1024x768
