use async_trait::async_trait;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use tokio::sync::RwLock;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("configuration error: {0}")]
    Config(String),
    #[error("startup error: {0}")]
    Startup(String),
    #[error("shutdown error: {0}")]
    Shutdown(String),
}

#[derive(Clone, Default)]
pub struct ServiceContainer {
    services: Arc<RwLock<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>>,
}

impl ServiceContainer {
    pub async fn insert<T>(&self, value: T)
    where
        T: Send + Sync + 'static,
    {
        self.services
            .write()
            .await
            .insert(TypeId::of::<T>(), Arc::new(value));
    }

    pub async fn get<T>(&self) -> Option<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        self.services
            .read()
            .await
            .get(&TypeId::of::<T>())
            .and_then(|item| item.clone().downcast::<T>().ok())
    }
}

#[derive(Clone)]
pub struct ModuleContext {
    pub app_name: &'static str,
    pub container: ServiceContainer,
}

#[async_trait]
pub trait Module: Send + Sync {
    fn name(&self) -> &'static str;
    async fn start(&self, _ctx: &ModuleContext) -> AppResult<()> {
        Ok(())
    }
    async fn shutdown(&self, _ctx: &ModuleContext) -> AppResult<()> {
        Ok(())
    }
}

#[derive(Default)]
pub struct ModuleRegistry {
    modules: Vec<Arc<dyn Module>>,
}

impl ModuleRegistry {
    pub fn register<M>(&mut self, module: M)
    where
        M: Module + 'static,
    {
        self.modules.push(Arc::new(module));
    }

    pub fn modules(&self) -> &[Arc<dyn Module>] {
        &self.modules
    }
}

type Hook = Arc<dyn Fn() + Send + Sync>;

#[derive(Default, Clone)]
pub struct LifecycleHooks {
    pub before_start: Vec<Hook>,
    pub before_shutdown: Vec<Hook>,
}

impl LifecycleHooks {
    pub fn on_before_start<F>(&mut self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.before_start.push(Arc::new(callback));
    }

    pub fn on_before_shutdown<F>(&mut self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.before_shutdown.push(Arc::new(callback));
    }
}

pub struct AppLauncher {
    pub app_name: &'static str,
    pub registry: ModuleRegistry,
    pub container: ServiceContainer,
    pub hooks: LifecycleHooks,
}

impl AppLauncher {
    pub fn new(app_name: &'static str) -> Self {
        Self {
            app_name,
            registry: ModuleRegistry::default(),
            container: ServiceContainer::default(),
            hooks: LifecycleHooks::default(),
        }
    }

    pub fn registry_mut(&mut self) -> &mut ModuleRegistry {
        &mut self.registry
    }

    pub fn hooks_mut(&mut self) -> &mut LifecycleHooks {
        &mut self.hooks
    }

    pub async fn run<F, Fut>(&self, serve: F) -> AppResult<()>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = AppResult<()>>,
    {
        for hook in &self.hooks.before_start {
            hook();
        }

        let ctx = ModuleContext {
            app_name: self.app_name,
            container: self.container.clone(),
        };

        for module in self.registry.modules() {
            module.start(&ctx).await?;
        }

        serve().await?;

        for hook in &self.hooks.before_shutdown {
            hook();
        }

        for module in self.registry.modules().iter().rev() {
            module.shutdown(&ctx).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn container_roundtrip() {
        let c = ServiceContainer::default();
        c.insert::<String>("abc".to_string()).await;
        let value = c.get::<String>().await.expect("value should exist");
        assert_eq!(value.as_str(), "abc");
    }
}
