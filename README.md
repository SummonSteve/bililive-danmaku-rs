# Bililive-danmu-rs

Bilibili直播弹幕ws协议rust实现

## build

due to regex

    #![cfg_attr(feature = "pattern", feature(pattern))]

can't be used on the stable release channel

add `cargo +nightly` toolchain

    rustup toolchain install nightly
    cargo +nightly build
