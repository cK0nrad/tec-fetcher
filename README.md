# TEC Fetcher

This is a GTFS-RT fetcher for the TEC (OTW).

## Usage

```bash
$ cargo run
```
By default, it'll run on port 3000.

There's some environment variables you can set:

- `API_KEY`: the API key to use
- `API_URL`: the API URL to use
- `IP`: the IP to listen on
- `PORT`: the port to listen on (default: `3000`)
- `SECRET`: the secret to use for refresh

Those are mendatory to run the server. There's no default values.