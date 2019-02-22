---
title: "DRAFT/WIP: IPv6 Automatic Configuration in Linux"
date: 2019-02-22T23:55:13+08:00
draft: false
---

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
    * DHCPv6: [DHCP server provides the NTP configuration to
      hosts][rfc-dhcpv6-ntp]




[rfc-ipv6-ra]: https://tools.ietf.org/html/rfc4861
[rfc-dhcpv6]: https://tools.ietf.org/html/rfc8415
[rfc-ipv6-ra-dns]: https://tools.ietf.org/html/rfc8106
[rfc-slaac]: https://tools.ietf.org/html/rfc4862
[rfc-dhcpv6-ntp]: https://tools.ietf.org/html/rfc5908
[ietf-dhcpv6-route]: https://datatracker.ietf.org/doc/draft-ietf-mif-dhcpv6-route-option/
