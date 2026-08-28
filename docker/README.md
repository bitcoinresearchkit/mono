# Run Bitview with Docker

The Compose setup builds and runs the official `bitviewd` composition against
an existing Bitcoin Core node.

## Requirements

- Docker Engine with Buildx and Docker Compose v2
- Bitcoin Core with RPC enabled
- A readable Bitcoin Core data directory containing `blocks/`
- About 290 GiB for the current Bitview data, plus Bitcoin Core storage and
  growth headroom
- 16 GB of RAM recommended for a full sync

## Start

```bash
cp docker/.env.example docker/.env
docker compose -f docker/docker-compose.yml up -d
```

Set `BITCOIN_DATA_DIR` and a `BTC_RPC_HOST` reachable from the container in
`docker/.env` before starting. The Compose file maps host port 7070 to
Bitview's port 3110 inside the container:

- Website and interactive API: <http://localhost:7070>
- API root: <http://localhost:7070/api>
- Health: <http://localhost:7070/health>

## Configuration

The Compose file passes settings to `bitviewd` as command-line arguments.
These environment variables are interpolated when Compose starts:

| Variable | Purpose | Default |
|---|---|---|
| `BITCOIN_DATA_DIR` | Host path to the Bitcoin Core data directory | `/path/to/bitcoin` |
| `BTC_RPC_HOST` | Bitcoin Core RPC host as reached from the container | `localhost` |
| `BTC_RPC_USER` | RPC username | `bitcoin` |
| `BTC_RPC_PASSWORD` | RPC password | `bitcoin` |
| `BITVIEW_DATA_VOLUME` | Docker volume used for Bitview data | `bitview-data` |

Edit the `command:` section in [`docker-compose.yml`](./docker-compose.yml) for
other `bitviewd` options.

### Reach Bitcoin Core

- Bitcoin Core on the Docker host: use `host.docker.internal` on macOS and
  Windows. On Linux, use an address reachable from the container.
- Bitcoin Core in another container: use its service name on a shared Docker
  network.
- Remote Bitcoin Core: use its reachable hostname or IP address.

For username/password authentication, set `BTC_RPC_USER` and
`BTC_RPC_PASSWORD` in `docker/.env`. For cookie authentication, follow the
commented alternative in [`docker-compose.yml`](./docker-compose.yml) and
remove the username/password arguments.

## Data storage

The default named volume is `bitview-data`. To select another named volume, set
`BITVIEW_DATA_VOLUME`.

For a bind mount:

1. Set `BITVIEW_DATA_DIR` in `docker/.env`.
2. Uncomment the bind-mount line and comment the named-volume line in
   [`docker-compose.yml`](./docker-compose.yml).
3. Remove the top-level `volumes:` declaration if it is no longer used.

Bitview uses sparse files. Docker volume inspection may report logical sizes
above 1 TiB; use `du -sh` on the volume mount point to see allocated space.

## Operate

```bash
docker compose -f docker/docker-compose.yml build
docker compose -f docker/docker-compose.yml ps
docker compose -f docker/docker-compose.yml logs -f
```

The Bitcoin directory is mounted read-only and `bitviewd` runs as the non-root
`bitview` user (UID 1000). If startup reports a permission error, make the
Bitcoin data directory readable by that user.
