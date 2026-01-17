# Simple Commander (simcom)

## Goal
Mimic Norton Commander functionality using a  web interface and the websocket.

## Technologies
- Rust
- WebSocket

This application doesn't use HTTP besides of loading the 'index' page. All exchanges with a server
are the websocket based. Every websocket packet is a JSON object.

## Dependencies
Currently the project has 4 direct dependencies:

- [SimJSON](https://github.com/vernisaz/simjson)
- [SimWEb](https://github.com/vernisaz/simweb)
- [SimTime](https://github.com/vernisaz/simtime)
- [SimZip](https://github.com/vernisaz/simple_rust_zip)
- [SimConfig](https://github.com/vernisaz/simconfig)

And one optional dependency:
- [SimTerminal](https://github.com/vernisaz/simterminal)

You will need also common [scripts](https://github.com/vernisaz/simscript) to build them.

The project uses also 2 3rd party dependencies for viewing an info of image files. How to get and build them is described in
[dep crates/README.md](https://github.com/vernisaz/simcom/blob/master/dep%20crates/README.md). Since the crate
requires the license for a distribution in a source or a binary form, I do not distribute Simple Commander in any form. Keep it in mind
if you plan to distribute the Simple Commander.

## Why the author is so concerened about dependencies
First, every dependency tells that a certain functionality goes from other product. And other product can present it better.
Second, every dependency adds own security risks, so the security of the entire product will depend on the security
of the weakest product in the chain.

## Web server
Since the project uses a websocket endpoint implemented on WS CGI technology,
only the [SimHTTP](https://github.com/vernisaz/simhttp)
can be used to run it properly.
It isn't a drawback since the server exists on all platforms.
Hopefully, more vendors will adopt the WS CGI soon and the list of supporting servers will be extended.

Read a note in the above server description regarding running it in a background mode without setup as a service.

## File upload
The functionality handled by [upload CGI](https://github.com/vernisaz/simupload) project. Make sure that *upload URL*
configured properly accordingly your web server settings. Default value is **./upload**.

Upload functionality is also possible using websocket, however currently SimHTTP doesn't support binary packets.

## Platforms
Windows, Mac, Linux and Free BSD are supported.

## Installation
Unzip the provided archive accordingly your platform and the processor type. Launch *simcom* script, or directly *./bin/simhttp*
from the directory where the installation archive was opened. The access URL will look like -

> http://localhost:3000/cmd/

You can edit **env.conf** to change a host name or port.

## Known issues
If the WebSocket connection was closed for some reason, then the first interaction with it after can be lost, 
and a repeating it can be required.

## No-Cargo build
This product uses an [alternative tool](https://github.com/vernisaz/rust_bee) to the Cargo building tool.
It makes  sense especially for products
having 0 dependencies on the _crates.io_. Some 3rd party dependency exists though, however they can be built
without Cargo as well.

## Status
The product is in the beta state.
