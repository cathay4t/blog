---
title: "Query Ethtool via Netlink Kernel Interface"
date: 2021-04-15T13:15:25+08:00
---

Besides old `ioctl` interface of ethtool, the linux kernel also provides
[netlink interface][kernel_ethtool_doc] for controlling ethtool.

The existing documents does not serve me well on getting [my
work][nispor_pr_ethtool] done, so I hope this blog could help others
by demonstrate on querying ethtool PAUSE informations via netlink.

The [libnl][libnl] has already provided many helper functions
regarding communicating through generic netlink interface. But I think
the understanding of every bits in netlink message is essential skill set
of a good network developer.

## Package structure

The ethtool is using `NETLINK_GENERIC` family of netlink with below
data structure on each request and reply:

```
+-------------------------------------------------------------------+
|                                                                   |
|                   Netlink Header(struct nlmsghdr)                 |
|                                                                   |
+-------------------------------------------------------------------+
|                                                                   |
|               Generic Netlink Header(struct genlmsghdr)           |
|                                                                   |
+-------------------------------------------------------------------+
|                                                                   |
|                       Ethtool Attributes                          |
|                                                                   |
+-------------------------------------------------------------------+
                            .
                            .
                            .
```

### Netlink Header

The netlink header `struct nlmsghdr` is holding `nlmsg_type` to differentiate
which kernel module is responsible for processing this netlink request(the
[wifi nl80211][wifi_netlink] is also using `NETLINK_GENERIC`).
In libnl and kernel code of generic netlink, this `nlmsg_type` is also called
as Family ID.

The ethtool netlink kernel module is required to register itself to get
a family id/message type dynamically allocated.

The user space program need to use the reserved `GENL_ID_CTRL` netlink message
type/family id to find out the family ID they are interested. I will explain
that in follow up sections.

The netlink header also holds `flags` which is useful for ethtool netlink
communication. The most commonly used flags for querying are:
 * `NLM_F_REQUEST`  -- Mandatory for all request sending from user to kernel.
 * `NLM_F_DUMP` -- Dump the information for all interfaces.

### Generic Netlink Header
The generic netlink header `struct genlmsghdr` is holding `cmd` to
differentiate which function is responsible for processing this netlink
request.

For example, the `CTRL_CMD_GETFAMILY` is used to retrieve the netlink message
type/family ID.

### Ethtool Attributes

The contents of ethtool netlink package after the generic netlink header
is wrapped into nested(`NLA_F_NESTED`) netlink attribute(`struct nlattr`)
including both the ethtool headers and contents.

There could be multiple ethtool header included in single netlink
request/reply. They are using these `nla_type`:

 * `ETHTOOL_A_HEADER_DEV_INDEX` -- For interface index
 * `ETHTOOL_A_HEADER_DEV_NAME`  -- For interface name
 * `ETHTOOL_A_HEADER_FLAGS`     -- For flags

For example, if you would like to request the ethtool PAUSE information of
`sim0`. You need to include this netlink attribute after the generic netlink
header:
 * type: `ETHTOOL_A_HEADER_DEV_NAME`
 * payload: `sim0\0`

Even when you are dumping(`NLM_F_DUMP`) the information, you still
need to include a empty netlink attribute:
 * type: `ETHTOOL_A_PAUSE_HEADER | NLA_F_NESTED`
 * payload: `NULL`


## Workflow

Let me walk you through the whole ethool netlink interface by querying sim0's
ethtool PAUSE option by a [userspace program][nispor_pr_ethtool]:
 * Query the family ID of ethtool.
 * Assemble the netlink message for querying PAUSE option.
 * Send request to netlink socket.
 * Parse the netlink reply message.

Using netdevsim kernel module, you may simulate a network interface with
ethtool PAUSE support via this script:

```bash
sudo modprobe netdevsim
echo '1 1' | sudo tee /sys/bus/netdevsim/new_device
sleep 10
sudo ip link set eni1np1 name sim0
sudo ethtool -A sim0 tx on
sudo ethtool -A sim0 rx on
```

### Query the Family ID of Ethtool

To query the family ID of ethtool, you need:
 * netlink header flag: `NLM_F_REQUEST`
 * netlink header message type: `GENL_ID_CTRL`
 * generic netlink header cmd: `CTRL_CMD_GETFAMILY`
 * generic netlink header version: `CTRL_CMD_GETFAMILY_VERSION`
 * netlink attribute type: `CTRL_ATTR_FAMILY_NAME`
 * netlink attribute payload: `sim0\0`

The message on wire send from userspace to kernel will be:

```
# Netlink Header
0x28 0x00 0x00 0x00         # totol netlink message length
0x10 0x00 0x01 0x00         # 0x10 0x00: `GENL_ID_CTRL`
                            # 0x01 0x00: flags: NLM_F_REQUEST
0x00 0x00 0x00 0x00         # Sequence number
0x00 0x00 0x00 0x00         # Sending process port ID

# Generic Netlink Header
0x03 0x01 0x00 0x00         # 0x03: CTRL_CMD_GETFAMILY
                            # 0x01: CTRL_CMD_GETFAMILY_VERSION
                            # 0x00 0x00: reserved

# Netlink attribute - CTRL_ATTR_FAMILY_NAME
0x14 0x00 0x02 0x00         # 0x14 0x00: lenth of this attribute
                            # 0x02 0x00: CTRL_ATTR_FAMILY_NAME
0x65 0x74 0x68 0x74         # C chars: e t h t
0x6f 0x6f 0x6c 0x00         # C chars: o o l NULL
0x00 0x00 0x00 0x00         # padding
0x00 0x00 0x00 0x00         # padding
```

The kernel will reply with a lot netlink attributes, one of them is:
 * netlink attribute type: `CTRL_ATTR_FAMILY_NAME`
 * netlink attribute payload: 21(just example, might be different in your OS)

The message on wire send from kernel to userspace is to large to explain
in this blog. You may parse it like above.

### Assemble the Netlink Message for Querying PAUSE Option

To query ethtool PAUSE options, you need:
 * netlink header flag: `NLM_F_REQUEST`
 * netlink header message type: 21 (the number you get in previous step)
 * generic netlink header cmd: `ETHTOOL_MSG_PAUSE_GET`
 * generic netlink header version: `ETHTOOL_GENL_VERSION`
 * netlink attribute type: `ETHTOOL_A_PAUSE_HEADER | NLA_F_NESTED`
 * netlink attribute payload: is another netlink attribute
    * netlink attribute type: `ETHTOOL_A_HEADER_DEV_NAME`
    * netlink attribute payload: `sim0\0`

The message on wire send from userspace to kernel will be:

```
# Netlink header
0x24 0x00 0x00 0x00         # length of netlink message
0x15 0x00 0x01 0x00         # 0x15 0x00: family ID 21
                            # 0x01 0x00: flags: NLM_F_REQUEST
0x00 0x00 0x00 0x00         # Sequence number
0x00 0x00 0x00 0x00         # Sending process port ID

# Generic Netlink Header
0x15 0x01 0x00 0x00         # 0x15: `ETHTOOL_MSG_PAUSE_GET`
                            # 0x01: `ETHTOOL_GENL_VERSION`
                            # 0x00 0x00: reserved

# Netlink attribute - ethtool header ETHTOOL_A_PAUSE_HEADER
0x10 0x00 0x01 0x80         # 0x10 0x00: length 16
                            # 0x01 0x80: `ETHTOOL_A_PAUSE_HEADER | NLA_F_NESTED`
# Netlink attribute - ethtool header ETHTOOL_A_HEADER_DEV_NAME
0x09 0x00 0x02 0x00         # 0x09 0x00: length 9 (without padding)
                            # 0x02 0x00: `ETHTOOL_A_HEADER_DEV_NAME`
0x73 0x69 0x6d 0x30         # c char: s i m 0
0x00 0x00 0x00 0x00         # NULL and padding.
```

### Send Request to Netlink Socket.

You should:
 * Bind to `AF_NETLINK` socket with family ID 21(the number you retrieved in
   previous call).

 * Send above netlink message to:
```c
struct sockaddr_nl {
   sa_family_t     nl_family;  /* AF_NETLINK */
   unsigned short  nl_pad;     /* Zero */
   pid_t           nl_pid;     /* Port ID, should be 0 */
   __u32           nl_groups;  /* Multicast groups mask, should be 0 */
};
```

### Parse the Netlink Reply message.

The message on wire reply from kernel to use space will be:

```
# Netlink Header
0x44 0x00 0x00 0x00         # length of this message
0x15 0x00 0x00 0x00         # 0x15 0x00: Family ID 21
                            # flags: 0
0x01 0x00 0x00 0x00         # sequence number, you may ignore it
0x3c 0x05 0x88 0xb2         # Sending process port ID, you may ignore it

# Generic Netlink Header
0x16 0x01 0x00 0x00         # 0x16: `ETHTOOL_MSG_PAUSE_GET_REPLY`
                            # 0x01: `ETHTOOL_GENL_VERSION`
                            # 0x00 0x00: reserved

# Netlink Attributes -- ethtool header
0x18 0x00 0x01 0x80         # 0x18 0x00: lenth 24
                            # 0x01 0x80: `ETHTOOL_A_PAUSE_HEADER | NLA_F_NESTED`

# Netlink Attributes -- ethtool header ETHTOOL_A_HEADER_DEV_INDEX
0x08 0x00 0x01 0x00         # 0x08 0x00: length 8
                            # 0x01 0x00: `ETHTOOL_A_HEADER_DEV_INDEX`
0x0b 0x00 0x00 0x00         # 0x0b 0x00, 0x00, 0x00: interface index 11

# Netlink Attributes -- ethtool header ETHTOOL_A_HEADER_DEV_NAME
0x09 0x00 0x02 0x00         # 0x09 0x00: length 9(without padding)
                            # 0x02 0x00: `ETHTOOL_A_HEADER_DEV_NAME`
0x73 0x69 0x6d 0x30         # c char: sim0
0x00 0x00 0x00 0x00         # NULL and padding. nested header ends at 24 length

# Netlink Attributes -- ethtool header ETHTOOL_A_PAUSE_AUTONEG
0x05 0x00 0x02 0x00         # 0x05 0x00: lenth 5(without padding)
                            # 0x02 0x00: ETHTOOL_A_PAUSE_AUTONEG

# Netlink Attributes -- ethtool header ETHTOOL_A_PAUSE_RX
0x00 0x00 0x00 0x00         # u8 payload: FALSE and padding
0x05 0x00 0x03 0x00         # 0x05 0x00: lenth 5(without padding)
                            # 0x03 0x00: ETHTOOL_A_PAUSE_RX
0x01 0x00 0x00 0x00         # u8 payload: TRUE and padding

# Netlink Attributes -- ethtool header ETHTOOL_A_PAUSE_TX
0x05 0x00 0x04 0x00         # 0x05 0x00: lenth 5(without padding)
                            # 0x04 0x00: ETHTOOL_A_PAUSE_TX
0x01 0x00 0x00 0x00         # u8 payload: TRUE and padding.
```

[kernel_ethtool_doc]: https://www.kernel.org/doc/html/latest/networking/ethtool-netlink.html
[nispor_pr_ethtool]: https://github.com/nispor/nispor/pull/118
[wifi_netlink]: https://wireless.wiki.kernel.org/en/developers/documentation/nl80211
[libnl]: https://www.infradead.org/~tgr/libnl/
