# 1Password-Fuse

A simple FUSE filesystem for 1Password vaults. It wraps the 1Password CLI to
provide a (currently) read-only view of your vaults and secrets.

It assumes that you have the 1Password CLI installed and configured so that
any `op` command will trigger the authentication flow on the 1Password desktop
app.

## Usage

```sh
op-fuse /path/to/config.toml
```

## Security

Security is (for the time being) entirely dependent on the filesystem permissions.

Files and directories are created with ownership matching the configured `uid`
and `gid`. The mode is set to `0400` for files and `0500` for directories.

A short lived cache will make op-fuse frequently re-fetch the data from the
1Password CLI and require re-authentication.

`op-fuse` makes no attempt to protect the data in memory.

**Be very careful when considering the use of this tool. Take the time to
look for a feature in the 1Password CLI that would match you use case
before considering op-fuse !**

## Example configuration

```toml
mountpoint = "/mnt/op"
uid = 1000
gid = 1000

cache_duration = "60s"

[onepassword]
cmd = "op"

[accounts.personal]
id = "ABCDEFGHIJKLMNOPQRSTUVWXYZ"

[accounts.personal.vaults.private]
id = "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
```

## Example Systemd service

```ini
[Unit]
Description=1Password FUSE
After=network.target
StartLimitIntervalSec=0

[Service]
Type=simple
Restart=always
RestartSec=1
User=<user>
ExecStart=/path/to/op-fuse /path/to/op-fuse.toml

[Install]
WantedBy=multi-user.target
```
