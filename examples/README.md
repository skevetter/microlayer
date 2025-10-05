# Examples

This directory contains example Dockerfiles demonstrating microlayer usage.

## Ubuntu/Debian Example

```bash
docker build -f examples/Dockerfile.ubuntu -t microlayer-ubuntu-demo .
```

This creates a container with curl, git, and htop installed with minimal layer size.

## Alpine Example

```bash
docker build -f examples/Dockerfile.alpine -t microlayer-alpine-demo .
```

This creates an Alpine-based container with the same packages.

## Comparison

To see the difference in layer size, build the traditional example:

```bash
docker build -f examples/Dockerfile.traditional -t microlayer-traditional-demo .
```

Then compare the image sizes:

```bash
docker images | grep microlayer
```

You should see that the microlayer-based images have significantly smaller layers.

## Testing Locally

To test the commands without Docker:

```bash
# Build the project
cargo build --release

# Test on Ubuntu/Debian (requires root/sudo)
sudo ./target/release/microlayer apt-get curl,git

# Test on Alpine (requires root/sudo)
sudo ./target/release/microlayer apk curl,git

# Test GitHub release installer
sudo ./target/release/microlayer gh-release cli/cli gh --version latest
```

## Automated Tests

Run the test script to verify the CLI interface:

```bash
./test.sh
```
