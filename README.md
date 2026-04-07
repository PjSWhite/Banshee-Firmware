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
