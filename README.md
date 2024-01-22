# `testnice`

This is **currently not usable**. It is under incredible construction and I am
going to the gym rn. I'll fix it all when I get back. 

You will need to `cargo build` this. 

For this to have a noticeable effect to allow students to view the scheduler
churning away in real-time, you will need to flood your CPU with garbage work. 
You can do this with 

```
sudo testnice flood <numthreads> --ni=-20
```

Doing a number that is too high for `--flood` will just freeze your PC. Don't 
do more than the number of cores you have as a start.

## System Requirements

You must be running on a valid **Linux** distribution (this is a demonstration
of linux-specific scheduling after all).

You will also need to install the following libraries which you probably 
already have. 

- [`ncurses`](https://github.com/gyscos/cursive/wiki/Install-ncurses)
- `libc`

## Demonstration

This has been considerably updated and now uses a TUI to compare two processes
with different nice levels. 

By default, all `testnice` instances use the same logfile.

1. In a separate terminal

```
sudo testnice flood --thread-count=<THREAD_COUNT>
```

2. Then in the terminal that you are working from 

```
sudo testnice tui --ni1=-20 --ni2=19
```

