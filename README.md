# Banshee

OpenPanahon weather station node firmware

## Reading logs

> [!IMPORTANT]
> `defmt`-based logging is still a work in progress

Since this project uses `defmt`, please install `defmt-print`, as
the serial monitor

```sh
cargo install defmt-print
```

## Using `minicom` on Banshee

Ensure that `--flow none` is passed to `minicom` when accessing
the serial monitor:

```sh
picocom --flow none /dev/ttyACM0s
```

## Using PuTTY on Banshee

Serial mode with 115200 baud is sufficient for data. As of the time of writing, the baud rate doesnt really matter, since `picocom` can work

## Note on fetching errors

During a `panic!()`, the firmware dumps the panic message at a certain region of ram, which can be fetched using [`picotool`](https://github.com/raspberrypi/pico-sdk-tools/releases/latest)

```sh
picotool save -r 0x15000000 0x15004000 message.bin
```

or, more succinctly,

```sh
picotool save -r 0x15000000 0x15004000 message.bin && cat message.bin && rm message.bin
```