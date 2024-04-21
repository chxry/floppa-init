#!/bin/sh
set -e
wget https://dl-cdn.alpinelinux.org/alpine/v3.19/releases/x86_64/alpine-virt-3.19.1-x86_64.iso -O alpine.iso -nc
qemu-img create -f raw alpine.raw 8G
qemu-system-x86_64 -m 1024 -nic user -boot d -cdrom alpine.iso -drive format=raw,file=alpine.raw -enable-kvm
