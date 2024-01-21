# `testnice`

You can use this by building the project (preferably in debug mode) and then 
using the builtin CLI.

For this to have a noticeable effect to allow students to view the scheduler
churning away in real-time, you will need to flood all your cores. You can 
do this with 

```
sudo testnice --flood=<numthreads> --nice=-20
```

Doing a number that is too high for `--flood` will just crash your PC tbh.

## Demonstration

After flooding your CPU with high-priority processes you can point out that

- operations are slow *and then fast* as opposed to just consistently slower.
  This is because they are running full-throttle and then have to wait a
  bit.

Then open another terminal and run

```
testnice --nice=19 & sudo testnice --nice=-20 
```

This should show the process with nice level `19` gets scheduled less 
frequently, but while they are both running, they exeute at the same 'speed'.

## Displaying `sched_entity`

We cannot show the exact `sched_entity` but we are able to acquire lots of 
interesting information about the schedule. This demonstration comes with a 
builtin way of displaying the nice level as well as

- `vrt` (`se.vruntime`)
- `nsw` (`nr_switches`)
- `w` (`se.load.weight`)
- `pol` (`policy`)
    - `oth` (`SCHED_OTHER`)
    - `ffo` (`SCHED_FIFO`)
    - etc.

Just pass `--display-sched`.