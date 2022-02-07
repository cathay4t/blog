---
title: "Nispor: Unified interface for querying network state"
date: 2020-10-13T22:00:14+08:00
---

After battling with Linux Network APIs, I decided to create rust project
providing a `state describing` API for querying linux kernel network state.

The new project is named as [`Nispor`][nispor_url], in the short of `Network
Inspector`.

## Why Nispor

In short, existing solutions is not simple enough:
 * [iproute][ip_route_link]
   The iproute provides json output, but command line output cannot be really
   called as a library considering the risk of changing format or string.

 * [NetworkManager][nm_link]
   NetworkManager are spreading network states among many structs/objects.
   And it is based on dbus interface, you need to refresh the data using
   complex async actions.

 * sysfs
   Regardless the debate on whether sysfs is a API or not, gathering
   network state among sysfs folders is time consuming work.

## What Nispor can provides

 * Pure Rust with promise of memory safe, thread-safe.
 * Rust crate.
 * C binding written in rust
 * Python binding written in Python using the c binding above.
 * Best effort on point-in-time consistence of network state.
 * Command line tool for debugging or scripting.
 * Supporting:
     * IPv4/IPv6 address
     * Bond
     * Linux Bridge
     * Linux Bridge VLAN filtering
     * VLAN
     * VxLAN
     * Route
     * Dummy
     * TUN/TAP
     * Veth
     * VRF(Virtual Routing and Forwarding)
     * SR-IOV
     * MacVlan
     * MacVtap

## How it looks like

The output of `npc iface bond99` would be:

```yaml
---
- name: bond99
  iface_type: Bond
  state: Up
  mtu: 1500
  flags:
    - Broadcast
    - LowerUp
    - Controller
    - Multicast
    - Running
    - Up
  ipv6:
    addresses:
      - address: "fe80::942c:a7ff:fe56:683c"
        prefix_len: 64
        valid_lft: forever
        preferred_lft: forever
  mac_address: "96:2C:A7:56:68:3C"
  bond:
    subordinates:
      - eth1
      - eth2
    mode: balance-rr
    miimon: 0
    updelay: 0
    downdelay: 0
    use_carrier: true
    arp_interval: 0
    arp_all_targets: any
    arp_validate: none
    resend_igmp: 1
    all_subordinates_active: dropped
    packets_per_subordinate: 1
    peer_notif_delay: 0
```

## Future of Nispor

 * Support applying network configure.

## How to Contribute

 * [Simple good first issues of Nispor][nispor_first_issue]
 * Contact `Gris` via IRC of [Libera][web_irc] or email <fge@redhat.com> for
   helps.

[ip_route_link]: https://git.kernel.org/pub/scm/network/iproute2/iproute2.git
[nm_link]: https://wiki.gnome.org/Projects/NetworkManager
[nispor_first_issue]: https://github.com/nispor/nispor/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22
[web_irc]: https://web.libera.chat/
[nispor_url]: https://github.com/nispor/nispor/
