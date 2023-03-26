output "index_url" {
  description = "PyPI index url. Set it as the -i parameter with pip."
  value       = "https://${scaleway_container.index.domain_name}"
}
