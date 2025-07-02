# Simple Commander (simcom)

## Goal
Mimic Norton Commander functionality using a  web interface and the websocket.

## Technologies
- Rust
- WebSocket

This application doesn't use HTTP besides of loading the 'index' page. All exchanges with a server
are the websocket based. Every websocket packet is a JSON object.

## Dependencies
Currently the project has 4 direct dependecies:

- [SimJSON](https://github.com/vernisaz/simjson)
- [SimWEb](https://github.com/vernisaz/simweb)
- [SimTime](https://github.com/vernisaz/simtime)
- [SimZip](https://github.com/vernisaz/simple_rust_zip)

## Web server
Since the project uses the websocket, only the [SimHTTP](https://github.com/vernisaz/simhttp)
can be used to run it now,
because the project implementation is WS CGI based.
It isn't a drawback since the server exists on all platforms.
Hopefully more vendors will adopt the WS CGI soon and the list of supporting servers will be extended after.

## File upload
The functionality handled by [upload CGI](https://github.com/vernisaz/simupload) project. Make sure that *upload URL*
configured properly accordingly your web server settings. Default value is **./upload**.

## Platforms
Windows, Mac, Linux and free BSD are supported.

## Installation
Unzip the provided archive accordingly your platform and a processor type. Launch *simcom* script, or directly *./bin/simhttp*
from the directory where the installation archive was opened. The access URL will look like -

> http://localhost:3000/cmd/

You cam edit **env.conf** to change a host name or port.

## Status
There is a beta version you can try.
