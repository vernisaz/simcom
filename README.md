# Simple Commander (simcom)

## Goal
Mimic Norton Commander functionality using a  web interface and the websocket.

## Technologies
- Rust
- WebSocket

This application doesn't use HTTP besides of loading 'index' page. All exchanges with a server
websocket based. Every packet is a JSON object.

## Dependencies
Currently the project has 3 direct dependecies:

- [SimJSON](https://github.com/vernisaz/simjson)
- [SimWEb](https://github.com/vernisaz/simweb)
- [SimTime](https://github.com/vernisaz/simtime)
-

## Web server
Since the project uses the websocket, only the [SimHTTP](https://github.com/vernisaz/simhttp) can be used to run it.
Hopefully more vendors will adopt the functionality soon and a list of supporting servers will be extended after.

## File upload
The functionality handled by [upload CGI](https://github.com/vernisaz/simupload) project. Make sure that *upload URL*
configured properly accordingly your web server settings. Default value is **/upload**.

## Platforms
Windows, Mac, Linux and free BSD are supported.

## Installation
Unzip provided archive accordingly your platform and a processor type. Launch *simcom* script, or directly *bin/simhttp*. An access URL will look like -

http://localhost:3000/cmd/

You cam edit **env.conf** to change a host name or port.

## Status
There is a beta version you can try.
