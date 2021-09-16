# Overview

This application allows building any RIOT application with riot-rs-core.

Compile & flash like this:

    laze task -b <board> -a riot-app -DRIOT_APP=foo/bar flash

or, if the application is not in the RIOT tree:

    laze task -b <board> -a riot-appdir -DAPP_DIR=path/to/app flash
