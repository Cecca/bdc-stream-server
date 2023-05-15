install:
    cargo install --path .
    test -d ~/.config/systemd/user || mkdir ~/.config/systemd/user
    cp *.service ~/.config/systemd/user
    systemctl enable --user bdc-stream-server
    systemctl start --user bdc-stream-server
