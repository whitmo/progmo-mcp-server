# Running p-mo as a Daemon

This guide explains how to run the p-mo server as a background daemon process.

## Command Line Usage

To start p-mo as a daemon:

```bash
p-mo start --daemon
```

You can also specify host and port:

```bash
p-mo start --daemon --host 0.0.0.0 --port 3000
```

Short options are also available:

```bash
p-mo start -d -h 0.0.0.0 -p 3000
```

## Configuration Files

By default, p-mo uses the following files when running as a daemon:

- PID file: `/tmp/p-mo.pid`
- Log file: `/tmp/p-mo.log`

## Checking Status

To check if the daemon is running:

```bash
p-mo status
```

## Stopping the Daemon

To stop the daemon:

```bash
p-mo stop
```

## System Service Integration

### systemd (Linux)

Create a systemd service file at `/etc/systemd/system/p-mo.service`:

```
[Unit]
Description=p-mo Server
After=network.target

[Service]
Type=simple
User=<your-username>
ExecStart=/usr/local/bin/p-mo start
ExecStop=/usr/local/bin/p-mo stop
Restart=on-failure

[Install]
WantedBy=multi-user.target
```

Enable and start the service:

```bash
sudo systemctl enable p-mo
sudo systemctl start p-mo
```

### launchd (macOS)

Create a plist file at `~/Library/LaunchAgents/com.user.p-mo.plist`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.user.p-mo</string>
    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/bin/p-mo</string>
        <string>start</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>/tmp/p-mo.log</string>
    <key>StandardErrorPath</key>
    <string>/tmp/p-mo.log</string>
</dict>
</plist>
```

Load the service:

```bash
launchctl load ~/Library/LaunchAgents/com.user.p-mo.plist
```

## Troubleshooting

If the daemon fails to start:

1. Check the log file at `/tmp/p-mo.log`
2. Ensure the port is not already in use
3. Verify you have permission to write to the PID and log files

If you need to force-kill the process:

```bash
# Find the PID
cat /tmp/p-mo.pid

# Kill the process
kill -9 <PID>
```
