//! Pure numeric-address classification.
//!
//! Registry source: IANA IPv4 and IPv6 Special-Purpose Address Registries,
//! retrieved 2026-09-01; both registries reported last update 2025-10-09.
//! The tables deliberately encode every listed block. More-specific entries
//! win. Registry entries with protocol, translation, documentation, reserved,
//! anycast, benchmarking, discard, or otherwise ambiguous semantics are
//! `SpecialOrUnresolved`; only private/shared/link-local ranges are
//! `LocalOrPrivateScope`. An unlisted address is only *addressed externally*;
//! this makes no reachability, routing, ownership, trust, or privacy claim.

use std::net::IpAddr;

use crate::model::AddressClass;

pub const IANA_REGISTRY_RETRIEVED: &str = "2026-09-01";
pub const IANA_REGISTRY_LAST_UPDATED: &str = "2025-10-09";

#[derive(Clone, Copy, Debug)]
struct Prefix {
    network: u128,
    bits: u8,
    class: AddressClass,
}

const LOCAL: AddressClass = AddressClass::LocalOrPrivateScope;
const SPECIAL: AddressClass = AddressClass::SpecialOrUnresolved;

// Source order is preserved where practical; lookup sorts logically by prefix
// length so exceptions inside broad IANA allocations remain exact.
const IPV4: &[Prefix] = &[
    p4(0x00000000, 8, SPECIAL),
    p4(0x00000000, 32, SPECIAL),
    p4(0x0a000000, 8, LOCAL),
    p4(0x64400000, 10, LOCAL),
    p4(0x7f000000, 8, AddressClass::SameMachineLoopback),
    p4(0xa9fe0000, 16, LOCAL),
    p4(0xac100000, 12, LOCAL),
    p4(0xc0000000, 24, SPECIAL),
    p4(0xc0000000, 29, LOCAL),
    p4(0xc0000008, 32, SPECIAL),
    p4(0xc0000009, 32, SPECIAL),
    p4(0xc000000a, 32, SPECIAL),
    p4(0xc00000aa, 32, SPECIAL),
    p4(0xc00000ab, 32, SPECIAL),
    p4(0xc0000200, 24, SPECIAL),
    p4(0xc01fc400, 24, SPECIAL),
    p4(0xc034c100, 24, SPECIAL),
    p4(0xc0586300, 24, SPECIAL),
    p4(0xc0586302, 32, SPECIAL),
    p4(0xc0a80000, 16, LOCAL),
    p4(0xc0af3000, 24, SPECIAL),
    p4(0xc6120000, 15, SPECIAL),
    p4(0xc6336400, 24, SPECIAL),
    p4(0xcb007100, 24, SPECIAL),
    p4(0xf0000000, 4, SPECIAL),
    p4(0xffffffff, 32, SPECIAL),
];

const IPV6: &[Prefix] = &[
    p6(0, 128, SPECIAL),
    p6(1, 128, AddressClass::SameMachineLoopback),
    p6(0x0000_0000_0000_0000_0000_ffff_0000_0000, 96, SPECIAL),
    p6(0x0064_ff9b_0000_0000_0000_0000_0000_0000, 96, SPECIAL),
    p6(0x0064_ff9b_0001_0000_0000_0000_0000_0000, 48, SPECIAL),
    p6(0x0100_0000_0000_0000_0000_0000_0000_0000, 64, SPECIAL),
    p6(0x0100_0000_0000_0001_0000_0000_0000_0000, 64, SPECIAL),
    p6(0x2001_0000_0000_0000_0000_0000_0000_0000, 23, SPECIAL),
    p6(0x2001_0000_0000_0000_0000_0000_0000_0000, 32, SPECIAL),
    p6(0x2001_0001_0000_0000_0000_0000_0000_0001, 128, SPECIAL),
    p6(0x2001_0001_0000_0000_0000_0000_0000_0002, 128, SPECIAL),
    p6(0x2001_0001_0000_0000_0000_0000_0000_0003, 128, SPECIAL),
    p6(0x2001_0002_0000_0000_0000_0000_0000_0000, 48, SPECIAL),
    p6(0x2001_0003_0000_0000_0000_0000_0000_0000, 32, SPECIAL),
    p6(0x2001_0004_0112_0000_0000_0000_0000_0000, 48, SPECIAL),
    p6(0x2001_0010_0000_0000_0000_0000_0000_0000, 28, SPECIAL),
    p6(0x2001_0020_0000_0000_0000_0000_0000_0000, 28, SPECIAL),
    p6(0x2001_0030_0000_0000_0000_0000_0000_0000, 28, SPECIAL),
    p6(0x2001_0db8_0000_0000_0000_0000_0000_0000, 32, SPECIAL),
    p6(0x2002_0000_0000_0000_0000_0000_0000_0000, 16, SPECIAL),
    p6(0x2620_004f_8000_0000_0000_0000_0000_0000, 48, SPECIAL),
    p6(0x3fff_0000_0000_0000_0000_0000_0000_0000, 20, SPECIAL),
    p6(0x5f00_0000_0000_0000_0000_0000_0000_0000, 16, SPECIAL),
    p6(0xfc00_0000_0000_0000_0000_0000_0000_0000, 7, LOCAL),
    p6(0xfe80_0000_0000_0000_0000_0000_0000_0000, 10, LOCAL),
];

const fn p4(network: u32, bits: u8, class: AddressClass) -> Prefix {
    Prefix {
        network: network as u128,
        bits,
        class,
    }
}

const fn p6(network: u128, bits: u8, class: AddressClass) -> Prefix {
    Prefix {
        network,
        bits,
        class,
    }
}

pub fn classify(address: IpAddr) -> AddressClass {
    match address {
        IpAddr::V4(value) => classify_value(u32::from(value) as u128, 32, IPV4),
        IpAddr::V6(value) => classify_value(u128::from(value), 128, IPV6),
    }
}

pub fn classify_numeric(text: &str) -> AddressClass {
    text.parse::<IpAddr>()
        .map(classify)
        .unwrap_or(AddressClass::SpecialOrUnresolved)
}

fn classify_value(value: u128, width: u8, table: &[Prefix]) -> AddressClass {
    table
        .iter()
        .filter(|entry| contains(value, width, **entry))
        .max_by_key(|entry| entry.bits)
        .map_or(AddressClass::ExternallyAddressed, |entry| entry.class)
}

fn contains(value: u128, width: u8, prefix: Prefix) -> bool {
    if prefix.bits == 0 {
        return true;
    }
    let shift = width - prefix.bits;
    (value >> shift) == (prefix.network >> shift)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, Ipv6Addr};

    #[test]
    fn registry_identity_is_pinned() {
        assert_eq!(IANA_REGISTRY_RETRIEVED, "2026-09-01");
        assert_eq!(IANA_REGISTRY_LAST_UPDATED, "2025-10-09");
    }

    #[test]
    fn every_ipv4_registry_range_boundary_is_classified() {
        for prefix in IPV4 {
            let (first, last) = edges(*prefix, 32);
            assert_ne!(
                classify(IpAddr::V4(Ipv4Addr::from(first as u32))),
                AddressClass::ExternallyAddressed
            );
            assert_ne!(
                classify(IpAddr::V4(Ipv4Addr::from(last as u32))),
                AddressClass::ExternallyAddressed
            );
        }
    }

    #[test]
    fn every_ipv6_registry_range_boundary_is_classified() {
        for prefix in IPV6 {
            let (first, last) = edges(*prefix, 128);
            assert_ne!(
                classify(IpAddr::V6(Ipv6Addr::from(first))),
                AddressClass::ExternallyAddressed
            );
            assert_ne!(
                classify(IpAddr::V6(Ipv6Addr::from(last))),
                AddressClass::ExternallyAddressed
            );
        }
    }

    #[test]
    fn loopback_precedes_broad_special_ranges() {
        assert_eq!(
            classify("127.255.255.255".parse().unwrap()),
            AddressClass::SameMachineLoopback
        );
        assert_eq!(
            classify("::1".parse().unwrap()),
            AddressClass::SameMachineLoopback
        );
    }

    #[test]
    fn ordinary_and_invalid_addresses_are_qualified() {
        assert_eq!(
            classify("8.8.8.8".parse().unwrap()),
            AddressClass::ExternallyAddressed
        );
        assert_eq!(
            classify("2606:4700:4700::1111".parse().unwrap()),
            AddressClass::ExternallyAddressed
        );
        assert_eq!(
            classify_numeric("not-an-address"),
            AddressClass::SpecialOrUnresolved
        );
    }

    #[test]
    fn private_and_ambiguous_special_ranges_remain_distinct() {
        assert_eq!(
            classify("10.0.0.1".parse().unwrap()),
            AddressClass::LocalOrPrivateScope
        );
        assert_eq!(
            classify("fc00::1".parse().unwrap()),
            AddressClass::LocalOrPrivateScope
        );
        assert_eq!(
            classify("192.0.2.1".parse().unwrap()),
            AddressClass::SpecialOrUnresolved
        );
        assert_eq!(
            classify("2001:db8::1".parse().unwrap()),
            AddressClass::SpecialOrUnresolved
        );
    }

    fn edges(prefix: Prefix, width: u8) -> (u128, u128) {
        let host_bits = width - prefix.bits;
        let mask = if host_bits == 128 {
            u128::MAX
        } else if host_bits == 0 {
            0
        } else {
            (1_u128 << host_bits) - 1
        };
        let first = prefix.network & !mask;
        (first, first | mask)
    }
}
