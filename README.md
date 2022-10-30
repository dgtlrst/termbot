# termbot

An implementation of USB communication with RP2040 written in Rust.

## Protocol Specification

There are <u>READ</u> and <u>WRITE</u> commands. <br/>
All READ commands start with 'TBR' from 'TERMBOT READ'
All WRITE commands start with 'TBW' from ..aah you get it <br/>

Currently implemented commands:

### Read uptime
Reads the uptime of the termbot.
