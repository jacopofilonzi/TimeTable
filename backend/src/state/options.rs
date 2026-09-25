use std::{collections::HashMap, sync::Arc};

use super::{
    AppState,
    validation::{canonical, find_field, required, validate_shallow},
};
use crate::{
    errors::AppError,
    models::{Field, FieldKind, SelectOption},
    universities::{Params, University},
};

impl AppState {
    /// Options for any select field; `query` must contain the field's dependencies.
    pub async fn options(
        &self,
        uni: &Arc<dyn University>,
        key: &str,
        query: &HashMap<String, String>,
    ) -> Result<Arc<Vec<SelectOption>>, AppError> {
        let info = uni.info();
        let field = find_field(info, key)?;
        match &field.kind {
            FieldKind::Select { options, .. } => Ok(Arc::new(options.clone())),
            FieldKind::RemoteSelect { .. } => {
                let mut deps = Params::new();
                for dep in field.depends_on {
                    let value = required(query, dep)?;
                    validate_shallow(find_field(info, dep)?, value)?;
                    deps.insert(dep.to_string(), value.clone());
                }
                self.remote_options(uni, field, &deps).await
            }
        }
    }

    /// Cached options of a remote field, for already-validated dependencies.
    pub(super) async fn remote_options(
        &self,
        uni: &Arc<dyn University>,
        field: &Field,
        deps: &Params,
    ) -> Result<Arc<Vec<SelectOption>>, AppError> {
        let key = format!(
            "options:{}:{}:{}",
            uni.info().id,
            field.key,
            canonical(deps)
        );
        self.cache
            .get_or_fetch(&key, self.config.options_ttl, || {
                uni.options(&self.http, field.key, deps)
            })
            .await
    }
}
