# Setup with Caprover

Initial installs work well, but as soon as a port-mapping is added, the container receives a `SIGTERM` and shuts down after every.
[Setting up the app before building could work](https://github.com/caprover/caprover/issues/720), but we luckily don't need any port-mappings for this container.
Otherwise, using [actix's `Signal`](https://docs.rs/actix-web/4.0.1/actix_web/rt/signal/unix/struct.Signal.html) could also help to ignore `SIGTERM`.
