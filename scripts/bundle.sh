#!/bin/bash
# สร้าง bundle สำหรับแจกจ่าย — คัดลอก exe + libpdfium.so
set -e
cargo build --release
cp target/release/thaiestimate ./thaiestimate
cp /home/linuxbrew/.linuxbrew/lib/python3.14/site-packages/pypdfium2_raw/libpdfium.so ./libpdfium.so
echo "Bundle ready: ./thaiestimate + ./libpdfium.so"
