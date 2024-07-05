---
title: "Using Linux BPF Filter in DHCP Client"
date: 2022-03-04T10:34:16+08:00
---

During DHCP discovery process, the DHCP network flow is based on UDP when lower
IP stack is not ready yet. The user space DHCP client program will need
ethernet RAW socket(AF_PACKET, SOCK_RAW) to receive UDP package otherwise
kernel will drop the UDP package as IP mismatch.

Hooking on Ethernet RAW socket requires DHCP client filter out non-DHCP network
package in user space which could result a large CPU overhead. The Linux kernel
is providing [BPF (Berkeley Packet Filter) facility][1] for filtering packages
in kernel space before sending to user space.

Let's go through the workflow via an example:

```rust
// Using the output of `tcpdump -dd 'ip and udp dst port 68'`
const BPF_FILTER_RAW: [(u16, u8, u8, u32); DHCP_BPF_LEN as usize] = [
    // Load protocol type to A
    (BPF_LD | BPF_H | BPF_ABS, 0, 0, ETHER_TYPE_POS),
    // Move on if ETHERTYPE_IP, otherwise drop package
    (BPF_JMP | BPF_JEQ | BPF_K, 0, 8, ETHERTYPE_IP),
    // Load IPv4 protocol type to A
    (BPF_LD | BPF_B | BPF_ABS, 0, 0, IP_PROTO_POS),
    // Move on if UDP, otherwise drop package
    (BPF_JMP | BPF_JEQ | BPF_K, 0, 6, IPPROTO_UDP),
    // Load IPv4 flag and fragment offset
    (BPF_LD | BPF_H | BPF_ABS, 0, 0, IP_FRAGMENT_POS),
    // Drop package which has MF(more fragment) set is 1 or is fragment
    (BPF_JMP | BPF_JSET | BPF_K, 4, 0, 0x1fff),
    // Store IP header length to X
    (BPF_LDX | BPF_B | BPF_MSH, 0, 0, IP_HEADER_LEN_POS),
    // Load UDP destination port number to A
    (
        BPF_LD | BPF_H | BPF_IND,
        0,
        0,
        ETHER_HEADER_LEN + DST_PORT_IN_IP_POS,
    ),
    // Check whether destination port is DHCPV4_DST_PORT
    (BPF_JMP | BPF_JEQ | BPF_K, 0, 1, DHCPV4_DST_PORT),
    // Accept this package
    (BPF_RET, 0, 0, u32::MAX),
    // Drop this package
    (BPF_RET, 0, 0, 0x00000000),
];

pub(crate) fn apply_dhcp_bpf(fd: libc::c_int) -> Result<(), DhcpError> {
    let mut raw_filters = [libc::sock_filter {
        code: 0,
        jt: 0,
        jf: 0,
        k: 0,
    }; DHCP_BPF_LEN as usize];
    for (i, (code, jt, jf, k)) in BPF_FILTER_RAW.iter().enumerate() {
        raw_filters[i].code = *code;
        raw_filters[i].jt = *jt;
        raw_filters[i].jf = *jf;
        raw_filters[i].k = *k;
    }
    let bpf_filter = libc::sock_fprog {
        len: DHCP_BPF_LEN,
        filter: (&raw_filters).as_ptr() as *mut _,
    };

    let rc = unsafe {
        libc::setsockopt(
            fd,
            libc::SOL_SOCKET,
            libc::SO_ATTACH_FILTER,
            (&bpf_filter as *const _) as *const libc::c_void,
            std::mem::size_of::<libc::sock_fprog>() as libc::socklen_t,
        )
    };
    if rc != 0 {
        let e = DhcpError::new(
            ErrorKind::Bug,
            format!(
                "Failed to apply socket BPF filter, error: {:?}",
                nix::errno::Errno::last()
            ),
        );
        log::error!("{}", e);
        Err(e)
    } else {
        Ok(())
    }
}
```

The kernel allows us to use `setsockopt()` via `SO_ATTACH_FILTER` to apply
an BPF filter on a socket. The filter is in special format allowing kernel
to do sanity check.

As user space developer, you don't need to understand every bites of it, using
the output of `tcpdump -dd <filter>` is sufficient.
In this case, we are using output of `tcpdump -dd 'ip and udp dst port 68'`,
it means only UDP package with 68 as destination port will be sent to
this socket for userspace processing. You may refer to man page
`pcap-filter(7)` for filter syntax.

[1]: https://www.kernel.org/doc/html/latest/networking/filter.html
