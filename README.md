# fanalog

Serial log collector: 
collects serial logging data from one or more
serial ports whose names match a given pattern,
automatically following ports as devices are
connected and disconnected,
and forwards these logs to a remote syslog server.


## Running

Binaries for multiple platforms are currently provided in (releases)[https://github.com/tstellanova/fanalog/releases]).
You can simply download the release archive, uncompress it, and place the binary somewhere in your path.

In your shell you will need to export an environment variable that is the URL for the logging service.
These webhooks are often provided as URL strings with authentication tokens included:

```
export COLLECTOR_ENDPOINT_URL=https://foo.bar.baz/xxxxx/xxxx
```

Then you should be able to simply run the `fanalog` binary to run it in your current session.  
(Your user will need to have permissions to access USB.)
If you wish to have fanalog continue running when you've logged out, you'll need to use something like `screen` 
or service wrapper than can be enabled and disabled.







