#!/bin/sh
set -e
cargo build
mkdir -p mnt/
sudo partx -a alpine.raw
sudo mount /dev/loop0p3 mnt/
sudo cp --remove-destination target/x86_64-unknown-linux-musl/debug/floppa-init mnt/sbin/init
cd mnt/
cd ..
sudo umount mnt/
sudo losetup -d /dev/loop0
qemu-system-x86_64 -m 1024 -nic user -drive format=raw,file=alpine.raw -enable-kvm
