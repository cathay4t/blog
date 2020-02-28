---
title: "Thinkpad: Turnoff the mic mute, sound mute and power button LEDs"
date: 2020-02-28T08:09:14+08:00
---
## Thinkpad: Turnoff the mic mute, sound mute and power button LEDs

```bash
#!/bin/bash
SYSFS_THINKPAD_LED="/sys/devices/platform/thinkpad_acpi/leds"
for LED in platform::mute platform::micmute tpacpi::power; do
    echo 0 | sudo tee "$SYSFS_THINKPAD_LED/$LED/brightness"
done
```
