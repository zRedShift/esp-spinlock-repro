# esp-spinlock-repro
Running `cargo espflash --release --monitor` with fat LTO enabled seems to cause the timer driver to misbehave and 
lock the thread up after working as intended beforehand, exactly 255 times.

The serial output should be the following and no more, when compiling with LTO="fat". Other settings result in the 
timer continuing to work:
```
I (314) esp_spinlock_repro: counter at 0
I (324) esp_spinlock_repro: counter at 1
I (334) esp_spinlock_repro: counter at 2
I (344) esp_spinlock_repro: counter at 3
...

I (2914) esp_spinlock_repro: counter at 252
I (2924) esp_spinlock_repro: counter at 253
I (2934) esp_spinlock_repro: counter at 254
I (2944) esp_spinlock_repro: counter at 255
```
This was reproduced on several ESP32S3 chips, using the `esp` rust toolchain, versions 1.64.0.0 and 1.65.0.0, using 
`esp-idf` branches `release/v4.4` and `master` (with tweaks to the rust's `libstd` to use a newer version of `libc` due
to the `espidf_time64` config).
