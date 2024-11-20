# log

## About

This application provides a simple demonstrator for log output of the main
application at different log levels.

This application prints a log statement at every log level. Only the log
statements which are enabled are printed.

The log level can be provided on the command line as argument to laze via the
`-DLOG` argument. The available log levels that can be configured are:

- error
- warn
- info
- debug
- trace

## How to run

In this folder, run

    laze build -b nrf52840dk -DLOG+=info -DLOG+=ariel_os_embassy=warn run

## Expected output

The output of this example depends on the enabled log level. With trace set as
log level, the following output is expected from this example (Ariel OS startup
logging not shown):

```
TRACE -- trace log level enabled
DEBUG -- debug log level enabled
INFO  -- info log level enabled
WARN  -- warn log level enabled
ERROR -- error log level enabled (just testing)
```
