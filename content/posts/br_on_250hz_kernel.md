---
title: "250HZ kernel Will Act Differently for Linux Bridge"
date: 2020-11-24T19:44:29+08:00
---

When playing with Linux Bridge in Ubuntu 18.04(Travis CI), I noticed
the default value of `multicast_startup_query_interval` option is 3124 while
RHEL/Fedora is 3125:

 * Ubuntu 18.04
```bash
fge@ubt1804:~$ sudo ip link add br0 type bridge
fge@ubt1804:~$ cat /sys/class/net/br0/bridge/multicast_startup_query_interval
3124
```
 * RHEL 8.3
```bash
[fge@el8 ~]$ sudo ip link add br0 type bridge
[fge@el8 ~]$ cat /sys/class/net/br0/bridge/multicast_startup_query_interval
3125
```

After debug, I confirmed this is caused by integer round up at two places:

 * Default value setter: `125 * HZ / 4`
 * `jiffies_to_clock_t()`

The default value of `multicast_startup_query_interval` in kernel jiffies:

 * Ubuntu 18.04, the kernel is holding `7812` rounded from `7812.5`.
 * RHEL 8.3, the kernel is holding `31250`, no round up.


When showing via sysfs, kernel use `jiffies_to_clock_t()`, both 250HZ and
1000HZ kernel uses:
```c
NSEC_PER_SEC = 1000000000L
clock_t = jiffies * ((NSEC_PER_SEC + hz / 2) / HZ ) / (NSEC_PER_SEC / USER_HZ))
```

Above formation will generate 3124 with 7812 jiffies on `250HZ` kernel and
100HZ(hard coded) `USER_HZ`.

Below python script could check the maximum deviation caused by round up
of `jiffies_to_clock_t()` in 250HZ kernel.


```python
#!/usr/bin/python3

NSEC_PER_SEC = 10 ** 9
MAX_U64 = 2 ** 64


def jiffies_to_clock_t(user_hz, hz, jiffies):
    tick_nsec = int((NSEC_PER_SEC + int(hz / 2)) / hz)
    if (tick_nsec % (NSEC_PER_SEC / user_hz)) == 0:
        if hz < user_hz:
            return int(jiffies * (user_hz / hz))
        else:
            return int(jiffies / (hz / user_hz))
    else:
        return int(jiffies * tick_nsec / (NSEC_PER_SEC / user_hz))


def clock_t_to_jiffies(user_hz, hz, clock):
    if hz % user_hz == 0:
        if clock >= MAX_U64 / (hz / user_hz):
            return MAX_U64
        return int(clock * (hz / user_hz))
    if clock >= MAX_U64 / hz * user_hz:
        return MAX_U64
    return int(clock * hz / user_hz)


max_deviation = 0
for original_num in range(0, 2 ** 32):
    if original_num % 100000 == 0:
        print(
            f"Tested {original_num} and found max_deviation: {max_deviation}"
        )
    user_hz = 100
    hz = 250
    new_num = jiffies_to_clock_t(
        user_hz, hz, clock_t_to_jiffies(user_hz, hz, original_num)
    )
    a = abs(original_num - new_num)
    if a > max_deviation:
        max_deviation = a
        print(f"{original_num}: {max_deviation}")

print(max_deviation)
```
