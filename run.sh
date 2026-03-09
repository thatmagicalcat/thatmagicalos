#!/bin/sh

KERNEL_PATH=$1

echo "Building ISO Image with kernel: $KERNEL_PATH"

mkdir -p isodir/boot/grub
cp $KERNEL_PATH isodir/boot/
cp grub.cfg isodir/boot/grub/

grub-mkrescue -o ros.iso isodir 2> /dev/null

echo "Launching QEMU"
qemu-system-i386 -cdrom ros.iso
