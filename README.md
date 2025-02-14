# rcon-echo

Small RCON server that prints any received command into STDOUT.

## Why?

Just a very small and quick project to get more experience on RCON spec & rust.

RCON spec: https://developer.valvesoftware.com/wiki/Source_RCON_Protocol


## Debugging

You can use [rcon-cli](https://github.com/gorcon/rcon-cli/) to send commands for testing

```bash
./rcon -a 127.0.0.1:27015 -p password
```