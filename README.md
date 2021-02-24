# fanalog

Serial log collector: 
collects serial logging data from one or more USB serial devices/ports,
automatically following ports as devices are connected and disconnected,
and forwards these logs to a remote logging endpoint via a webhook URL.


## Installing and Running

Binaries for multiple platforms are currently provided (on github) in [releases](./releases).
You can simply download the release archive, uncompress it, and place the binary somewhere in your path.

In your shell you will need to export an environment variable 
`COLLECTOR_ENDPOINT_URL`
that is the URL for the logging service.
These webhooks are often provided as URL strings with authentication tokens included:

```
export COLLECTOR_ENDPOINT_URL=https://foo.bar.baz/xxxxx/xxxx
```

Then you should be able to simply run the `fanalog` binary to run it in your current session.  
(Your user will need to have permissions to access USB.)
If you wish to have fanalog continue running when you've logged out, you'll need to use something like `screen`
or eg systemd service wrapper than can be enabled and disabled.



## Building

[Install rust](https://www.rust-lang.org/tools/install) 
on your target platform of choice, clone this repository, and run:

`cargo build` for the debug build or 
`cargo build --release` for the optimized release build. 

You can also buid and run from source with either `cargo run`, `cargo run --release`,
or by moving eg `/target/release/fanalog` to somewhere in your `$PATH` 

### Troubleshooting Build

- On Linux-based systems (eg Raspberry Pi) you may get the following error if SSL is not installed:

```
  Could not find directory of OpenSSL installation, and this `-sys` crate cannot
  proceed without this knowledge. If OpenSSL is installed and this crate had
  trouble finding it,  you can set the `OPENSSL_DIR` environment variable for the
  compilation process.

  Make sure you also have the development packages of openssl installed.
  For example, `libssl-dev` on Ubuntu or `openssl-devel` on Fedora.
```
  
Follow the directions to install required SSL libraries.
For example, on RPi4 running Rasbian, this worked for me: 
`sudo apt-get install libssl-dev`








