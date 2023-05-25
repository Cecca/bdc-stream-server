A simple TCP server that serves random numbers to whoever connects to its port.

The random (integer) numbers come from one of the following two distributions:

- `Zipf`, with a random `alpha` between two user-defined `alpha_min` and `alpha_max`
- Uniform, with values between `0` and `uniform_max`

For each number, the software decides from which distribution to draw numbers based on a coin flip with probability `balance`: if `true` then a uniformly distributed number is generated, otherwise one is sampled from the Zipf distribution.

All the above parameters are set in a `toml` file.

Such file is continuously monitored so that we can reconfigure the server without restarting it.

The default installation has two instances of the server managed by `systemd`, each reading a different configuration file:

- `~/streamer.toml`
- `~/streamer-test.toml`

