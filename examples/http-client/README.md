# http-client

## About

This application is demonstrating making basic HTTP GET requests.

## How to run

In this folder, run

    laze build -b nrf52840dk run

This example needs to be provided with an endpoint URL to send the HTTP GET
request to, through the `ENDPOINT_URL` environment variable.
TLS 1.3 and mDNS are supported; however the server is not authenticated.
A GET request will be made every 3 seconds, even in case of failure and the
response body will be printed as a string, along with some relevant HTTP
response headers.

Look [here](../README.md#networking) or more information about network configuration.
