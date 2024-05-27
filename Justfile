# Install the software as a systemd service and starts it
install:
    cargo install --path .
    test -d ~/.config/systemd/user || mkdir -p ~/.config/systemd/user
    cp *.service ~/.config/systemd/user
    test -f ~/streamer-two.toml || cp config-two.toml ~/streamer-two.toml
    test -f ~/streamer-two-test.toml || cp config-two-test.toml ~/streamer-two-test.toml
    test -f ~/streamer-three.toml || cp config-three.toml ~/streamer-three.toml
    test -f ~/streamer-three-test.toml || cp config-three-test.toml ~/streamer-three-test.toml
    systemctl enable --user bdc-stream-server-two
    systemctl enable --user bdc-stream-server-two-test
    systemctl restart --user bdc-stream-server-two
    systemctl restart --user bdc-stream-server-two-test
    systemctl enable --user bdc-stream-server-three
    systemctl enable --user bdc-stream-server-three-test
    systemctl restart --user bdc-stream-server-three
    systemctl restart --user bdc-stream-server-three-test

# Run a stress test by reading from `nclients` simultaneous connections
# 10 million bytes from a server at the given host:port pair
stress host port nclients:
    #!/bin/bash
    for CLIENT in $(seq {{nclients}})
    do
        nc {{host}} {{port}} | head -c 10000000 > /dev/null &
    done

