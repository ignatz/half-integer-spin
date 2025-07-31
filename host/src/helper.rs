use spin_factor_outbound_networking::OutboundNetworkingFactor;

pub fn outbound_networking_factor() -> OutboundNetworkingFactor {
  fn disallowed_host_handler(scheme: &str, authority: &str) {
    let host_pattern = format!("{scheme}://{authority}");
    tracing::error!("Outbound network destination not allowed: {host_pattern}");
    if scheme.starts_with("http") && authority == "self" {
      terminal::warn!(
        "A component tried to make an HTTP request to its own app but it does not have permission."
      );
    } else {
      terminal::warn!(
        "A component tried to make an outbound network connection to disallowed destination '{host_pattern}'."
      );
    };
    eprintln!(
      "To allow this request, add 'allowed_outbound_hosts = [\"{host_pattern}\"]' to the manifest component section."
    );
  }

  let mut factor = OutboundNetworkingFactor::new();
  factor.set_disallowed_host_handler(disallowed_host_handler);
  factor
}
