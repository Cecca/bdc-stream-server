# Install the software as a systemd service and starts it
install:
    cargo install --path .
    test -d ~/.config/systemd/user || mkdir -p ~/.config/systemd/user
    cp *.service ~/.config/systemd/user
    systemctl enable --user bdc-stream-server
    systemctl start --user bdc-stream-server

# Run a stress test by reading from `nclients` simultaneous connections
# 10 million bytes from a server at the given host:port pair
stress host port nclients:
    #!/bin/bash
    for CLIENT in $(seq {{nclients}})
    do
        nc {{host}} {{port}} | head -c 10000000 > /dev/null &
    done

