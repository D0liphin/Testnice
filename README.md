# `testnice`

You can use this by building the project (preferably in debug mode) and then 
using the builtin CLI.

For this to have a noticeable effect to allow students to view the scheduler
churning away in real-time, you will need to flood all your cores. You can 
do this with 

```
sudo testnice --flood=<numcores> --nice=-20
```

Doing a number that is too high for `--flood` will just crash your PC tbh.
