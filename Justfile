# Install the software as a systemd service and starts it
install:
    cargo install --path .
    test -d ~/.config/systemd/user || mkdir -p ~/.config/systemd/user
    cp *.service ~/.config/systemd/user
    test -f ~/streamer.toml || cp config.toml ~/streamer.toml
    test -f ~/streamer-test.toml || cp config-test.toml ~/streamer-test.toml
    systemctl enable --user bdc-stream-server
    systemctl enable --user bdc-stream-server-test
    systemctl restart --user bdc-stream-server
    systemctl restart --user bdc-stream-server-test

# Run a stress test by reading from `nclients` simultaneous connections
# 10 million bytes from a server at the given host:port pair
stress host port nclients:
    #!/bin/bash
    for CLIENT in $(seq {{nclients}})
    do
        nc {{host}} {{port}} | head -c 10000000 > /dev/null &
    done

