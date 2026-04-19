use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ResolvedStorageObjectLocation {
    pub object_uri: String,
    pub bucket_name: Option<String>,
    pub object_key: Option<String>,
}

pub fn resolve_storage_object_location(
    object_uri: &str,
    fallback_bucket: Option<&str>,
) -> ResolvedStorageObjectLocation {
    let trimmed = object_uri.trim();
    if trimmed.is_empty() {
        return ResolvedStorageObjectLocation::default();
    }

    if let Some(rest) = trimmed.strip_prefix("s3://") {
        let mut parts = rest.splitn(2, '/');
        let bucket_name = parts
            .next()
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        let object_key = parts
            .next()
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        return ResolvedStorageObjectLocation {
            object_uri: trimmed.to_string(),
            bucket_name,
            object_key,
        };
    }

    if let Ok(parsed) = reqwest::Url::parse(trimmed) {
        let mut segments = parsed
            .path_segments()
            .into_iter()
            .flat_map(|items| items.filter(|value| !value.is_empty()));
        let bucket_name = segments.next().map(str::to_string).or_else(|| {
            fallback_bucket
                .filter(|value| !value.trim().is_empty())
                .map(|value| value.to_string())
        });
        let object_key = {
            let remaining = segments.collect::<Vec<_>>().join("/");
            if remaining.is_empty() {
                None
            } else {
                Some(remaining)
            }
        };
        return ResolvedStorageObjectLocation {
            object_uri: trimmed.to_string(),
            bucket_name,
            object_key,
        };
    }

    let normalized = trimmed.trim_start_matches('/');
    let bucket_name = fallback_bucket
        .filter(|value| !value.trim().is_empty())
        .map(|value| value.to_string());
    let object_key = if normalized.is_empty() {
        None
    } else if bucket_name.is_some() {
        Some(normalized.to_string())
    } else {
        let mut parts = normalized.splitn(2, '/');
        let inferred_bucket = parts
            .next()
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        let inferred_key = parts
            .next()
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        return ResolvedStorageObjectLocation {
            object_uri: trimmed.to_string(),
            bucket_name: inferred_bucket,
            object_key: inferred_key,
        };
    };

    ResolvedStorageObjectLocation {
        object_uri: trimmed.to_string(),
        bucket_name,
        object_key,
    }
}

#[cfg(test)]
mod tests {
    use super::resolve_storage_object_location;

    #[test]
    fn resolves_s3_uri() {
        let resolved = resolve_storage_object_location("s3://delivery-objects/orders/a.bin", None);
        assert_eq!(resolved.bucket_name.as_deref(), Some("delivery-objects"));
        assert_eq!(resolved.object_key.as_deref(), Some("orders/a.bin"));
    }

    #[test]
    fn resolves_http_uri_with_bucket_prefix() {
        let resolved = resolve_storage_object_location(
            "http://127.0.0.1:9000/delivery-objects/orders/a.bin",
            None,
        );
        assert_eq!(resolved.bucket_name.as_deref(), Some("delivery-objects"));
        assert_eq!(resolved.object_key.as_deref(), Some("orders/a.bin"));
    }

    #[test]
    fn falls_back_to_bucket_when_uri_is_relative() {
        let resolved = resolve_storage_object_location("orders/a.bin", Some("delivery-objects"));
        assert_eq!(resolved.bucket_name.as_deref(), Some("delivery-objects"));
        assert_eq!(resolved.object_key.as_deref(), Some("orders/a.bin"));
    }
}
