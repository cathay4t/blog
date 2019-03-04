---
title: "DRAFT/WIP: IPv6 Automatic Configuration in Linux"
date: 2019-02-22T23:55:13+08:00
draft: false
---
## IPv4 vs IPv6 on Automatic Configuration

In world of IPv4, DHCPv4 provides a perfect way for IPv4 automatic
configuration including IP address, DNS and routing entires.
In world of IPv6, above works has been split into types:

 * Stateless automatic configuration --
   [IPv6 Router Advertisement(IPv6-RA)][rfc-ipv6-ra].

 * Stateful automatic configuration -- [DHCPv6][rfc-dhcpv6].

The IPv6-RA is designed to provides fundamental network configuration to host
with minimum cost.
The DHCPv6 is designed to provides additional and extensive network
configuration.

Since DHCPv6 [does not provides routing configuration][ietf-dhcpv6-route],
IPv6-RA is mandatory in order to serve the automatic IPv6 configuration.

The brief difference between IPv6-RA and DHCPv6 are:
```
    |            |  IPv6-RA |   DHCPv6  |
    +------------+----------+-----------+
    | IP Address |  /64     | /128      |
    | Routing    | Yes      | No        |
    | DNS        | Yes      | Yes       |
    | NTP        | No       | Yes       |
```

 * IP Address configuration
    * IPv6-RA: Host use [SLAAC][rfc-slaac] to generate IP address from
      router provided /64 prefix.
    * DHCPv6: DHCPv6 server provides hosts a /128 address which could be used
      to create the DNS AAAA entry for the acknowledged host.

 * DNS Configuration
    * IPv6-RA: Host use [IPv6-RA DNS RA options][rfc-ipv6-ra-dns] to get DNS
      configuration from route.
    * DHCPv6: DHCP server provides the DNS configuration to hosts.

 * Routing
    * IPv6-RA: Host use Router Advertisements
    * DHCPv6: [Not standardized][ietf-dhcpv6-route], hence DHCPv6 should not
      provide routing information.

 * NTP:
    * IPv6-RA: Do not provide NTP time server configuration.
    * DHCPv6: DHCP server [provides][rfc-dhcpv6-ntp] the NTP configuration to
      hosts.

### Server side configuration: dnsmasq

The `dnsmasq` project provide IPv6-RA and DHCPv6 with simple configuration for
common use cases.

For more detail, please refer to manpage of `dnsmasq`.
A quick example would be:

```conf
# Enable IPv6-RA
enable-ra
# Use the IPv6 prefix on eth1 interaface, and 0x1 to 0xff as DHCPv6 range.
dhcp-range=tag:eth1,::1,::ff,constructor:eth1,ra-names,64,48h
```

Above configuration are using `ra-names` method for DHCP range, as a result,
the host will get:
 * A /64 address generated using [SLAAC][rfc-slaac] algorithm. Ignore the
   DHCPv6 range setting.
 * A /128 address chose by DHCPv6 server from above DHCPv6 range.
   And this address if pingable, will be the DNS AAAA entry for the host.
 * The DNS options and defaults routes get from IPv6-RA.
   Host might use DHCPv6 for DNS option if [DNS RA options][rfc-ipv6-ra-dns]
   is not supported.

Example output on the host(The DHCPv6 server has the `2001:db8:1::1` address):
```
[fge@fed ~]$ ip -6 addr show dev  ens10
3: ens10: <BROADCAST,MULTICAST,UP,LOWER_UP> mtu 1280 state UP qlen 1000
    inet6 2001:db8:1::1f/128 scope global dynamic noprefixroute
       valid_lft 6932sec preferred_lft 6632sec
    inet6 2001:db8:1:0:415b:bc93:c47b:76c8/64 scope global dynamic noprefixroute
       valid_lft 172754sec preferred_lft 172754sec
    inet6 fe80::db63:c3ff:b814:f4e2/64 scope link noprefixroute
       valid_lft forever preferred_lft forever

[fge@fed ~]$ ip -6 route show dev ens10
2001:db8:1::1f proto kernel metric 102 pref medium
2001:db8:1::/64 proto ra metric 102 pref medium
fe80::/64 proto kernel metric 102 pref medium
default via fe80::be2b:a3e2:2b5a:2e7d proto ra metric 102 pref medium

[fge@fed ~]$ cat /etc/resolv.conf |grep :
nameserver fe80::be2b:a3e2:2b5a:2e7d%ens10
nameserver 2001:db8:1::1

[fge@fed ~]$ host -t AAAA fed 2001:db8:1::1
Using domain server:
Name: 2001:db8:1::1
Address: 2001:db8:1::1#53
Aliases:

fed has IPv6 address 2001:db8:1::1f
```

### Server side configuration: radvd

The `radvd` provide feature-rich support of IPv6-RA and allowing you to do
those thins which `dnsmasq` does not support yet:

 * Advertise more routes.
 * Customize IPv6-RA options.

For more detail, please refer to manpage of `dnsmasq`.

```
interface eth1
{
    AdvSendAdvert on;
    MinRtrAdvInterval 30;
    MaxRtrAdvInterval 100;
    prefix 2001:db8:1::/64 {    # Host will use SLAAC for this prefix
        AdvOnLink on;
        AdvAutonomous on;
        AdvRouterAddr off;
    };
    route ::/0 {};              # Host will get default gateway
    route 2001:db8:f::/64 {};   # Host will get this optional stateless route
    RDNSS 2001:db8:1::1 {};     # Host will get nameserver as 2001:db8:1::1
};
```

### Client side configuration using nmstate

The `nmstate` project provides simple way to set network state.

For example: to enable auto configuration on eth1 for IPv4 and IPv6:

```bash
echo '
---
interfaces:
- name: eth1
  type: ethernet
  state: up
  ipv4:
    dhcp: true
    auto-dns: true
    auto-gateway: true
    auto-routes: true
    enabled: true
  ipv6:
    autoconf: true
    dhcp: true
    auto-dns: true
    auto-gateway: true
    auto-routes: true
    enabled: true
  mtu: 1500
' | sudo nmstatectl set -
```

Then you may use command `nmstatectl  show eth1` to get the current network
state of `eth1`.

### Client side configuration using NetworkManager

With NetworkManager daemon running, the below simple command would enable both
IPv4 and IPv6 automatic configuration on `eth1`:

```bash
sudo nmcli connection add type ethernet \
    ifname eth1 \
    +connection.id auto-eth1 \
    +ipv4.method auto \
    +ipv6.method auto
```

### Client side configuration manually

TODO

[rfc-ipv6-ra]: https://tools.ietf.org/html/rfc4861
[rfc-dhcpv6]: https://tools.ietf.org/html/rfc8415
[rfc-ipv6-ra-dns]: https://tools.ietf.org/html/rfc8106
[rfc-slaac]: https://tools.ietf.org/html/rfc4862
[rfc-dhcpv6-ntp]: https://tools.ietf.org/html/rfc5908
[ietf-dhcpv6-route]: https://datatracker.ietf.org/doc/draft-ietf-mif-dhcpv6-route-option/
