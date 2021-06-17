---
title: "Nmstate: Convert temporary iproute network configuration to persist"
date: 2021-06-10T16:20:06+08:00
---

The network configuration done by [iproute][iproute_url] will be removed
after reboot. People are using they own scripts or service to persist their
network configurations.

Starting version 1.0, [Nmstate][nmstate_url] supports converting network
configuration created by iproute to persist with the help of
[Nispor][nispor_url] and [NetworkManager][nm_url]:
 * Nispor query current network configurations from kernel.
 * Nmstate instruct NetworkManager to save the network configure in persistent
   way.

Only a single command required:

```bash
sudo nmstatectl show | sudo nmstatectl apply -
```

Please note: bond in "active_backup" mode with `fail_over_mac=active`
option does not allow MAC address been set explicitly, you need to remove
it manually before asking nmstate to apply it:

```bash
sudo nmstatectl show bond0 > bond0.yml
sed -i -e '/mac-address/d' bond0.yml
sudo nmstatectl apply bond0.yml
```

## Example

Assume you create a bond and hope to persist it.

```bash
sudo ip link add bond0 type bond
echo 'active-backup' | sudo tee /sys/class/net/bond0/bonding/mode
sudo ip link set eth1 down
sudo ip link set eth2 down
sudo ip link set eth1 master bond0
sudo ip link set eth2 master bond0
sudo ip link set eth1 up
sudo ip link set eth2 up
sudo ip link set bond0 up
sudo ip addr add 192.0.2.251/24 dev bond0
sudo ip route add default via 192.0.2.1 dev bond0 table 100
sudo ip rule add from 203.0.113.0/24 lookup 100
sudo nmstatectl show bond0,eth1,eth2 | sudo nmstatectl apply -
```

Now you get the configuration of bond0 and its ports saved in NetworkManager:

```bash
[fge@el8 ~]$ sudo nmcli connection show
NAME    UUID                                  TYPE      DEVICE
bond0   c0d0b060-ac09-40b1-bdec-65f3ac9ecfb6  bond      bond0
eth1    23893461-2227-474c-935b-916c5d94e7ae  veth      eth1
eth2    6c60a1a4-4d40-4adf-b129-fcf5af0163fd  veth      eth2
```

[iproute_url]: https://git.kernel.org/pub/scm/network/iproute2/iproute2.git
[nispor_url]: https://github.com/nispor/nispor
[nm_url]: https://networkmanager.dev/
[nmstate_url]: https://nmstate.io/
