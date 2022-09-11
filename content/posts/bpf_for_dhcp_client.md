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
    (0x28, 0, 0, 0x0000000c),
    (0x15, 0, 8, 0x00000800),
    (0x30, 0, 0, 0x00000017),
    (0x15, 0, 6, 0x00000011),
    (0x28, 0, 0, 0x00000014),
    (0x45, 4, 0, 0x00001fff),
    (0xb1, 0, 0, 0x0000000e),
    (0x48, 0, 0, 0x00000010),
    (0x15, 0, 1, 0x00000044),
    (0x6, 0, 0, 0x00040000),
    (0x6, 0, 0, 0x00000000),
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
