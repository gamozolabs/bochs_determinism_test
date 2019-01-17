# Summary

This is a tool designed to test if Bochs is deterministic. It's a mainly stock version of Bochs with a named pipe built in allowing for instances to report all of the operations they're performing.

Inside you'll find a nearly-stock Bochs source tree in `bochs` and a `hammer` Rust project which runs multiple instances of Bochs comparing their outputs.

# What it does

This tool uses a named pipe (named "\\\\.\\pipe\\mynamedpipe%d" where %d is the PID). The `hammer` process then connects to this named pipe in each Bochs instance, comparing their streams to verify they're executing the same things in the same ways.

What this compares varies, but currently it's things like the event type (interrupt, instruction execution, etc), RIP, and various internal Bochs timers which may influence execution.

This tool actually launches 2 instances of Bochs from boot based on `bochsrc.bxrc` and compares them. At some point (see `perform_logging()` in `exception.cc`) one of the instances will take a snapshot, will exit, and will start a new instance of Bochs from this snapshot. This allows us to compare a Bochs instance that booted and continued, versus one that was snapshotted and resumed. In theory these should always have the same result. In practice there are some things in Bochs that are not deterministic. This project is to find those things and fix them :D

# Performance

On my machine (Intel i7-8700) I'm able to diff Bochs at about 25M instructions per second in realtime. This means there's almost 3 GiB/second of traffic going over the pipes. I'm pretty happy with this performance and don't really plan to improve it. It's about a ~2-3x slowdown from stock Bochs, which is a pretty good cost for diffing them at the instruction level.

# Status

Currently Bochs is not deterministic, but it's close. I expect we'll be fully deterministic in a short while :)

# Building

Run `python build.py` in a MSVC x64 prompt. You'll need `make` and `autoconf` installed in Cygwin 64 as well.

# Support

This is just meant to be a testing repo. There's not really gonna be any release cycles or support here. Once we get things deterministic we'll document the changes and this repo will then cease to be updated.

I still plan to maintain this a little bit as it's a great tool to test that modifications to Bochs haven't influenced the guest behaviour.
