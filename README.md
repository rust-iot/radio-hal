# Rust IoT Radio Abstraction(s)

An [embedded-hal](https://github.com/rust-embedded/embedded-hal) like abstraction for digital radio devices, this is intended to provide a common basis for implementing packet radio drivers, and for extension to support 802.15.4 / BLE etc. in the hope that we can construct embedded network stacks using this common abstraction.

For similar examples, check out:
- Riot-OS 
  - [netdev.h](https://github.com/RIOT-OS/RIOT/blob/master/drivers/include/net/netdev.h)
  - [ieee802154.h](https://github.com/RIOT-OS/RIOT/blob/master/drivers/include/net/netdev/ieee802154.h)
    [netdev_ieee802154.c](https://github.com/RIOT-OS/RIOT/blob/master/drivers/netdev_ieee802154/netdev_ieee802154.c)
- Contiki-OS
  - TODO

## Status

[![GitHub tag](https://img.shields.io/github/tag/ryankurte/rust-radio.svg)](https://github.com/ryankurte/rust-radio)
[![Build Status](https://travis-ci.com/ryankurte/rust-radio.svg?token=s4CML2iJ2hd54vvqz5FP&branch=master)](https://travis-ci.com/ryankurte/rust-radio)
[![Crates.io](https://img.shields.io/crates/v/radio.svg)](https://crates.io/crates/radio)
[![Docs.rs](https://docs.rs/radio/badge.svg)](https://docs.rs/radio)

[Open Issues](https://github.com/ryankurte/rust-radio/issues)

** Work In Progress **

Features:

- [x] Transmit
- [x] Receive
- [x] Fetch RSSI
- [x] Register Access
- [ ] Configuration
- [ ] 802.15.4
