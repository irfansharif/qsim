## Running the simulations
```
cargo build
.\target\debug\qsim.exe
```
## Options
```
-h, --help      Show help
--rate NUM      Average number of generated packets/s (default: 10000)
--psize NUM     Packet size; bits (default: 1)
--pspeed NUM    Packet processing speed; bits/s (default: 10000)
--duration NUM  Duration of simulation; seconds (default: 5)
--qlimit NUM    Limit on of the buffer queue length; int (default: None)
```