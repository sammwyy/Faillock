# Faillock

## Description

Block failed login attempts on popular services (SSH, MySQL, PostgreSQL). It serves as an alternative to `Fail2ban`, implemented in Rust.

## Features

- Automatic blocking of IP addresses with failed login attempts.
- Supports services such as SSH, MySQL, PostgreSQL, and Postfix.
- Built with Rust for fast and secure performance.
- Real-time monitoring through log file analysis.

## Configuration

### Configuration File (`config.toml`)

The main configuration is defined in the `config.toml` file:

```toml
ban_command = "iptables -A INPUT -s {ip} -j DROP"

[[monitors]]
type = "ssh"
cleanup_interval = 60
max_attempts = 3
```

- `ban_command` defines the command used to block IP addresses (e.g., `iptables` command).
- `cleanup_interval` specifies the time interval in seconds for cleaning up outdated entries.
- `max_attempts` sets the threshold for the maximum number of failed attempts allowed before banning an IP.
- `name` (optional) provides a custom name for the monitor.

### Custom Services

You can add custom services by creating a `.toml` file for each service under the `services` directory. For example:

**services/ssh.toml**:

```toml
log_file = "/var/log/auth.log"
log_trigger = "Failed password for"
log_ip_regex = "(\\d+\\.\\d+\\.\\d+\\.\\d+)"
```

#### Default Services

Faillock comes with default services:

- `mysql`
- `postfix`
- `postgresql`
- `ssh`

The `type` of the monitor corresponds to the filename `<type>.toml` inside the `services` directory.

### Adding Custom Monitors

1. Create a new `.toml` file in the `services` directory for your custom service.
2. Define `log_file`, `log_trigger`, and `log_ip_regex` as required.
3. Update the configuration to include your custom service under `[monitors]`.

## Usage

1. Configure your services using the provided configuration schema (via `config.toml` and individual service `.toml` files).
2. Add monitors for the necessary services (e.g., SSH, MySQL, PostgreSQL).
3. Run the monitoring process and let `Faillock` handle failed login attempts.

> It's recommended to run `Faillock` as a daemon to prevent it from exiting unexpectedly.

## Contributing

Your contributions are welcome! If you’d like to improve or add new features, feel free to fork the project and submit pull requests. Ensure you follow Rust’s style and best practices.

## License

This project is licensed under the MIT License. See `LICENSE` for more details.
